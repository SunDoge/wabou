use http::header::CONTENT_TYPE;
use serde::de::{DeserializeOwned, Error as _};

use crate::{ContentType, Error, ResponseContent};

fn content_type(response: &reqwest::Response) -> ContentType {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .into()
}

/// [process_with_json_response] is generic, which means it gets monomorphized for every type it's
/// used with. This function contains the non-generic logic for processing the response so that it
/// doesn't get duplicated.
#[inline(never)]
async fn process_with_json_response_internal(
    request: reqwest_middleware::RequestBuilder,
) -> Result<String, crate::Error> {
    let response = request.send().await?;
    let status = response.status();
    let content_type = content_type(&response);
    let content = response.text().await?;

    if !status.is_client_error() && !status.is_server_error() {
        match content_type {
            ContentType::Json => Ok(content),
            ct => Err(Error::from(serde_json::Error::custom(format!(
                "Received `{ct:?}` content type response when JSON was expected"
            )))),
        }
    } else {
        Err(ResponseContent {
            status,
            message: content,
        }
        .into())
    }
}

/// Sends and processes a request expecting a JSON response, deserializing it into the type `T`.
pub async fn process_with_json_response<T: DeserializeOwned>(
    request: reqwest_middleware::RequestBuilder,
) -> Result<T, crate::Error> {
    process_with_json_response_internal(request)
        .await
        .and_then(|content| {
            let content = remove_duplicate_pascal_case_fields(content);
            serde_json::from_str(&content).map_err(Into::into)
        })
}

/// Prefer camelCase when a server emits both the legacy PascalCase property
/// and its current camelCase spelling. A lone PascalCase property is retained
/// so generated Serde aliases continue to support older servers.
pub fn remove_duplicate_pascal_case_fields(content: String) -> String {
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

    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return content;
    };
    visit(&mut value);
    serde_json::to_string(&value).unwrap_or(content)
}

/// Sends and processes a request expecting an empty response, returning `Ok(())` when successful.
#[inline(never)]
pub async fn process_with_empty_response(
    request: reqwest_middleware::RequestBuilder,
) -> Result<(), crate::Error> {
    let response = request.send().await?;
    let status = response.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = response.text().await?;
        Err(ResponseContent {
            status,
            message: content,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::remove_duplicate_pascal_case_fields;

    #[test]
    fn duplicate_pascal_case_fields_prefer_camel_case_recursively() {
        let normalized = remove_duplicate_pascal_case_fields(
            r#"{"Object":"legacy","object":"current","nested":{"Salt":"legacy","salt":"current"},"OnlyPascal":true}"#
                .to_owned(),
        );
        let value: serde_json::Value = serde_json::from_str(&normalized).unwrap();
        assert_eq!(value["object"], "current");
        assert!(value.get("Object").is_none());
        assert_eq!(value["nested"]["salt"], "current");
        assert_eq!(value["OnlyPascal"], true);
    }
}
