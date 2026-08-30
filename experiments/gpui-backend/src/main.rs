use gpui::{
    App, Bounds, Context, FontWeight, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};

/// Deliberately backend-neutral data, standing in for the flattened frame that
/// Wabou currently gives to its AnyRender scene builder.
#[derive(Clone)]
struct DemoFrame {
    project: &'static str,
    title: &'static str,
    body: &'static str,
    activity: &'static str,
}

struct GpuiFrameView {
    frame: DemoFrame,
    action_count: usize,
}

impl Render for GpuiFrameView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let frame = self.frame.clone();
        let action_count = self.action_count;

        div()
            .flex()
            .size_full()
            .bg(rgb(0xf7f7f8))
            .text_color(rgb(0x202124))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(250.0))
                    .h_full()
                    .px_4()
                    .py_4()
                    .gap_3()
                    .bg(rgb(0xf0f1f2))
                    .border_r_1()
                    .border_color(rgb(0xdcdde0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Wabou · GPUI"),
                    )
                    .child(
                        div()
                            .mt_3()
                            .text_xs()
                            .text_color(rgb(0x74777d))
                            .child("PROJECTS"),
                    )
                    .child(
                        div()
                            .id("project")
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_3()
                            .rounded_lg()
                            .bg(rgb(0xe1e4e8))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.action_count += 1;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(frame.project),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x74777d))
                                    .child("Retained frame adapter"),
                            ),
                    )
                    .child(
                        div()
                            .mt_auto()
                            .text_xs()
                            .text_color(rgb(0x74777d))
                            .child(format!("Input routed by GPUI · {action_count}")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .flex_1()
                    .h_full()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .h(px(52.0))
                            .px_5()
                            .border_b_1()
                            .border_color(rgb(0xe1e2e4))
                            .font_weight(FontWeight::MEDIUM)
                            .child(frame.title),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .p_8()
                            .gap_5()
                            .child(
                                div()
                                    .max_w(px(760.0))
                                    .text_lg()
                                    .line_height(gpui::relative(1.55))
                                    .child(frame.body),
                            )
                            .child(
                                div()
                                    .max_w(px(760.0))
                                    .p_4()
                                    .rounded_xl()
                                    .border_1()
                                    .border_color(rgb(0xdcdde0))
                                    .bg(rgb(0xffffff))
                                    .shadow_sm()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x74777d))
                                            .child("FRAME CONTRACT"),
                                    )
                                    .child(
                                        div()
                                            .mt_2()
                                            .text_sm()
                                            .child(frame.activity),
                                    ),
                            )
                            .child(
                                div()
                                    .mt_auto()
                                    .max_w(px(760.0))
                                    .w_full()
                                    .h(px(104.0))
                                    .p_4()
                                    .rounded_xl()
                                    .border_1()
                                    .border_color(rgb(0xc9cbd0))
                                    .bg(rgb(0xffffff))
                                    .shadow_sm()
                                    .text_color(rgb(0x74777d))
                                    .child("QuickJS + Solid frame input will be projected here"),
                            ),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(720.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("Wabou GPUI backend experiment");
                cx.new(|_| GpuiFrameView {
                    frame: DemoFrame {
                        project: "GPUI backend",
                        title: "Parallel renderer experiment",
                        body: "Keep Wabou's Solid 2 runtime, typed protocol and explicit resource model; replace the platform, text and paint boundary incrementally.",
                        activity: "GPUI owns the window, text shaping, paint and input for this frame. The existing Winit + AnyRender path remains untouched.",
                    },
                    action_count: 0,
                })
            },
        )
        .expect("open GPUI experiment window");
        cx.activate(true);
    });
}

