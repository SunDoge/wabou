use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use directories::ProjectDirs;
use oar_ocr::prelude::{OAROCR, OAROCRBuilder, TextRegion};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wabou::{
    ImageResourceHandle, ImageResourceStore, JsonCapability, JsonCapabilityContract, JsonMethod,
    rquickjs,
};

pub const CAPABILITY: JsonCapabilityContract = JsonCapabilityContract::new("mangaReader", 1);
const LIST_IMAGES: JsonMethod<ListImagesRequest, Vec<ImagePage>> = JsonMethod::new("listImages");
const DESCRIBE_IMAGES: JsonMethod<DescribeImagesRequest, Vec<ImagePage>> =
    JsonMethod::new("describeImages");
const MODEL_STATUS: JsonMethod<(), OcrModelStatus> = JsonMethod::no_request("modelStatus");
const DOWNLOAD_MODEL: JsonMethod<(), OcrModelStatus> = JsonMethod::no_request("downloadModel");
const RECOGNIZE_PAGE: JsonMethod<RecognizePageRequest, Vec<OcrRegion>> =
    JsonMethod::new("recognizePage");
const TRANSLATE: JsonMethod<TranslateRequest, Vec<String>> = JsonMethod::new("translate");

const MODEL_VERSION: &str = "ppocrv6-small-0.7.0";
const RELEASE_BASE: &str = "https://github.com/GreatV/oar-ocr/releases/download/v0.7.0";
const MODEL_FILES: [ModelFile; 3] = [
    ModelFile {
        name: "pp-ocrv6_small_det.onnx",
        size: 9_880_512,
        sha256: "d73e0058b7a8086bbd57f3d10b8bcd4ff95363f67e06e2762b5e814fe9c9410e",
    },
    ModelFile {
        name: "pp-ocrv6_small_rec.onnx",
        size: 21_159_378,
        sha256: "5435fd747c9e0efe15a96d0b378d5bd157e9492ed8fd80edf08f30d02fa24634",
    },
    ModelFile {
        name: "ppocrv6_dict.txt",
        size: 74_947,
        sha256: "b5f2bfe2bdd9448429e3e82b51c789775d9b42f2403d08200662eb77e401c5d",
    },
];

#[derive(Clone, Copy)]
struct ModelFile {
    name: &'static str,
    size: u64,
    sha256: &'static str,
}

#[derive(Clone)]
pub struct ReaderService(Arc<ReaderState>);

struct ReaderState {
    model_dir: PathBuf,
    engine: Mutex<Option<OAROCR>>,
    http: reqwest::Client,
    images: ImageResourceStore,
}

impl ReaderService {
    pub fn new(images: ImageResourceStore) -> Result<Self, String> {
        let project = ProjectDirs::from("dev", "Wabou", "Manga OCR")
            .ok_or_else(|| "could not resolve application directories".to_owned())?;
        let model_dir = project.data_dir().join("models").join(MODEL_VERSION);
        Ok(Self(Arc::new(ReaderState {
            model_dir,
            engine: Mutex::new(None),
            http: reqwest::Client::builder()
                .user_agent("MangaOcrWabou/0.1")
                .build()
                .map_err(|error| error.to_string())?,
            images,
        })))
    }

    fn model_status(&self) -> OcrModelStatus {
        let installed = MODEL_FILES
            .iter()
            .all(|file| model_file_valid(&self.0.model_dir, *file));
        OcrModelStatus {
            installed,
            ready: installed && self.0.engine.lock().is_ok_and(|engine| engine.is_some()),
            version: MODEL_VERSION.to_owned(),
        }
    }

