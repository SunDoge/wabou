use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use bitwarden_api_api::models::SyncResponseModel;
use bitwarden_core::auth::login::{
    PasswordLoginRequest, TwoFactorEmailRequest, TwoFactorProvider, TwoFactorRequest,
    response::two_factor::TwoFactorProviders,
};
use bitwarden_core::{ClientSettings, DeviceType};
use bitwarden_crypto_sync_handler::CryptoSyncHandler;
use bitwarden_pm::PasswordManagerClient;
use bitwarden_sync::{SyncClient, SyncHandler, SyncHandlerError, SyncRequest};
use bitwarden_vault::{Cipher, CipherListViewType, CipherType, CipherView};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use url::Url;
use zeroize::{Zeroize, Zeroizing};

const CLIPBOARD_CLEAR_AFTER: Duration = Duration::from_secs(30);
pub const AUTO_LOCK_AFTER: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub region: String,
    pub server_url: Option<String>,
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorSubmitRequest {
    pub provider: String,
    pub token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorOption {
    pub id: &'static str,
    pub label: &'static str,
    pub hint: Option<String>,
    pub supported: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum LoginOutcome {
    Authenticated { snapshot: VaultSnapshot },
    TwoFactorRequired { providers: Vec<TwoFactorOption> },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VaultItem {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub kind: &'static str,
    pub favorite: bool,
    pub has_username: bool,
    pub has_password: bool,
    pub has_totp: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultSnapshot {
    pub email: String,
    pub items: Vec<VaultItem>,
    pub decrypt_failures: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDetails {
    pub id: String,
    pub name: String,
    pub kind: &'static str,
    pub username: Option<String>,
    pub uris: Vec<String>,
    pub favorite: bool,
    pub has_password: bool,
    pub has_totp: bool,
}

#[derive(Default)]
struct CapturedVault {
    ciphers: Vec<Cipher>,
    parse_failures: usize,
}

#[derive(Default)]
struct CipherCapture(RwLock<CapturedVault>);

#[async_trait]
impl SyncHandler for CipherCapture {
    async fn on_sync(&self, response: &SyncResponseModel) -> Result<(), SyncHandlerError> {
        let mut ciphers = Vec::new();
        let mut parse_failures = 0;
        for model in response.ciphers.as_deref().unwrap_or_default() {
            match Cipher::try_from(model.clone()) {
                Ok(cipher) => ciphers.push(cipher),
                Err(_) => parse_failures += 1,
            }
        }
        *self.0.write().await = CapturedVault {
            ciphers,
            parse_failures,
        };
        Ok(())
    }
}

struct Session {
    email: String,
    client: PasswordManagerClient,
    sync: SyncClient,
    capture: Arc<CipherCapture>,
    last_activity: StdMutex<Instant>,
}

struct PendingLogin {
    email: String,
    client: PasswordManagerClient,
    password: Zeroizing<String>,
    created_at: Instant,
}

#[derive(Default)]
pub struct VaultService {
    session: Option<Session>,
    pending_login: Option<PendingLogin>,
}

pub type SharedVaultService = Arc<Mutex<VaultService>>;

impl VaultService {
    pub async fn login(
        &mut self,
        request: LoginRequest,
        password: Zeroizing<String>,
    ) -> Result<LoginOutcome, String> {
        self.lock();
        let settings = settings_for(&request.region, request.server_url.as_deref())?;
        let client = PasswordManagerClient::new(Some(settings));
        let email = request.email.trim().to_owned();

        let mut sdk_request = PasswordLoginRequest {
            email: email.clone(),
            password: password.to_string(),
            two_factor: None::<TwoFactorRequest>,
        };
        let login = client.0.auth().login_password(&sdk_request).await;
        sdk_request.password.zeroize();
        let login =
            login.map_err(|_| "Login failed. Check the server and credentials.".to_string())?;
        if !login.authenticated {
            let providers = login
                .two_factor
                .ok_or_else(|| "The server did not authenticate this account.".to_string())?;
            let options = two_factor_options(&providers);
            self.pending_login = Some(PendingLogin {
                email,
                client,
                password,
                created_at: Instant::now(),
            });
            return Ok(LoginOutcome::TwoFactorRequired { providers: options });
        }

        let snapshot = self.establish_session(email, client).await?;
        Ok(LoginOutcome::Authenticated { snapshot })
    }

    pub async fn submit_two_factor(
        &mut self,
        request: TwoFactorSubmitRequest,
    ) -> Result<LoginOutcome, String> {
        let mut pending = self
            .pending_login
            .take()
            .ok_or_else(|| "Two-step login has expired. Start again.".to_string())?;
        let provider = match parse_two_factor_provider(&request.provider) {
            Ok(provider) => provider,
            Err(error) => {
                self.pending_login = Some(pending);
                return Err(error);
            }
        };
        let token = request.token.trim();
        if token.is_empty() {
            self.pending_login = Some(pending);
            return Err("Enter the verification code.".into());
        }
        let mut sdk_request = PasswordLoginRequest {
            email: pending.email.clone(),
            password: pending.password.to_string(),
            two_factor: Some(TwoFactorRequest {
                token: token.to_owned(),
                provider,
                // Remembered-device tokens need secure persistence that this
                // prototype deliberately does not implement.
                remember: false,
            }),
        };
        let login = pending.client.0.auth().login_password(&sdk_request).await;
        sdk_request.password.zeroize();
        let login = match login {
            Ok(login) => login,
            Err(_) => {
                self.pending_login = Some(pending);
                return Err("Verification failed. Check the code and try again.".into());
            }
        };
        if !login.authenticated {
            self.pending_login = Some(pending);
            return Err("Verification failed. Check the code and try again.".into());
        }
        let email = pending.email.clone();
        let client = pending.client;
        pending.password.zeroize();
        let snapshot = self.establish_session(email, client).await?;
        Ok(LoginOutcome::Authenticated { snapshot })
    }

    pub async fn send_two_factor_email(&self) -> Result<(), String> {
        let pending = self
            .pending_login
            .as_ref()
            .ok_or_else(|| "Two-step login has expired. Start again.".to_string())?;
        let mut request = TwoFactorEmailRequest {
            email: pending.email.clone(),
            password: pending.password.to_string(),
        };
        let result = pending
            .client
            .0
            .auth()
            .send_two_factor_email(&request)
            .await;
        request.password.zeroize();
        result.map_err(|_| "Could not send the email verification code.".to_string())
    }

    pub fn cancel_two_factor(&mut self) {
        self.pending_login = None;
    }

    async fn establish_session(
        &mut self,
        email: String,
        client: PasswordManagerClient,
    ) -> Result<VaultSnapshot, String> {
        let sync = client.sync();
        let capture = Arc::new(CipherCapture::default());
        // Crypto must run before cipher capture is decrypted below: it installs
        // the user and organization keys supplied by the sync response.
        sync.register_sync_handler(Arc::new(CryptoSyncHandler::new(client.0.clone())));
        sync.register_sync_handler(capture.clone());
        sync.sync(SyncRequest {
            force: true,
            exclude_subdomains: None,
        })
        .await
        .map_err(|_| "Login succeeded, but the first vault sync failed.".to_string())?;

        self.session = Some(Session {
            email,
            client,
            sync,
            capture,
            last_activity: StdMutex::new(Instant::now()),
        });
        self.pending_login = None;
        self.snapshot().await
    }

    pub async fn refresh(&self) -> Result<VaultSnapshot, String> {
        let session = self.session.as_ref().ok_or_else(locked)?;
        session
            .sync
            .sync(SyncRequest {
                force: true,
                exclude_subdomains: None,
            })
            .await
            .map_err(|_| "Vault sync failed.".to_string())?;
        self.snapshot().await
    }

    pub async fn snapshot(&self) -> Result<VaultSnapshot, String> {
        let session = self.session.as_ref().ok_or_else(locked)?;
        touch(session);
        let capture = session.capture.0.read().await;
        let result = session
            .client
            .vault()
            .ciphers()
            .decrypt_list_with_failures(capture.ciphers.clone())
            .await;
        let mut items = result
            .successes
            .into_iter()
            .filter_map(to_vault_item)
            .collect::<Vec<_>>();
        items.sort_by_cached_key(|item| item.name.to_lowercase());
        Ok(VaultSnapshot {
            email: session.email.clone(),
            items,
            decrypt_failures: capture.parse_failures + result.failures.len(),
        })
    }

    pub async fn details(&self, id: &str) -> Result<ItemDetails, String> {
        let view = self.decrypt_one(id).await?;
        let login = view.login.as_ref();
        Ok(ItemDetails {
            id: view.id.map(|id| id.to_string()).unwrap_or_default(),
            name: view.name,
            kind: cipher_kind(view.r#type),
            username: login.and_then(|login| login.username.clone()),
            uris: login
                .and_then(|login| login.uris.as_ref())
                .into_iter()
                .flatten()
                .filter_map(|uri| uri.uri.clone())
                .collect(),
            favorite: view.favorite,
            has_password: login.and_then(|login| login.password.as_ref()).is_some(),
            has_totp: login.and_then(|login| login.totp.as_ref()).is_some(),
        })
    }

    pub async fn copy_field(&self, id: &str, field: &str) -> Result<(), String> {
        let view = self.decrypt_one(id).await?;
        let value = match field {
            "username" => view.login.and_then(|login| login.username),
            "password" => view.login.and_then(|login| login.password),
            _ => return Err("Unsupported copy field.".into()),
        }
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "This item does not contain that field.".to_string())?;
        write_clipboard(Zeroizing::new(value)).await
    }

    async fn decrypt_one(&self, id: &str) -> Result<CipherView, String> {
        let session = self.session.as_ref().ok_or_else(locked)?;
        touch(session);
        let cipher = session
            .capture
            .0
            .read()
            .await
            .ciphers
            .iter()
            .find(|cipher| {
                cipher
                    .id
                    .is_some_and(|cipher_id| cipher_id.to_string() == id)
            })
            .cloned()
            .ok_or_else(|| "Vault item not found.".to_string())?;
        session
            .client
            .vault()
            .ciphers()
            .decrypt(cipher)
            .await
            .map_err(|_| "The SDK could not decrypt this item.".to_string())
    }

    pub fn lock(&mut self) {
        self.session = None;
        self.pending_login = None;
    }

    pub fn lock_if_idle(&mut self) -> bool {
        let expired = self.session.as_ref().is_some_and(|session| {
            session
                .last_activity
                .lock()
                .map(|activity| activity.elapsed() >= AUTO_LOCK_AFTER)
                .unwrap_or(true)
        }) || self
            .pending_login
            .as_ref()
            .is_some_and(|pending| pending.created_at.elapsed() >= AUTO_LOCK_AFTER);
        if expired {
            self.lock();
        }
        expired
    }

    pub fn is_locked(&self) -> bool {
        self.session.is_none()
    }
}

fn two_factor_options(providers: &TwoFactorProviders) -> Vec<TwoFactorOption> {
    let mut options = Vec::new();
    if providers.authenticator.is_some() {
        options.push(TwoFactorOption {
            id: "authenticator",
            label: "Authenticator app",
            hint: None,
            supported: true,
        });
    }
    if let Some(email) = &providers.email {
        options.push(TwoFactorOption {
            id: "email",
            label: "Email",
            hint: Some(email.email.clone()),
            supported: true,
        });
    }
    if providers.yubi_key.is_some() {
        options.push(TwoFactorOption {
            id: "yubikey",
            label: "YubiKey OTP",
            hint: None,
            supported: true,
        });
    }
    if providers.duo.is_some() || providers.organization_duo.is_some() {
        options.push(TwoFactorOption {
            id: "duo",
            label: "Duo",
            hint: None,
            supported: false,
        });
    }
    if providers.web_authn.is_some() {
        options.push(TwoFactorOption {
            id: "webauthn",
            label: "WebAuthn",
            hint: None,
            supported: false,
        });
    }
    options
}

fn parse_two_factor_provider(provider: &str) -> Result<TwoFactorProvider, String> {
    match provider {
        "authenticator" => Ok(TwoFactorProvider::Authenticator),
        "email" => Ok(TwoFactorProvider::Email),
        "yubikey" => Ok(TwoFactorProvider::Yubikey),
        _ => Err("This two-step login provider is not supported yet.".into()),
    }
}

fn touch(session: &Session) {
    if let Ok(mut activity) = session.last_activity.lock() {
        *activity = Instant::now();
    }
}

fn to_vault_item(view: bitwarden_vault::CipherListView) -> Option<VaultItem> {
    let id = view.id?.to_string();
    let (has_username, has_password, has_totp) = match &view.r#type {
        CipherListViewType::Login(login) => (
            login
                .username
                .as_ref()
                .is_some_and(|value| !value.is_empty()),
            // The SDK intentionally keeps CopyableCipherFields private. The
            // detail view determines password availability after decrypting
            // the selected item.
            false,
            login.totp.is_some(),
        ),
        _ => (false, false, false),
    };
    Some(VaultItem {
        id,
        name: view.name,
        subtitle: view.subtitle,
        kind: list_kind(&view.r#type),
        favorite: view.favorite,
        has_username,
        has_password,
        has_totp,
    })
}

fn list_kind(kind: &CipherListViewType) -> &'static str {
    match kind {
        CipherListViewType::Login(_) => "login",
        CipherListViewType::SecureNote => "note",
        CipherListViewType::Card(_) => "card",
        CipherListViewType::Identity => "identity",
        CipherListViewType::SshKey => "ssh-key",
        CipherListViewType::BankAccount(_) => "bank-account",
        CipherListViewType::Passport => "passport",
        CipherListViewType::DriversLicense => "drivers-license",
    }
}

fn cipher_kind(kind: CipherType) -> &'static str {
    match kind {
        CipherType::Login => "login",
        CipherType::SecureNote => "note",
        CipherType::Card => "card",
        CipherType::Identity => "identity",
        CipherType::SshKey => "ssh-key",
        CipherType::BankAccount => "bank-account",
        CipherType::DriversLicense => "drivers-license",
        CipherType::Passport => "passport",
    }
}

fn settings_for(region: &str, custom_server: Option<&str>) -> Result<ClientSettings, String> {
    let mut settings = ClientSettings::default();
    settings.user_agent = "Wabou Bitwarden Read-only Demo/0.1".into();
    settings.device_type = desktop_device_type();
    match region {
        "us" => {}
        "eu" => {
            settings.identity_url = "https://identity.bitwarden.eu".into();
            settings.api_url = "https://api.bitwarden.eu".into();
        }
        "self-hosted" => {
            let base = validate_server(custom_server.unwrap_or_default())?;
            settings.identity_url = format!("{base}/identity");
            settings.api_url = format!("{base}/api");
        }
        _ => return Err("Unknown Bitwarden region.".into()),
    }
    Ok(settings)
}

fn validate_server(raw: &str) -> Result<String, String> {
    let mut url = Url::parse(raw.trim()).map_err(|_| "Enter a valid server URL.".to_string())?;
    let local_http =
        url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !local_http {
        return Err(
            "Self-hosted servers must use HTTPS (HTTP is allowed only on localhost).".into(),
        );
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("Server URL cannot contain a query or fragment.".into());
    }
    let path = url.path().trim_end_matches('/').to_owned();
    url.set_path(&path);
    Ok(url.to_string().trim_end_matches('/').to_owned())
}

const fn desktop_device_type() -> DeviceType {
    #[cfg(target_os = "windows")]
    return DeviceType::WindowsDesktop;
    #[cfg(target_os = "macos")]
    return DeviceType::MacOsDesktop;
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return DeviceType::LinuxDesktop;
}

fn locked() -> String {
    "Vault is locked.".into()
}

async fn write_clipboard(value: Zeroizing<String>) -> Result<(), String> {
    let fingerprint = blake3::hash(value.as_bytes());
    tokio::task::spawn_blocking(move || {
        let mut clipboard = arboard::Clipboard::new().map_err(|_| ())?;
        clipboard.set_text(value.as_str()).map_err(|_| ())
    })
    .await
    .map_err(|_| "Clipboard task failed.".to_string())?
    .map_err(|_| "Could not write to the system clipboard.".to_string())?;
    tokio::spawn(async move {
        tokio::time::sleep(CLIPBOARD_CLEAR_AFTER).await;
        let _ = tokio::task::spawn_blocking(move || {
            let mut clipboard = arboard::Clipboard::new().ok()?;
            let current = clipboard.get_text().ok()?;
            if blake3::hash(current.as_bytes()) == fingerprint {
                let _ = clipboard.set_text("");
            }
            Some(())
        })
        .await;
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_validation_requires_tls_except_for_loopback() {
        assert_eq!(
            validate_server("https://vault.example.test/").unwrap(),
            "https://vault.example.test"
        );
        assert!(validate_server("http://vault.example.test").is_err());
        assert_eq!(
            validate_server("http://127.0.0.1:8080/").unwrap(),
            "http://127.0.0.1:8080"
        );
    }

    #[test]
    fn lock_discards_the_entire_session() {
        let mut service = VaultService::default();
        service.lock();
        assert!(service.is_locked());
    }

    #[test]
    fn two_factor_parser_accepts_only_supported_code_flows() {
        assert!(matches!(
            parse_two_factor_provider("authenticator"),
            Ok(TwoFactorProvider::Authenticator)
        ));
        assert!(matches!(
            parse_two_factor_provider("email"),
            Ok(TwoFactorProvider::Email)
        ));
        assert!(matches!(
            parse_two_factor_provider("yubikey"),
            Ok(TwoFactorProvider::Yubikey)
        ));
        assert!(parse_two_factor_provider("duo").is_err());
        assert!(parse_two_factor_provider("webauthn").is_err());
    }
}
