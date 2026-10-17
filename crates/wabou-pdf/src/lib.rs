//! Pure-Rust PDF page rasterization for Wabou.

#![warn(missing_docs)]

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::{LoadPdfError, Pdf};
use hayro::vello_cpu::{
    color::{AlphaColor, Srgb},
    peniko::ImageAlphaType,
};
use hayro::{PixmapSettings, RenderCache, RenderSettings};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
#[cfg(feature = "bindings")]
use wabou_bindgen::specta;
use wabou_renderer::{
    NativeWidgetPlugin, PaintContext, SemanticRole, WakeCallback, Widget, WidgetAccessibility,
    WidgetChanges, WidgetRasterImage, WidgetRasterImageError, decode_widget_config,
};

/// Stable native tag shared by Rust registration and generated TSX bindings.
pub const PDF_VIEW_TAG: &str = "pdf-view";

/// A parsed PDF document whose source bytes remain owned by the parser.
pub struct PdfDocument {
    pdf: Pdf,
}

/// Options controlling page rasterization.
#[derive(Clone, Copy, Debug)]
pub struct PageRenderOptions {
    /// Physical pixels generated for each PDF point.
    pub scale: f32,
    /// Straight-alpha sRGB background placed behind the PDF page.
    pub background: [u8; 4],
}

impl Default for PageRenderOptions {
    fn default() -> Self {
        Self {
            scale: 1.0,
            background: [255, 255, 255, 255],
        }
    }
}

/// An unpremultiplied RGBA8 page ready to retain as a Wabou image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedPage {
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
}

impl RenderedPage {
    /// Width of the rendered page in physical pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height of the rendered page in physical pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Unpremultiplied, tightly packed RGBA8 pixels.
    pub fn rgba8(&self) -> &[u8] {
        &self.rgba8
    }

    /// Convert this page into Wabou's retained raster-image representation.
    pub fn into_widget_image(self) -> Result<WidgetRasterImage, PdfRenderError> {
        WidgetRasterImage::from_rgba8(self.width, self.height, self.rgba8)
            .map_err(PdfRenderError::Image)
    }
}

/// Failure while loading or rasterizing a PDF page.
#[derive(Debug)]
pub enum PdfRenderError {
    /// The PDF data could not be parsed.
    Load(LoadPdfError),
    /// The requested page does not exist.
    PageOutOfBounds {
        /// Zero-based requested page index.
        index: usize,
        /// Number of pages in the document.
        page_count: usize,
    },
    /// The requested scale is zero, negative, non-finite, or exceeds hayro's
    /// current 16-bit raster dimensions.
    InvalidScale(f32),
    /// Wabou rejected the generated image buffer.
    Image(WidgetRasterImageError),
}

impl std::fmt::Display for PdfRenderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "could not load PDF: {error:?}"),
            Self::PageOutOfBounds { index, page_count } => {
                write!(formatter, "PDF page {index} is outside 0..{page_count}")
            }
            Self::InvalidScale(scale) => write!(formatter, "invalid PDF render scale {scale}"),
            Self::Image(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for PdfRenderError {}

impl PdfDocument {
    /// Parse an owned PDF byte buffer.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, PdfRenderError> {
        Pdf::new(bytes)
            .map(|pdf| Self { pdf })
            .map_err(PdfRenderError::Load)
    }

    /// Number of pages exposed by the document.
    pub fn page_count(&self) -> usize {
        self.pdf.pages().len()
    }

    /// Rasterize one page with hayro's pure-Rust PDF interpreter and Vello CPU.
    pub fn render_page(
        &self,
        index: usize,
        options: PageRenderOptions,
    ) -> Result<RenderedPage, PdfRenderError> {
        if !options.scale.is_finite() || options.scale <= 0.0 {
            return Err(PdfRenderError::InvalidScale(options.scale));
        }

        let pages = self.pdf.pages();
        let page = pages.get(index).ok_or(PdfRenderError::PageOutOfBounds {
            index,
            page_count: pages.len(),
        })?;
        let (page_width, page_height) = page.render_dimensions();
        let scaled_width = page_width * options.scale;
        let scaled_height = page_height * options.scale;
        if scaled_width < 1.0
            || scaled_height < 1.0
            || scaled_width > f32::from(u16::MAX)
            || scaled_height > f32::from(u16::MAX)
        {
            return Err(PdfRenderError::InvalidScale(options.scale));
        }

        let [red, green, blue, alpha] = options.background;
        let pixmap = hayro::render(
            page,
            &RenderCache::new(),
            &InterpreterSettings::default(),
            &RenderSettings::default(),
            &PixmapSettings {
                x_scale: options.scale,
                y_scale: options.scale,
                bg_color: AlphaColor::<Srgb>::from_rgba8(red, green, blue, alpha),
            },
        );
        let width = u32::from(pixmap.width());
        let height = u32::from(pixmap.height());
        Ok(RenderedPage {
            width,
            height,
            rgba8: pixmap.take_rgba8(ImageAlphaType::Alpha),
        })
    }
}

