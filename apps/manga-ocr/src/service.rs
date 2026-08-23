use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use directories::ProjectDirs;
use image::ImageEncoder as _;
use oar_ocr::prelude::{OAROCR, OAROCRBuilder, TextRegion};
use serde::de::DeserializeOwned;
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
const ADJUST_BBOXES: JsonMethod<AdjustBboxesRequest, Vec<OcrRegion>> =
    JsonMethod::new("adjustBboxes");

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
    ocr: OcrWorker,
    llm: LlmWorker,
    http: reqwest::Client,
    images: ImageResourceStore,
}

struct LlmWorker {
    sender: mpsc::Sender<LlmCommand>,
}

enum LlmCommand {
    Translate {
        request: TranslateRequest,
        reply: tokio::sync::oneshot::Sender<Result<Vec<String>, String>>,
    },
    AdjustBboxes {
        request: AdjustBboxesRequest,
        reply: tokio::sync::oneshot::Sender<Result<Vec<OcrRegion>, String>>,
    },
}

#[derive(Clone)]
struct ResultCache {
    directory: PathBuf,
}

impl ResultCache {
    fn new(directory: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        prune_result_cache(&directory, 512);
        Ok(Self { directory })
    }

    fn path(&self, namespace: &str, key: &str) -> PathBuf {
        self.directory.join(format!("{namespace}-{key}.json"))
    }

    fn get<T: DeserializeOwned>(&self, namespace: &str, key: &str) -> Option<T> {
        serde_json::from_slice(&fs::read(self.path(namespace, key)).ok()?).ok()
    }

    fn insert<T: Serialize>(&self, namespace: &str, key: &str, value: &T) -> Result<(), String> {
        let target = self.path(namespace, key);
        let partial = target.with_extension("json.part");
        fs::write(
            &partial,
            serde_json::to_vec(value).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        fs::rename(partial, target).map_err(|error| error.to_string())
    }
}

impl LlmWorker {
    fn spawn(cache_dir: PathBuf, images: ImageResourceStore) -> Result<Self, String> {
        let cache = ResultCache::new(cache_dir.join("llm-results-v1"))?;
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("manga-ocr-llm".to_owned())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        tracing::error!(%error, "failed to start Manga OCR LLM runtime");
                        return;
                    }
                };
                let http = match reqwest::Client::builder()
                    .user_agent("MangaOcrWabou/0.1")
                    .connect_timeout(std::time::Duration::from_secs(10))
                    .timeout(std::time::Duration::from_secs(90))
                    .build()
                {
                    Ok(http) => http,
                    Err(error) => {
                        tracing::error!(%error, "failed to create Manga OCR LLM client");
                        return;
                    }
                };
                while let Ok(command) = receiver.recv() {
                    match command {
                        LlmCommand::Translate { request, reply } => {
                            let result = runtime.block_on(translate(&http, &cache, request));
                            let _ = reply.send(result);
                        }
                        LlmCommand::AdjustBboxes { request, reply } => {
                            let result =
                                runtime.block_on(adjust_bboxes(&http, &cache, &images, request));
                            let _ = reply.send(result);
                        }
                    }
                }
            })
            .map_err(|error| format!("failed to start LLM worker: {error}"))?;
        Ok(Self { sender })
    }

    async fn translate(&self, request: TranslateRequest) -> Result<Vec<String>, String> {
        let (reply, result) = tokio::sync::oneshot::channel();
        self.sender
            .send(LlmCommand::Translate { request, reply })
            .map_err(|_| "LLM worker has stopped".to_owned())?;
        result
            .await
            .map_err(|_| "LLM worker stopped before replying".to_owned())?
    }

    async fn adjust_bboxes(&self, request: AdjustBboxesRequest) -> Result<Vec<OcrRegion>, String> {
        let (reply, result) = tokio::sync::oneshot::channel();
        self.sender
            .send(LlmCommand::AdjustBboxes { request, reply })
            .map_err(|_| "LLM worker has stopped".to_owned())?;
        result
            .await
            .map_err(|_| "LLM worker stopped before replying".to_owned())?
    }
}

fn prune_result_cache(directory: &Path, capacity: usize) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut files = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((modified, entry.path()))
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|(modified, _)| *modified);
    let remove = files.len().saturating_sub(capacity);
    for (_, path) in files.into_iter().take(remove) {
        let _ = fs::remove_file(path);
    }
}

struct OcrWorker {
    sender: mpsc::Sender<OcrCommand>,
    ready: Arc<AtomicBool>,
}

enum OcrCommand {
    Recognize {
        handle: ImageResourceHandle,
        reply: tokio::sync::oneshot::Sender<Result<Vec<OcrRegion>, String>>,
    },
    Reset,
}

