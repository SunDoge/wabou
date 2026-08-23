use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use directories::ProjectDirs;
use image::ImageEncoder as _;
use oar_ocr::prelude::{OAROCR, OAROCRBuilder, TextRegion};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wabou::{
    ImageResourceHandle, ImageResourceStore, JsonCapability, JsonCapabilityContract, JsonMethod,
    PersistentJsonCache, SerialWorker, rquickjs,
};

pub const CAPABILITY: JsonCapabilityContract = JsonCapabilityContract::new("mangaReader", 2);
const LIST_IMAGES: JsonMethod<ListImagesRequest, Vec<ImagePage>> = JsonMethod::new("listImages");
const DESCRIBE_IMAGES: JsonMethod<DescribeImagesRequest, Vec<ImagePage>> =
    JsonMethod::new("describeImages");
const MODEL_STATUS: JsonMethod<(), OcrModelStatus> = JsonMethod::no_request("modelStatus");
const MODEL_DOWNLOAD_PROGRESS: JsonMethod<(), ModelDownloadProgress> =
    JsonMethod::no_request("modelDownloadProgress");
const DOWNLOAD_MODEL: JsonMethod<(), OcrModelStatus> = JsonMethod::no_request("downloadModel");
const RECENT_ENTRIES: JsonMethod<(), Vec<RecentEntry>> = JsonMethod::no_request("recentEntries");
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
        sha256: "b5f2bfe2bdd9448429e3e82b51c789775d9b42f2403d082b00662eb77e401c5d",
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
    recent_path: PathBuf,
    recent: Mutex<Vec<RecentEntry>>,
    download_progress: Mutex<ModelDownloadProgress>,
    ocr: OcrWorker,
    llm: LlmWorker,
    http: reqwest::Client,
    images: ImageResourceStore,
}

struct LlmWorker {
    worker: SerialWorker<LlmCommand, LlmResponse>,
}

enum LlmCommand {
    Translate(TranslateRequest),
    AdjustBboxes(AdjustBboxesRequest),
}

enum LlmResponse {
    Translations(Vec<String>),
    Regions(Vec<OcrRegion>),
}

struct LlmState {
    runtime: tokio::runtime::Runtime,
    http: reqwest::Client,
    cache: PersistentJsonCache,
    images: ImageResourceStore,
}

impl LlmWorker {
    fn spawn(cache: PersistentJsonCache, images: ImageResourceStore) -> Result<Self, String> {
        let worker = SerialWorker::spawn(
            "manga-ocr-llm",
            move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| error.to_string())?;
                let http = reqwest::Client::builder()
                    .user_agent("MangaOcrWabou/0.1")
                    .connect_timeout(std::time::Duration::from_secs(10))
                    .timeout(std::time::Duration::from_secs(90))
                    .build()
                    .map_err(|error| error.to_string())?;
                Ok(LlmState {
                    runtime,
                    http,
                    cache,
                    images,
                })
            },
            |state, command| match command {
                LlmCommand::Translate(request) => state
                    .runtime
                    .block_on(translate(&state.http, &state.cache, request))
                    .map(LlmResponse::Translations),
                LlmCommand::AdjustBboxes(request) => state
                    .runtime
                    .block_on(adjust_bboxes(
                        &state.http,
                        &state.cache,
                        &state.images,
                        request,
                    ))
                    .map(LlmResponse::Regions),
            },
        )?;
        Ok(Self { worker })
    }

    async fn translate(&self, request: TranslateRequest) -> Result<Vec<String>, String> {
        match self.worker.request(LlmCommand::Translate(request)).await? {
            LlmResponse::Translations(value) => Ok(value),
            LlmResponse::Regions(_) => Err("LLM worker returned the wrong response".to_owned()),
        }
    }

    async fn adjust_bboxes(&self, request: AdjustBboxesRequest) -> Result<Vec<OcrRegion>, String> {
        match self
            .worker
            .request(LlmCommand::AdjustBboxes(request))
            .await?
        {
            LlmResponse::Regions(value) => Ok(value),
            LlmResponse::Translations(_) => {
                Err("LLM worker returned the wrong response".to_owned())
            }
        }
    }
}

struct OcrWorker {
    worker: SerialWorker<OcrCommand, Vec<OcrRegion>>,
    ready: Arc<AtomicBool>,
}

enum OcrCommand {
    Recognize(ImageResourceHandle),
    Reset,
}

struct OcrState {
    model_dir: PathBuf,
    images: ImageResourceStore,
    cache: PersistentJsonCache,
    engine: Option<OAROCR>,
    ready: Arc<AtomicBool>,
}