/// Construct the native widget registered by the `@wabou/pdf` Solid wrapper.
///
/// Applications normally install it with
/// `HostBuilder::widget("pdf-view", wabou_pdf::pdf_viewer_widget)`.
pub fn pdf_viewer_widget() -> Box<dyn Widget> {
    Box::new(PdfViewerWidget::default())
}

/// Return the statically linked PDF widget registration for `HostBuilder`.
pub const fn plugin() -> NativeWidgetPlugin {
    NativeWidgetPlugin::new(PDF_VIEW_TAG, pdf_viewer_widget)
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(wabou_bindgen::Type))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PdfViewConfig {
    /// Native filesystem path to the PDF document.
    path: PathBuf,
    /// One-based page number.
    #[serde(default = "first_page")]
    page: u32,
    /// Raster pixels generated for each PDF point.
    #[serde(default = "default_render_scale")]
    #[cfg_attr(
        feature = "bindings",
        specta(type = wabou_bindgen::specta_typescript::Number)
    )]
    render_scale: f32,
    /// Straight-alpha RGBA8 background behind transparent page content.
    #[serde(default = "white")]
    background: [u8; 4],
}

/// Build the Rust-owned TSX binding contract for this widget crate.
#[cfg(feature = "bindings")]
pub fn bindings() -> wabou_bindgen::WidgetModule {
    wabou_bindgen::WidgetModule::new().widget::<PdfViewConfig>("PdfView", PDF_VIEW_TAG)
}

const fn first_page() -> u32 {
    1
}

const fn default_render_scale() -> f32 {
    2.0
}

const fn white() -> [u8; 4] {
    [255, 255, 255, 255]
}

struct LoadedPage {
    image: WidgetRasterImage,
    logical_size: [f32; 2],
    page_count: usize,
}

struct RenderCompletion {
    generation: u64,
    result: Result<LoadedPage, String>,
}

struct RenderRequest {
    generation: u64,
    config: PdfViewConfig,
}

/// A retained PDF page whose file parsing and rasterization happen off the UI
/// thread. The previous page remains visible while a replacement is loading.
pub struct PdfViewerWidget {
    config: Option<PdfViewConfig>,
    generation: u64,
    page: Option<LoadedPage>,
    error: Option<String>,
    request_sender: mpsc::Sender<RenderRequest>,
    completion_receiver: mpsc::Receiver<RenderCompletion>,
    wake: Arc<Mutex<Option<WakeCallback>>>,
}

impl Default for PdfViewerWidget {
    fn default() -> Self {
        let (request_sender, request_receiver) = mpsc::channel::<RenderRequest>();
        let (completion_sender, completion_receiver) = mpsc::channel::<RenderCompletion>();
        let wake: Arc<Mutex<Option<WakeCallback>>> = Arc::new(Mutex::new(None));
        let worker_wake = Arc::clone(&wake);
        std::thread::Builder::new()
            .name("wabou-pdf-render".into())
            .spawn(move || {
                while let Ok(mut request) = request_receiver.recv() {
                    while let Ok(newer) = request_receiver.try_recv() {
                        request = newer;
                    }
                    let result = render_file(&request.config);
                    if completion_sender
                        .send(RenderCompletion {
                            generation: request.generation,
                            result,
                        })
                        .is_err()
                    {
                        break;
                    }
                    let wake = worker_wake.lock().expect("PDF wake lock poisoned").clone();
                    if let Some(wake) = wake {
                        wake();
                    }
                }
            })
            .expect("could not start PDF render worker");
        Self {
            config: None,
            generation: 0,
            page: None,
            error: None,
            request_sender,
            completion_receiver,
            wake,
        }
    }
}