impl OcrWorker {
    fn spawn(model_dir: PathBuf, images: ImageResourceStore) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();
        let ready = Arc::new(AtomicBool::new(false));
        let worker_ready = ready.clone();
        std::thread::Builder::new()
            .name("manga-ocr-inference".to_owned())
            .spawn(move || {
                let mut engine = None;
                while let Ok(command) = receiver.recv() {
                    match command {
                        OcrCommand::Recognize { handle, reply } => {
                            let result = recognize(&images, &model_dir, &mut engine, handle);
                            worker_ready.store(engine.is_some(), Ordering::Release);
                            let _ = reply.send(result);
                        }
                        OcrCommand::Reset => {
                            engine = None;
                            worker_ready.store(false, Ordering::Release);
                        }
                    }
                }
            })
            .map_err(|error| format!("failed to start OCR worker: {error}"))?;
        Ok(Self { sender, ready })
    }

    async fn recognize(&self, handle: ImageResourceHandle) -> Result<Vec<OcrRegion>, String> {
        let (reply, result) = tokio::sync::oneshot::channel();
        self.sender
            .send(OcrCommand::Recognize { handle, reply })
            .map_err(|_| "OCR worker has stopped".to_owned())?;
        result
            .await
            .map_err(|_| "OCR worker stopped before replying".to_owned())?
    }

    fn reset(&self) {
        self.ready.store(false, Ordering::Release);
        let _ = self.sender.send(OcrCommand::Reset);
    }

    fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }
}

