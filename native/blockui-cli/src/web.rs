use blockui_dev::project::{Project, checked};
use serde_json::Value;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
const TARGET: &str = "wasm32-unknown-unknown";
pub fn doctor(project: &Project) -> Result<(), String> {
    let targets = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|e| format!("cannot query Rust targets: {e}"))?;
    if !targets.status.success()
        || !String::from_utf8_lossy(&targets.stdout)
            .lines()
            .any(|line| line == TARGET)
    {
        return Err(format!(
            "missing Rust target; run: rustup target add {TARGET}"
        ));
    }
    let expected = project.metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "wasm-bindgen")
        .and_then(|p| p["version"].as_str())
        .ok_or("browser application needs a wasm-bindgen dependency")?;
    let output=Command::new("wasm-bindgen").arg("--version").output().map_err(|_|format!("install the matching generator: cargo install wasm-bindgen-cli --version {expected} --locked"))?;
    if !output.status.success()
        || String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .nth(1)
            != Some(expected)
    {
        return Err(format!(
            "wasm-bindgen version mismatch; run: cargo install wasm-bindgen-cli --version {expected} --locked"
        ));
    }
    println!("OK: {TARGET}, wasm-bindgen {expected}");
    Ok(())
}
pub fn assets(project: &Project) -> Result<PathBuf, String> {
    let assets = project.package["metadata"]["blockui"]["web-assets"]
        .as_str()
        .unwrap_or("web");
    project
        .package_root()
        .join(assets)
        .canonicalize()
        .map_err(|e| format!("cannot find browser assets ({assets}): {e}"))
}
pub fn build(project: &Project, features: &str) -> Result<PathBuf, String> {
    doctor(project)?;
    let root = assets(project)?;
    if !root.join("index.html").is_file() {
        return Err("browser asset directory needs index.html".into());
    }
    let libraries: Vec<_> = project.package["targets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| {
            t["crate_types"]
                .as_array()
                .is_some_and(|k| k.contains(&Value::from("cdylib")))
        })
        .collect();
    if libraries.len() != 1 {
        return Err("browser app needs one cdylib library target".into());
    }
    let name = libraries[0]["name"].as_str().unwrap();
    let mut command = project.cargo("build", features);
    command.args(["--lib", "--release", "--target", TARGET]);
    checked(&mut command)?;
    let wasm = project
        .target
        .join(TARGET)
        .join("release")
        .join(format!("{name}.wasm"));
    let stage = tempfile::tempdir_in(&root).map_err(|e| e.to_string())?;
    let next = stage.path().join("pkg");
    checked(
        Command::new("wasm-bindgen")
            .args(["--target", "web", "--out-dir"])
            .arg(&next)
            .arg(&wasm),
    )?;
    // Opt-in applications share the SDK browser backend without copying its
    // implementation into source control. Publish it with the Wasm generation.
    if project.package["metadata"]["blockui"]["web-shared-assets"].as_bool() == Some(true) {
        let sdk_assets = project.sdk_root()?.join("native/blockui-cli/assets");
        for (source, destination) in [("renderer.js", "renderer.js"), ("style.css", "base.css")] {
            fs::copy(sdk_assets.join(source), next.join(destination)).map_err(|e| e.to_string())?;
        }
    }
    let destination = root.join("pkg");
    if destination.is_symlink() {
        return Err("generated pkg directory must not be a symlink".into());
    }
    let previous = stage.path().join("previous");
    if destination.exists() {
        fs::rename(&destination, &previous).map_err(|e| e.to_string())?;
    }
    if let Err(error) = fs::rename(&next, &destination) {
        if previous.exists()
            && let Err(restore) = fs::rename(&previous, &destination)
        {
            let preserved = stage.keep();
            return Err(format!(
                "{error}; restore failed: {restore}; previous assets retained in {}",
                preserved.display()
            ));
        }
        return Err(error.to_string());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let bytes = fs::read_dir(&destination)
            .map_err(|e| e.to_string())?
            .map(|e| e.and_then(|e| e.metadata()).map(|m| m.blocks() * 512))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
            .iter()
            .sum::<u64>();
        println!(
            "Built {} ({} allocated bytes)",
            destination.display(),
            bytes
        );
    }
    #[cfg(not(unix))]
    println!("Built {}", destination.display());
    Ok(root)
}
fn decode_path(path: &str) -> Option<String> {
    let mut decoded = Vec::new();
    let mut bytes = path.as_bytes().iter().copied();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let a = (bytes.next()? as char).to_digit(16)?;
            let b = (bytes.next()? as char).to_digit(16)?;
            decoded.push((a * 16 + b) as u8);
        } else {
            decoded.push(byte);
        }
    }
    String::from_utf8(decoded).ok()
}
fn asset(root: &Path, url: &str) -> Option<PathBuf> {
    let path = decode_path(url.split('?').next()?)?;
    if !path.starts_with('/') || path.contains(['\0', '\\']) {
        return None;
    }
    let path = if path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };
    let candidate = root.join(path).canonicalize().ok()?;
    (candidate.starts_with(root) && candidate.is_file()).then_some(candidate)
}
fn connection(
    mut stream: TcpStream,
    root: &Path,
    dev: Option<&crate::web_hot::Dev>,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let mut request = Vec::new();
    let mut buffer = [0; 1024];
    while request.len() < 8192 && !request.windows(4).any(|b| b == b"\r\n\r\n") {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            return Ok(());
        }
        request.extend_from_slice(&buffer[..n]);
    }
    let text = String::from_utf8_lossy(&request);
    let mut fields = text.lines().next().unwrap_or("").split_whitespace();
    let method = fields.next().unwrap_or("");
    let url = fields.next().unwrap_or("");
    if let Some(dev) = dev {
        if method == "GET" && url == "/__blockui/events" {
            return dev.events(stream);
        }
        let special = match url.split('?').next().unwrap_or("") {
            "/__blockui/status" => Some((
                dev.status.lock().unwrap().to_string().into_bytes(),
                "application/json",
            )),
            "/__blockui/client.js" => Some((
                include_bytes!("../assets/hot.js").to_vec(),
                "text/javascript",
            )),
            _ => None,
        };
        if let Some((body, mime)) = special {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
                body.len()
            )?;
            return stream.write_all(&body);
        }
    }
    let mut served_root = root;
    let mut served_url = url;
    if let Some(dev) = dev
        && let Some(path) = url.strip_prefix("/__blockui/generations/")
    {
        served_root = &dev.generations;
        served_url = path;
    }
    let served_url = if served_url.starts_with('/') {
        served_url.to_owned()
    } else {
        format!("/{served_url}")
    };
    let (status, body, mime) = if matches!(method, "GET" | "HEAD") {
        match asset(served_root, &served_url) {
            Some(path) => {
                let mime = match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
                    "html" => "text/html; charset=utf-8",
                    "js" => "text/javascript; charset=utf-8",
                    "css" => "text/css; charset=utf-8",
                    "wasm" => "application/wasm",
                    "json" => "application/json",
                    "svg" => "image/svg+xml",
                    "png" => "image/png",
                    _ => "application/octet-stream",
                };
                (
                    "200 OK",
                    {
                        let mut bytes = fs::read(path)?;
                        if let Some(dev) = dev.filter(|_| mime.starts_with("text/html")) {
                            let status = dev.status.lock().unwrap().clone();
                            let prefix =
                                format!("/__blockui/generations/{}/", status["generation"]);
                            let html = String::from_utf8_lossy(&bytes)
                                .replace("./pkg/", &prefix)
                                .replace("\"/pkg/", &format!("\"{prefix}"))
                                .replace("'/pkg/", &format!("'{prefix}"));
                            bytes = html.into_bytes();
                            bytes.extend_from_slice(format!(
                                "\n<script type=\"module\" src=\"/__blockui/client.js\" data-renderer=\"{}\"></script>\n",
                                status["renderer"].as_str().unwrap_or("")
                            ).as_bytes());
                        }
                        bytes
                    },
                    mime,
                )
            }
            None => ("404 Not Found", b"Not found".to_vec(), "text/plain"),
        }
    } else {
        (
            "405 Method Not Allowed",
            b"Method not allowed".to_vec(),
            "text/plain",
        )
    };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    if method != "HEAD" {
        stream.write_all(&body)?;
    }
    Ok(())
}
pub fn serve(
    root: &Path,
    port: u16,
    dev: Option<std::sync::Arc<crate::web_hot::Dev>>,
) -> Result<(), String> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    println!(
        "Serving browser at http://{}/",
        listener.local_addr().map_err(|e| e.to_string())?
    );
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let root = root.to_owned();
                let dev = dev.clone();
                std::thread::spawn(move || {
                    if let Err(e) = connection(stream, &root, dev.as_deref()) {
                        eprintln!("HTTP request: {e}");
                    }
                });
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn assets_cannot_escape_root_or_follow_external_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("web");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("index.html"), "hello").unwrap();
        fs::write(dir.path().join("secret"), "secret").unwrap();
        assert_eq!(asset(&root, "/?cache=1"), Some(root.join("index.html")));
        for url in ["/../secret", "/%2e%2e/secret", "/%00", "/%zz"] {
            assert!(asset(&root, url).is_none());
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.path().join("secret"), root.join("outside")).unwrap();
            assert!(asset(&root, "/outside").is_none());
        }
    }
}