impl PdfViewerWidget {
    fn start_render(&mut self, config: PdfViewConfig) -> Result<(), String> {
        if config.path.as_os_str().is_empty() {
            return Err("PDF path must not be empty".into());
        }
        if config.page == 0 {
            return Err("PDF page numbers start at 1".into());
        }
        if !config.render_scale.is_finite() || config.render_scale <= 0.0 {
            return Err("PDF renderScale must be finite and greater than zero".into());
        }

        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.config = Some(config.clone());
        self.error = None;
        self.request_sender
            .send(RenderRequest { generation, config })
            .map_err(|_| "PDF render worker stopped unexpectedly".to_string())?;
        Ok(())
    }

    fn apply_completion(&mut self, completion: RenderCompletion) -> WidgetChanges {
        if completion.generation != self.generation {
            return WidgetChanges::empty();
        }
        match completion.result {
            Ok(page) => {
                self.page = Some(page);
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error);
            }
        }
        WidgetChanges::REDRAW
            | WidgetChanges::MEASURE
            | WidgetChanges::LAYOUT
            | WidgetChanges::SEMANTICS
    }
}

fn render_file(config: &PdfViewConfig) -> Result<LoadedPage, String> {
    let bytes = std::fs::read(&config.path)
        .map_err(|error| format!("could not read {}: {error}", config.path.display()))?;
    let document = PdfDocument::from_bytes(bytes).map_err(|error| error.to_string())?;
    let page_count = document.page_count();
    let rendered = document
        .render_page(
            (config.page - 1) as usize,
            PageRenderOptions {
                scale: config.render_scale,
                background: config.background,
            },
        )
        .map_err(|error| error.to_string())?;
    let logical_size = [
        rendered.width() as f32 / config.render_scale,
        rendered.height() as f32 / config.render_scale,
    ];
    let image = rendered
        .into_widget_image()
        .map_err(|error| error.to_string())?;
    Ok(LoadedPage {
        image,
        logical_size,
        page_count,
    })
}

impl Widget for PdfViewerWidget {
    fn paint(&mut self, context: &mut PaintContext<'_>) {
        let Some(page) = &self.page else {
            return;
        };
        let [width, height] = context.size();
        let [page_width, page_height] = page.logical_size;
        if width <= 0.0 || height <= 0.0 || page_width <= 0.0 || page_height <= 0.0 {
            return;
        }
        let scale = (width / page_width).min(height / page_height);
        let drawn_width = page_width * scale;
        let drawn_height = page_height * scale;
        context.draw_raster_image_in(
            &page.image,
            [
                f64::from((width - drawn_width) * 0.5),
                f64::from((height - drawn_height) * 0.5),
                f64::from(drawn_width),
                f64::from(drawn_height),
            ],
        );
    }

    fn config_changed(&mut self, json: &str) -> Result<WidgetChanges, String> {
        let config: PdfViewConfig = decode_widget_config(json)?;
        if self.config.as_ref() == Some(&config) {
            return Ok(WidgetChanges::empty());
        }
        self.start_render(config)?;
        Ok(WidgetChanges::SEMANTICS)
    }

    fn config_removed(&mut self) -> WidgetChanges {
        self.generation = self.generation.wrapping_add(1);
        self.config = None;
        self.page = None;
        self.error = None;
        WidgetChanges::REDRAW
            | WidgetChanges::MEASURE
            | WidgetChanges::LAYOUT
            | WidgetChanges::SEMANTICS
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        self.page.as_ref().map(|page| page.logical_size)
    }