impl OcrWorker {
    fn spawn(
        model_dir: PathBuf,
        images: ImageResourceStore,
        cache: PersistentJsonCache,
    ) -> Result<Self, String> {
        let ready = Arc::new(AtomicBool::new(false));
        let worker_ready = ready.clone();
        let worker = SerialWorker::spawn(
            "manga-ocr-inference",
            move || {
                Ok(OcrState {
                    model_dir,
                    images,
                    cache,
                    engine: None,
                    ready: worker_ready,
                })
            },
            |state, command| match command {
                OcrCommand::Recognize(handle) => {
                    let result = recognize(
                        &state.images,
                        &state.model_dir,
                        &state.cache,
                        &mut state.engine,
                        handle,
                    );
                    state.ready.store(state.engine.is_some(), Ordering::Release);
                    result
                }
                OcrCommand::Reset => {
                    state.engine = None;
                    state.ready.store(false, Ordering::Release);
                    Ok(Vec::new())
                }
            },
        )?;
        Ok(Self { worker, ready })
    }

    async fn recognize(&self, handle: ImageResourceHandle) -> Result<Vec<OcrRegion>, String> {
        self.worker.request(OcrCommand::Recognize(handle)).await
    }

    async fn reset(&self) -> Result<(), String> {
        self.ready.store(false, Ordering::Release);
        self.worker.request(OcrCommand::Reset).await.map(|_| ())
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
        let recent_path = project.data_dir().join("recent.json");
        let recent = load_recent(&recent_path);
        let cache = PersistentJsonCache::new(project.cache_dir().join("pipeline-results-v1"), 512)?;
        let ocr = OcrWorker::spawn(model_dir.clone(), images.clone(), cache.clone())?;
        let llm = LlmWorker::spawn(cache, images.clone())?;
        Ok(Self(Arc::new(ReaderState {
            model_dir,
            recent_path,
            recent: Mutex::new(recent),
            download_progress: Mutex::new(ModelDownloadProgress::idle()),
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
        let total_bytes = MODEL_FILES.iter().map(|file| file.size).sum();
        let existing_bytes = MODEL_FILES
            .iter()
            .filter(|file| model_file_valid(&self.0.model_dir, **file))
            .map(|file| file.size)
            .sum();
        self.set_download_progress(ModelDownloadProgress {
            state: DownloadState::Downloading,
            downloaded_bytes: existing_bytes,
            total_bytes,
            current_file: None,
            error: None,
        });
        fs::create_dir_all(&self.0.model_dir).map_err(|error| error.to_string())?;
        let mut downloaded_bytes = existing_bytes;
        for file in MODEL_FILES {
            let target = self.0.model_dir.join(file.name);
            if model_file_valid(&self.0.model_dir, file) {
                continue;
            }
            self.update_download_progress(|progress| {
                progress.current_file = Some(file.name.to_owned());
            });
            let mut response = self
                .0
                .http
                .get(format!("{RELEASE_BASE}/{}", file.name))
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|error| format!("failed to download {}: {error}", file.name))?;
            let partial = target.with_extension("part");
            let mut output = fs::File::create(&partial).map_err(|error| error.to_string())?;
            let mut bytes = Vec::with_capacity(file.size as usize);
            while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
                output
                    .write_all(&chunk)
                    .map_err(|error| error.to_string())?;
                bytes.extend_from_slice(&chunk);
                downloaded_bytes += chunk.len() as u64;
                self.update_download_progress(|progress| {
                    progress.downloaded_bytes = downloaded_bytes.min(progress.total_bytes);
                });
            }
            output.sync_all().map_err(|error| error.to_string())?;
            if bytes.len() as u64 != file.size || sha256(&bytes) != file.sha256 {
                let _ = fs::remove_file(&partial);
                return Err(format!("model checksum failed: {}", file.name));
            }
            fs::rename(&partial, &target).map_err(|error| error.to_string())?;
        }
        self.update_download_progress(|progress| {
            progress.state = DownloadState::Verifying;
            progress.current_file = None;
        });
        self.0.ocr.reset().await?;
        self.set_download_progress(ModelDownloadProgress {
            state: DownloadState::Complete,
            downloaded_bytes: total_bytes,
            total_bytes,
            current_file: None,
            error: None,
        });
        Ok(self.model_status())
    }

    fn download_progress(&self) -> ModelDownloadProgress {
        self.0
            .download_progress
            .lock()
            .expect("download progress lock")
            .clone()
    }

    fn set_download_progress(&self, progress: ModelDownloadProgress) {
        *self
            .0
            .download_progress
            .lock()
            .expect("download progress lock") = progress;
    }

    fn update_download_progress(&self, update: impl FnOnce(&mut ModelDownloadProgress)) {
        update(
            &mut self
                .0
                .download_progress
                .lock()
                .expect("download progress lock"),
        );
    }

    fn recent_entries(&self) -> Vec<RecentEntry> {
        self.0.recent.lock().expect("recent entries lock").clone()
    }

    fn record_recent(&self, entry: RecentEntry) -> Result<(), String> {
        let mut recent = self.0.recent.lock().expect("recent entries lock");
        recent.retain(|current| current.path != entry.path || current.kind != entry.kind);
        recent.insert(0, entry);
        recent.truncate(20);
        write_json_atomic(&self.0.recent_path, &*recent)
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
        async move { service.describe_images(&request.paths) }
    })?;
    let status = service.clone();
    capability.method(MODEL_STATUS, move |(): ()| {
        let service = status.clone();
        async move { Ok::<_, String>(service.model_status()) }
    })?;
    let progress = service.clone();
    capability.method(MODEL_DOWNLOAD_PROGRESS, move |(): ()| {
        let service = progress.clone();
        async move { Ok::<_, String>(service.download_progress()) }
    })?;
    let recent = service.clone();
    capability.method(RECENT_ENTRIES, move |(): ()| {
        let service = recent.clone();
        async move { Ok::<_, String>(service.recent_entries()) }
    })?;
    let download = service.clone();
    capability.method(DOWNLOAD_MODEL, move |(): ()| {
        let service = download.clone();
        async move {
            let result = service.download_model().await;
            if let Err(error) = &result {
                service.update_download_progress(|progress| {
                    progress.state = DownloadState::Failed;
                    progress.current_file = None;
                    progress.error = Some(error.clone());
                });
            }
            result
        }
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct RecentEntry {
    kind: RecentKind,
    path: String,
    label: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum RecentKind {
    File,
    Directory,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelDownloadProgress {
    state: DownloadState,
    downloaded_bytes: u64,
    total_bytes: u64,
    current_file: Option<String>,
    error: Option<String>,
}

impl ModelDownloadProgress {
    fn idle() -> Self {
        Self {
            state: DownloadState::Idle,
            downloaded_bytes: 0,
            total_bytes: MODEL_FILES.iter().map(|file| file.size).sum(),
            current_file: None,
            error: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum DownloadState {
    Idle,
    Downloading,
    Verifying,
    Complete,
    Failed,
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
        let pages = paths
            .iter()
            .map(|path| self.describe_image(path))
            .collect::<Result<Vec<_>, _>>()?;
        let canonical = Path::new(directory)
            .canonicalize()
            .map_err(|error| error.to_string())?;
        self.record_recent(RecentEntry {
            kind: RecentKind::Directory,
            label: natural_name(&canonical),
            path: canonical.to_string_lossy().into_owned(),
        })?;
        Ok(pages)
    }

    fn describe_images(&self, paths: &[String]) -> Result<Vec<ImagePage>, String> {
        let pages = paths
            .iter()
            .map(|path| self.describe_image(Path::new(path)))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(page) = pages.first() {
            self.record_recent(RecentEntry {
                kind: RecentKind::File,
                label: if pages.len() == 1 {
                    page.name.clone()
                } else {
                    format!("{} and {} more", page.name, pages.len() - 1)
                },
                path: paths.join("\n"),
            })?;
        }
        Ok(pages)
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

fn load_recent(path: &Path) -> Vec<RecentEntry> {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn write_json_atomic(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let partial = path.with_extension("tmp");
    fs::write(
        &partial,
        serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::rename(partial, path).map_err(|error| error.to_string())
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
    cache: &PersistentJsonCache,
    engine: &mut Option<OAROCR>,
    handle: ImageResourceHandle,
) -> Result<Vec<OcrRegion>, String> {
    let image = images
        .get(handle)
        .ok_or_else(|| "image resource is missing or stale".to_owned())?
        .to_rgb8();
    let (width, height) = image.dimensions();
    let cache_key = PersistentJsonCache::content_key(&[MODEL_VERSION.as_bytes(), image.as_raw()]);
    if let Some(regions) = cache.get("ocr", &cache_key) {
        return Ok(regions);
    }
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
    cache.insert("ocr", &cache_key, &regions)?;
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
    cache: &PersistentJsonCache,
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
    cache: &PersistentJsonCache,
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
    let cache_key = PersistentJsonCache::content_key(&[
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
    fn recent_entries_round_trip_through_atomic_json() {
        let path = std::env::temp_dir().join(format!(
            "wabou-manga-recent-{}-{}.json",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let entries = vec![RecentEntry {
            kind: RecentKind::Directory,
            path: "/tmp/chapter".to_owned(),
            label: "chapter".to_owned(),
        }];
        write_json_atomic(&path, &entries).unwrap();
        assert_eq!(load_recent(&path), entries);
        let _ = fs::remove_file(path);
    }
}