    async fn download_model(&self) -> Result<OcrModelStatus, String> {
        fs::create_dir_all(&self.0.model_dir).map_err(|error| error.to_string())?;
        for file in MODEL_FILES {
            let target = self.0.model_dir.join(file.name);
            if model_file_valid(&self.0.model_dir, file) {
                continue;
            }
            let bytes = self
                .0
                .http
                .get(format!("{RELEASE_BASE}/{}", file.name))
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|error| format!("failed to download {}: {error}", file.name))?
                .bytes()
                .await
                .map_err(|error| error.to_string())?;
            let partial = target.with_extension("part");
            fs::write(&partial, &bytes).map_err(|error| error.to_string())?;
            if bytes.len() as u64 != file.size || sha256(&bytes) != file.sha256 {
                let _ = fs::remove_file(&partial);
                return Err(format!("model checksum failed: {}", file.name));
            }
            fs::rename(&partial, &target).map_err(|error| error.to_string())?;
        }
        if let Ok(mut engine) = self.0.engine.lock() {
            *engine = None;
        }
        Ok(self.model_status())
    }

    fn recognize(&self, handle: ImageResourceHandle) -> Result<Vec<OcrRegion>, String> {
        let image = self
            .0
            .images
            .get(handle)
            .ok_or_else(|| "image resource is missing or stale".to_owned())?
            .to_rgb8();
        let (width, height) = image.dimensions();
        let mut engine = self
            .0
            .engine
            .lock()
            .map_err(|_| "OCR engine lock poisoned".to_owned())?;
        if engine.is_none() {
            *engine = Some(
                OAROCRBuilder::new(
                    self.0.model_dir.join(MODEL_FILES[0].name),
                    self.0.model_dir.join(MODEL_FILES[1].name),
                    self.0.model_dir.join(MODEL_FILES[2].name),
                )
                .region_batch_size(64)
                .build()
                .map_err(|error| format!("failed to initialize OCR: {error}"))?,
            );
        }
        let results = engine
            .as_ref()
            .expect("engine initialized")
            .predict(vec![image])
            .map_err(|error| format!("OCR inference failed: {error}"))?;
        let mut regions = results[0]
            .text_regions
            .iter()
            .filter_map(|region| normalized_region(region, width, height))
            .collect::<Vec<_>>();
        regions.sort_by(|a, b| b.x.total_cmp(&a.x).then_with(|| a.y.total_cmp(&b.y)));
        Ok(regions)
    }

    async fn translate(&self, request: TranslateRequest) -> Result<Vec<String>, String> {
        if request.api_key.trim().is_empty() {
            return Err("OpenRouter API key is required".to_owned());
        }
        if request.texts.is_empty() {
            return Ok(Vec::new());
        }
        let numbered = request
            .texts
            .iter()
            .enumerate()
            .map(|(index, text)| format!("{}. {}", index + 1, text))
            .collect::<Vec<_>>()
            .join("\n");
        let response = self
            .0
            .http
            .post("https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(request.api_key)
            .json(&serde_json::json!({
                "model": request.model,
                "messages": [
                    {"role": "system", "content": format!("Translate Japanese manga text to {}. Return only a JSON object shaped as {{\"translations\":[\"...\"]}}, preserving the original order and count.", request.target_language)},
                    {"role": "user", "content": numbered}
                ],
                "response_format": {"type": "json_object"}
            }))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| format!("OpenRouter request failed: {error}"))?
            .json::<serde_json::Value>()
            .await
            .map_err(|error| error.to_string())?;
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| "OpenRouter returned no message".to_owned())?;
        parse_translation_array(content, request.texts.len())
    }
}

