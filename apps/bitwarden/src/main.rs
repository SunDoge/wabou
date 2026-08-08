mod secure_input;
mod service;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use serde_json::json;
use snafu::{ResultExt, Whatever};
use tokio::sync::Mutex;
use wabou_quick::rquickjs::{Function, prelude::Async};
use wabou_quick::{HostBuilder, WindowOptions};

use secure_input::{SharedSecret, take_secret};
use service::{LoginRequest, SharedVaultService, VaultService};

fn response<T: Serialize>(result: Result<T, String>) -> String {
    match result {
        Ok(value) => json!({ "ok": true, "value": value }).to_string(),
        Err(error) => json!({ "ok": false, "error": error }).to_string(),
    }
}

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let service: SharedVaultService = Arc::new(Mutex::new(VaultService::default()));
    let auto_lock_started = Arc::new(AtomicBool::new(false));
    let master_password = SharedSecret::default();
    HostBuilder::new()
        .window(
            WindowOptions::new()
                .title("Wabou Vault")
                .initial_inner_size(1100, 720)
                .min_inner_size(760, 520),
        )
        // Debug snapshots can contain decrypted item titles and usernames.
        .devtools(false)
        .widget("secure-input", {
            let master_password = master_password.clone();
            move || Box::new(secure_input::SecureInput::new(master_password.clone()))
        })
        .capability("vault", move |ctx, capability| {
            let login_service = service.clone();
            let login_auto_lock_started = auto_lock_started.clone();
            let login_password = master_password.clone();
            capability.set(
                "login",
                Function::new(
                    ctx.clone(),
                    Async(move |raw: String| {
                        let service = login_service.clone();
                        let auto_lock_started = login_auto_lock_started.clone();
                        let password = take_secret(&login_password);
                        async move {
                            let request = serde_json::from_str::<LoginRequest>(&raw)
                                .map_err(|_| "Invalid login request.".to_string());
                            let result = match request {
                                Ok(_request) if password.is_empty() => {
                                    Err("Enter your master password.".to_string())
                                }
                                Ok(request) => service.lock().await.login(request, password).await,
                                Err(error) => Err(error),
                            };
                            if result.is_ok() && !auto_lock_started.swap(true, Ordering::AcqRel) {
                                let auto_lock_service = service.clone();
                                tokio::spawn(async move {
                                    loop {
                                        tokio::time::sleep(Duration::from_secs(15)).await;
                                        auto_lock_service.lock().await.lock_if_idle();
                                    }
                                });
                            }
                            response(result)
                        }
                    }),
                )?,
            )?;

            let refresh_service = service.clone();
            capability.set(
                "refresh",
                Function::new(
                    ctx.clone(),
                    Async(move || {
                        let service = refresh_service.clone();
                        async move { response(service.lock().await.refresh().await) }
                    }),
                )?,
            )?;

            let details_service = service.clone();
            capability.set(
                "details",
                Function::new(
                    ctx.clone(),
                    Async(move |id: String| {
                        let service = details_service.clone();
                        async move { response(service.lock().await.details(&id).await) }
                    }),
                )?,
            )?;

            let copy_service = service.clone();
            capability.set(
                "copy",
                Function::new(
                    ctx.clone(),
                    Async(move |id: String, field: String| {
                        let service = copy_service.clone();
                        async move {
                            response(
                                service
                                    .lock()
                                    .await
                                    .copy_field(&id, &field)
                                    .await
                                    .map(|()| true),
                            )
                        }
                    }),
                )?,
            )?;

            let lock_service = service.clone();
            capability.set(
                "lock",
                Function::new(
                    ctx.clone(),
                    Async(move || {
                        let service = lock_service.clone();
                        async move {
                            service.lock().await.lock();
                            response(Ok(true))
                        }
                    }),
                )?,
            )?;

            let status_service = service.clone();
            capability.set(
                "isLocked",
                Function::new(
                    ctx,
                    Async(move || {
                        let service = status_service.clone();
                        async move { response(Ok(service.lock().await.is_locked())) }
                    }),
                )?,
            )?;
            Ok(())
        })
        .run()
        .whatever_context("failed to run the Bitwarden read-only demo")
}
