//! WHATWG URL parsing and mutation backed by the existing Rust `url` crate.
//! No JavaScript handles are retained by these synchronous host functions.
use rquickjs::{Ctx, Exception, Object, Result};
use url::{Url, quirks};

pub fn url_operation<'js>(
    ctx: Ctx<'js>,
    input: String,
    base: String,
    field: String,
    value: String,
) -> Result<Object<'js>> {
    let base = if base.is_empty() {
        None
    } else {
        Some(Url::parse(&base).map_err(|_| Exception::throw_type(&ctx, "Invalid base URL"))?)
    };
    let mut url = Url::options()
        .base_url(base.as_ref())
        .parse(&input)
        .map_err(|_| Exception::throw_type(&ctx, "Invalid URL"))?;
    // quirks implements Web IDL URL setter behavior (invalid setters are ignored).
    match field.as_str() {
        "href" => {
            url = Url::parse(&value).map_err(|_| Exception::throw_type(&ctx, "Invalid URL"))?;
        }
        "protocol" => {
            let _ = quirks::set_protocol(&mut url, &value);
        }
        "username" => {
            let _ = quirks::set_username(&mut url, &value);
        }
        "password" => {
            let _ = quirks::set_password(&mut url, &value);
        }
        "host" => {
            let _ = quirks::set_host(&mut url, &value);
        }
        "hostname" => {
            let _ = quirks::set_hostname(&mut url, &value);
        }
        "port" => {
            let _ = quirks::set_port(&mut url, &value);
        }
        "pathname" => quirks::set_pathname(&mut url, &value),
        "search" => quirks::set_search(&mut url, &value),
        "hash" => quirks::set_hash(&mut url, &value),
        _ => {}
    }
    let result = Object::new(ctx)?;
    for (key, value) in [
        ("href", quirks::href(&url)),
        ("protocol", quirks::protocol(&url)),
        ("username", quirks::username(&url)),
        ("password", quirks::password(&url)),
        ("host", quirks::host(&url)),
        ("hostname", quirks::hostname(&url)),
        ("port", quirks::port(&url)),
        ("pathname", quirks::pathname(&url)),
        ("search", quirks::search(&url)),
        ("hash", quirks::hash(&url)),
    ] {
        result.set(key, value)?;
    }
    result.set("origin", url.origin().ascii_serialization())?;
    Ok(result)
}

pub fn form_decode(input: String) -> String {
    let pairs: Vec<(String, String)> = url::form_urlencoded::parse(input.as_bytes())
        .into_owned()
        .collect();
    serde_json::to_string(&pairs).expect("string pairs serialize")
}
pub fn form_encode<'js>(ctx: Ctx<'js>, input: String) -> Result<String> {
    let pairs: Vec<(String, String)> = serde_json::from_str(&input)
        .map_err(|_| Exception::throw_type(&ctx, "Invalid query pairs"))?;
    Ok(url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish())
}
