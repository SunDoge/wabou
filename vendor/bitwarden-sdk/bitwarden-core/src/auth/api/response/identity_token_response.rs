use bitwarden_api_base::ResponseContent;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::{
    api::response::{
        IdentityTokenFailResponse, IdentityTokenPayloadResponse, IdentityTokenRefreshResponse,
        IdentityTokenSuccessResponse, IdentityTwoFactorResponse,
    },
    login::LoginError,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) enum IdentityTokenResponse {
    Authenticated(IdentityTokenSuccessResponse),
    Payload(IdentityTokenPayloadResponse),
    Refreshed(IdentityTokenRefreshResponse),
    TwoFactorRequired(Box<IdentityTwoFactorResponse>),
}

pub(crate) fn parse_identity_response(
    status: StatusCode,
    response: String,
) -> Result<IdentityTokenResponse, LoginError> {
    let response = remove_duplicate_pascal_case_fields(response);
    if let Ok(r) = serde_json::from_str::<IdentityTokenSuccessResponse>(&response) {
        Ok(IdentityTokenResponse::Authenticated(r))
    } else if let Ok(r) = serde_json::from_str::<IdentityTokenPayloadResponse>(&response) {
        Ok(IdentityTokenResponse::Payload(r))
    } else if let Ok(r) = serde_json::from_str::<IdentityTokenRefreshResponse>(&response) {
        Ok(IdentityTokenResponse::Refreshed(r))
    } else if let Ok(r) = serde_json::from_str::<IdentityTwoFactorResponse>(&response) {
        Ok(IdentityTokenResponse::TwoFactorRequired(Box::new(r)))
    } else if let Ok(r) = serde_json::from_str::<IdentityTokenFailResponse>(&response) {
        Err(LoginError::IdentityFail(r))
    } else {
        Err(LoginError::Api(
            ResponseContent {
                status,
                message: response,
            }
            .into(),
        ))
    }
}

/// Vaultwarden can serialize both legacy PascalCase and current camelCase
/// properties into the same response. Serde aliases accept either spelling in
/// isolation, but reject both as a duplicate field. Prefer camelCase only when
/// both spellings are present, recursively.
fn remove_duplicate_pascal_case_fields(response: String) -> String {
    fn visit(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(object) => {
                for value in object.values_mut() {
                    visit(value);
                }
                let duplicates = object
                    .keys()
                    .filter_map(|key| {
                        let first = key.chars().next()?;
                        if !first.is_ascii_uppercase() {
                            return None;
                        }
                        let camel = format!("{}{}", first.to_ascii_lowercase(), &key[1..]);
                        object.contains_key(&camel).then(|| key.clone())
                    })
                    .collect::<Vec<_>>();
                for key in duplicates {
                    object.remove(&key);
                }
            }
            serde_json::Value::Array(values) => values.iter_mut().for_each(visit),
            _ => {}
        }
    }

    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&response) else {
        return response;
    };
    visit(&mut value);
    serde_json::to_string(&value).unwrap_or(response)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn success() {
        let expected = IdentityTokenSuccessResponse::default();
        let success = serde_json::to_string(&expected).unwrap();
        let expected = IdentityTokenResponse::Authenticated(expected);
        let actual = parse_identity_response(StatusCode::OK, success).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn two_factor() {
        let expected = Box::<IdentityTwoFactorResponse>::default();
        let two_factor = serde_json::to_string(&expected).unwrap();
        let expected = IdentityTokenResponse::TwoFactorRequired(expected);
        let actual = parse_identity_response(StatusCode::BAD_REQUEST, two_factor).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn duplicate_pascal_case_fields_prefer_camel_case_recursively() {
        let normalized = remove_duplicate_pascal_case_fields(
            r#"{"Kdf":0,"kdf":1,"Nested":{"Salt":"legacy","salt":"current"},"OnlyPascal":true}"#
                .to_owned(),
        );
        let value: serde_json::Value = serde_json::from_str(&normalized).unwrap();
        assert_eq!(value["kdf"], 1);
        assert!(value.get("Kdf").is_none());
        assert_eq!(value["Nested"]["salt"], "current");
        assert_eq!(value["OnlyPascal"], true);
    }
}