    fn accessibility(&self) -> WidgetAccessibility {
        let label = if let Some(error) = &self.error {
            format!("PDF failed to load: {error}")
        } else if let (Some(config), Some(page)) = (&self.config, &self.page) {
            format!("PDF page {} of {}", config.page, page.page_count)
        } else {
            "PDF page loading".into()
        };
        WidgetAccessibility {
            role: Some(SemanticRole::Image),
            label: Some(label),
            ..Default::default()
        }
    }

    fn set_wake_callback(&mut self, wake: WakeCallback) {
        *self.wake.lock().expect("PDF wake lock poisoned") = Some(wake);
    }

    fn poll_async_changes(&mut self) -> WidgetChanges {
        let mut changes = WidgetChanges::empty();
        while let Ok(completion) = self.completion_receiver.try_recv() {
            changes |= self.apply_completion(completion);
        }
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    fn one_page_pdf() -> Vec<u8> {
        let stream = "0.92 0.16 0.22 rg 20 20 160 60 re f";
        let mut objects = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Contents 4 0 R >>".to_string(),
            format!(
                "<< /Length {} >>\nstream\n{stream}\nendstream",
                stream.len()
            ),
        ];
        let mut bytes = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (index, object) in objects.drain(..).enumerate() {
            offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
        }
        let xref = bytes.len();
        bytes.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
        for offset in offsets {
            bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        bytes.extend_from_slice(
            format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
        );
        bytes
    }

    #[test]
    fn renders_a_pdf_page_into_wabou_rgba() {
        let document = PdfDocument::from_bytes(one_page_pdf()).unwrap();
        assert_eq!(document.page_count(), 1);

        let page = document
            .render_page(0, PageRenderOptions::default())
            .unwrap();
        assert_eq!([page.width(), page.height()], [200, 100]);
        assert_eq!(page.rgba8().len(), 200 * 100 * 4);
        let center = &page.rgba8()[(50 * 200 + 50) * 4..][..4];
        assert!(center[0] > 200 && center[1] < 80 && center[2] < 100);
        assert_eq!(center[3], 255);
        assert_eq!(&page.rgba8()[..4], &[255, 255, 255, 255]);
        assert_eq!(page.into_widget_image().unwrap().dimensions(), [200, 100]);
    }

    #[test]
    fn rejects_invalid_page_and_scale() {
        let document = PdfDocument::from_bytes(one_page_pdf()).unwrap();
        assert!(matches!(
            document.render_page(1, PageRenderOptions::default()),
            Err(PdfRenderError::PageOutOfBounds { .. })
        ));
        assert!(matches!(
            document.render_page(
                0,
                PageRenderOptions {
                    scale: 0.0,
                    ..PageRenderOptions::default()
                }
            ),
            Err(PdfRenderError::InvalidScale(0.0))
        ));
    }

    #[test]
    fn native_widget_renders_files_off_thread_and_wakes_the_host() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&one_page_pdf()).unwrap();
        let config = serde_json::json!({
            "path": file.path(),
            "page": 1,
            "renderScale": 2,
            "background": [255, 255, 255, 255],
        });
        let woke = Arc::new(AtomicBool::new(false));
        let mut widget = PdfViewerWidget::default();
        let worker_woke = Arc::clone(&woke);
        widget.set_wake_callback(Arc::new(move || {
            worker_woke.store(true, Ordering::Release);
        }));

        assert_eq!(
            widget.config_changed(&config.to_string()).unwrap(),
            WidgetChanges::SEMANTICS
        );
        let completion = widget
            .completion_receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("background PDF render did not finish");
        let changes = widget.apply_completion(completion);

        assert!(woke.load(Ordering::Acquire));
        assert!(changes.contains(WidgetChanges::REDRAW));
        assert!(changes.contains(WidgetChanges::MEASURE));
        assert_eq!(widget.intrinsic_size(), Some([200.0, 100.0]));
        assert_eq!(
            widget.accessibility().label.as_deref(),
            Some("PDF page 1 of 1")
        );
    }

    #[test]
    fn plugin_owns_the_tag_and_factory_registration() {
        let plugin = plugin();
        assert_eq!(plugin.tag(), PDF_VIEW_TAG);
        assert_eq!(
            plugin.create().accessibility().role,
            Some(SemanticRole::Image)
        );
    }
}