impl ReaderService {
    pub fn new(images: ImageResourceStore) -> Result<Self, String> {
        let project = ProjectDirs::from("dev", "Wabou", "Manga OCR")
            .ok_or_else(|| "could not resolve application directories".to_owned())?;
        let model_dir = project.data_dir().join("models").join(MODEL_VERSION);
        let ocr = OcrWorker::spawn(model_dir.clone(), images.clone())?;
        let llm = LlmWorker::spawn(project.cache_dir().to_owned(), images.clone())?;
        Ok(Self(Arc::new(ReaderState {
            model_dir,
            ocr,
            llm,
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
            ready: installed && self.0.ocr.ready(),
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
        self.0.ocr.reset();
        Ok(self.model_status())
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
        async move { service.0.ocr.recognize(request.handle).await }
    })?;
    let translate = service.clone();
    capability.method(TRANSLATE, move |request: TranslateRequest| {
        let service = translate.clone();
        async move { service.0.llm.translate(request).await }
    })?;
    let adjust = service.clone();
    capability.method(ADJUST_BBOXES, move |request: AdjustBboxesRequest| {
        let service = adjust.clone();
        async move { service.0.llm.adjust_bboxes(request).await }
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdjustBboxesRequest {
    handle: ImageResourceHandle,
    regions: Vec<OcrRegion>,
    api_key: String,
    model: String,
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

#[derive(Clone, Deserialize, Serialize)]
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

fn recognize(
    images: &ImageResourceStore,
    model_dir: &Path,
    engine: &mut Option<OAROCR>,
    handle: ImageResourceHandle,
) -> Result<Vec<OcrRegion>, String> {
    let image = images
        .get(handle)
        .ok_or_else(|| "image resource is missing or stale".to_owned())?
        .to_rgb8();
    let (width, height) = image.dimensions();
    if engine.is_none() {
        *engine = Some(
            OAROCRBuilder::new(
                model_dir.join(MODEL_FILES[0].name),
                model_dir.join(MODEL_FILES[1].name),
                model_dir.join(MODEL_FILES[2].name),
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

async fn translate(
    http: &reqwest::Client,
    cache: &ResultCache,
    request: TranslateRequest,
) -> Result<Vec<String>, String> {
    if request.api_key.trim().is_empty() {
        return Err("OpenRouter API key is required".to_owned());
    }
    if request.texts.is_empty() {
        return Ok(Vec::new());
    }
    let cache_key = json_cache_key(&serde_json::json!({
        "version": 1,
        "model": request.model,
        "targetLanguage": request.target_language,
        "texts": request.texts,
    }))?;
    if let Some(value) = cache.get("translation", &cache_key) {
        return Ok(value);
    }
    let numbered = request
        .texts
        .iter()
        .enumerate()
        .map(|(index, text)| format!("{}. {}", index + 1, text))
        .collect::<Vec<_>>()
        .join("\n");
    let content = openrouter_content(
        http,
        &request.api_key,
        &request.model,
        serde_json::json!([
            {"role": "system", "content": format!("Translate Japanese manga text to {}. Return only a JSON object shaped as {{\"translations\":[\"...\"]}}, preserving the original order and count.", request.target_language)},
            {"role": "user", "content": numbered}
        ]),
    )
    .await?;
    let translations = parse_translation_array(&content, request.texts.len())?;
    cache.insert("translation", &cache_key, &translations)?;
    Ok(translations)
}

async fn adjust_bboxes(
    http: &reqwest::Client,
    cache: &ResultCache,
    images: &ImageResourceStore,
    request: AdjustBboxesRequest,
) -> Result<Vec<OcrRegion>, String> {
    if request.api_key.trim().is_empty() {
        return Err("OpenRouter API key is required".to_owned());
    }
    if request.regions.is_empty() {
        return Ok(Vec::new());
    }
    let image = images
        .get(request.handle)
        .ok_or_else(|| "image resource is missing or stale".to_owned())?
        .to_rgb8();
    let (width, height) = image.dimensions();
    let cache_key = binary_cache_key(&[
        request.model.as_bytes(),
        serde_json::to_vec(&request.regions)
            .map_err(|error| error.to_string())?
            .as_slice(),
        image.as_raw(),
    ]);
    if let Some(value) = cache.get("bbox", &cache_key) {
        return Ok(value);
    }

    let preview = image::imageops::thumbnail(&image, 1600, 1600);
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 82)
        .write_image(
            preview.as_raw(),
            preview.width(),
            preview.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|error| format!("failed to encode LLM image preview: {error}"))?;
    let region_json = serde_json::to_string(&request.regions).map_err(|error| error.to_string())?;
    let prompt = format!(
        "Review these OCR bounding boxes against the manga page. Coordinates use the original {width}x{height} image. Tighten boxes around the complete visible text, preserve every id, and do not invent regions. Return only {{\"regions\":[{{\"id\":string,\"x\":number,\"y\":number,\"width\":number,\"height\":number}}]}}. Current regions: {region_json}"
    );
    let data_url = format!("data:image/jpeg;base64,{}", BASE64.encode(jpeg));
    let content = openrouter_content(
        http,
        &request.api_key,
        &request.model,
        serde_json::json!([{
            "role": "user",
            "content": [
                {"type": "text", "text": prompt},
                {"type": "image_url", "image_url": {"url": data_url}}
            ]
        }]),
    )
    .await?;
    let geometry = parse_bbox_array(&content)?;
    let by_id = geometry
        .into_iter()
        .map(|region| (region.id.clone(), region))
        .collect::<std::collections::HashMap<_, _>>();
    let adjusted = request
        .regions
        .into_iter()
        .map(|mut region| {
            if let Some(next) = by_id.get(&region.id) {
                region.x = next.x.clamp(0.0, width as f32);
                region.y = next.y.clamp(0.0, height as f32);
                region.width = next.width.clamp(1.0, width as f32 - region.x);
                region.height = next.height.clamp(1.0, height as f32 - region.y);
            }
            region
        })
        .collect::<Vec<_>>();
    cache.insert("bbox", &cache_key, &adjusted)?;
    Ok(adjusted)
}

async fn openrouter_content(
    http: &reqwest::Client,
    api_key: &str,
    model: &str,
    messages: serde_json::Value,
) -> Result<String, String> {
    let response = http
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": model,
            "messages": messages,
            "response_format": {"type": "json_object"}
        }))
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| format!("OpenRouter request failed: {error}"))?
        .json::<serde_json::Value>()
        .await
        .map_err(|error| error.to_string())?;
    response["choices"][0]["message"]["content"]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| "OpenRouter returned no message".to_owned())
}

fn json_cache_key(value: &serde_json::Value) -> Result<String, String> {
    Ok(sha256(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}

fn binary_cache_key(parts: &[&[u8]]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_le_bytes());
        digest.update(part);
    }
    format!("{:x}", digest.finalize())
}

#[derive(Deserialize)]
struct BboxGeometry {
    id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn parse_bbox_array(content: &str) -> Result<Vec<BboxGeometry>, String> {
    let value: serde_json::Value = serde_json::from_str(content)
        .map_err(|error| format!("bbox response was not JSON: {error}"))?;
    serde_json::from_value(value.get("regions").cloned().unwrap_or(value))
        .map_err(|error| format!("bbox response did not contain valid regions: {error}"))
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

    #[test]
    fn bbox_parser_requires_typed_geometry() {
        let regions = parse_bbox_array(
            r#"{"regions":[{"id":"line-1","x":10,"y":20,"width":30,"height":40}]}"#,
        )
        .unwrap();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].id, "line-1");
        assert_eq!(regions[0].width, 30.0);
        assert!(parse_bbox_array(r#"{"regions":[{"id":"missing"}]}"#).is_err());
    }

    #[test]
    fn binary_cache_keys_preserve_part_boundaries() {
        assert_ne!(
            binary_cache_key(&[b"ab", b"c"]),
            binary_cache_key(&[b"a", b"bc"])
        );
        assert_eq!(
            binary_cache_key(&[b"image", b"regions"]),
            binary_cache_key(&[b"image", b"regions"])
        );
    }

    #[test]
    fn result_cache_round_trips_json_atomically() {
        let directory = std::env::temp_dir().join(format!(
            "wabou-manga-cache-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let cache = ResultCache::new(directory.clone()).unwrap();
        cache
            .insert("translation", "key", &vec!["译文".to_owned()])
            .unwrap();
        assert_eq!(
            cache.get::<Vec<String>>("translation", "key").unwrap(),
            ["译文"]
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