pub fn mount(capability: JsonCapability<'_>, service: ReaderService) -> rquickjs::Result<()> {
    let list = service.clone();
    capability.method(LIST_IMAGES, move |request: ListImagesRequest| {
        let service = list.clone();
        async move { service.list_images(&request.directory) }
    })?;
    let describe = service.clone();
    capability.method(DESCRIBE_IMAGES, move |request: DescribeImagesRequest| {
        let service = describe.clone();
        async move {
            request
                .paths
                .iter()
                .map(|path| service.describe_image(Path::new(path)))
                .collect()
        }
    })?;
    let status = service.clone();
    capability.method(MODEL_STATUS, move |(): ()| {
        let service = status.clone();
        async move { Ok::<_, String>(service.model_status()) }
    })?;
    let download = service.clone();
    capability.method(DOWNLOAD_MODEL, move |(): ()| {
        let service = download.clone();
        async move { service.download_model().await }
    })?;
    let recognize = service.clone();
    capability.method(RECOGNIZE_PAGE, move |request: RecognizePageRequest| {
        let service = recognize.clone();
        async move {
            tokio::task::spawn_blocking(move || service.recognize(request.handle))
                .await
                .map_err(|error| error.to_string())?
        }
    })?;
    capability.method(TRANSLATE, move |request: TranslateRequest| {
        let service = service.clone();
        async move { service.translate(request).await }
    })?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListImagesRequest {
    directory: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DescribeImagesRequest {
    paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecognizePageRequest {
    handle: ImageResourceHandle,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranslateRequest {
    texts: Vec<String>,
    api_key: String,
    model: String,
    target_language: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImagePage {
    id: String,
    path: String,
    name: String,
    width: u32,
    height: u32,
    handle: ImageResourceHandle,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OcrModelStatus {
    installed: bool,
    ready: bool,
    version: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OcrRegion {
    id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
    confidence: f32,
}

fn supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif"
            )
        })
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn model_file_valid(directory: &Path, file: ModelFile) -> bool {
    let path = directory.join(file.name);
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    bytes.len() as u64 == file.size && sha256(&bytes) == file.sha256
}

impl ReaderService {
    fn list_images(&self, directory: &str) -> Result<Vec<ImagePage>, String> {
        let mut paths = fs::read_dir(directory)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && supported_image(path))
            .collect::<Vec<_>>();
        paths.sort_by(|a, b| natord::compare(&natural_name(a), &natural_name(b)));
        paths.iter().map(|path| self.describe_image(path)).collect()
    }

    fn describe_image(&self, path: &Path) -> Result<ImagePage, String> {
        let canonical = path.canonicalize().map_err(|error| error.to_string())?;
        let handle = self.0.images.create_file(&canonical)?;
        let (width, height) = self
            .0
            .images
            .get(handle)
            .ok_or_else(|| "new image resource did not resolve".to_owned())?
            .dimensions();
        let value = canonical.to_string_lossy().into_owned();
        Ok(ImagePage {
            id: value.clone(),
            name: natural_name(&canonical),
            path: value,
            width,
            height,
            handle,
        })
    }
}

fn natural_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_owned()
}

fn normalized_region(region: &TextRegion, width: u32, height: u32) -> Option<OcrRegion> {
    let (text, confidence) = region.text_with_confidence()?;
    let points = &region.bounding_box.points;
    let min_x = points.iter().map(|point| point.x).reduce(f32::min)?;
    let min_y = points.iter().map(|point| point.y).reduce(f32::min)?;
    let max_x = points.iter().map(|point| point.x).reduce(f32::max)?;
    let max_y = points.iter().map(|point| point.y).reduce(f32::max)?;
    let width = width as f32;
    let height = height as f32;
    Some(OcrRegion {
        id: format!("ocr-{min_x:.0}-{min_y:.0}"),
        x: (min_x / width).clamp(0.0, 1.0) * width,
        y: (min_y / height).clamp(0.0, 1.0) * height,
        width: ((max_x - min_x) / width).clamp(0.0, 1.0) * width,
        height: ((max_y - min_y) / height).clamp(0.0, 1.0) * height,
        text: text.to_owned(),
        confidence,
    })
}

fn parse_translation_array(content: &str, expected: usize) -> Result<Vec<String>, String> {
    let value: serde_json::Value = serde_json::from_str(content)
        .or_else(|_| {
            let start = content.find('[').unwrap_or(0);
            let end = content.rfind(']').map_or(content.len(), |index| index + 1);
            serde_json::from_str(&content[start..end])
        })
        .map_err(|error| format!("translation response was not JSON: {error}"))?;
    let array = value
        .as_array()
        .or_else(|| value.get("translations")?.as_array())
        .ok_or_else(|| "translation response did not contain an array".to_owned())?;
    let translations = array
        .iter()
        .map(|value| value.as_str().unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    if translations.len() != expected {
        return Err(format!(
            "translation count mismatch: expected {expected}, got {}",
            translations.len()
        ));
    }
    Ok(translations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_parser_accepts_array_and_object_forms() {
        assert_eq!(
            parse_translation_array(r#"["a","b"]"#, 2).unwrap(),
            ["a", "b"]
        );
        assert_eq!(
            parse_translation_array(r#"{"translations":["a"]}"#, 1).unwrap(),
            ["a"]
        );
    }
}
