use gpui::{
    App, AppContext, Bounds, Context, Render, Window, WindowBounds, WindowOptions, hsla, px, size,
};
use wabou_shell::{NodeKey, ProjectionTree, application};

struct Example {
    tree: ProjectionTree,
    root: NodeKey,
}

impl Render for Example {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.tree
            .element(self.root)
            .expect("example projection remains structurally valid")
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let root = NodeKey::new(1, 1);
        let child = NodeKey::new(2, 1);
        let mut tree = ProjectionTree::default();

        let mut root_style = gpui::Style::default();
        root_style.size.width = gpui::relative(1.0).into();
        root_style.size.height = gpui::relative(1.0).into();
        root_style.padding.top = px(24.0).into();
        root_style.padding.right = px(24.0).into();
        root_style.padding.bottom = px(24.0).into();
        root_style.padding.left = px(24.0).into();
        root_style.background = Some(hsla(220.0 / 360.0, 0.25, 0.96, 1.0).into());
        tree.insert(root, None, 0, root_style, None).unwrap();

        let mut child_style = gpui::Style::default();
        child_style.padding.top = px(20.0).into();
        child_style.padding.right = px(20.0).into();
        child_style.padding.bottom = px(20.0).into();
        child_style.padding.left = px(20.0).into();
        child_style.background = Some(hsla(0.0, 0.0, 1.0, 1.0).into());
        child_style.corner_radii.top_left = px(14.0).into();
        child_style.corner_radii.top_right = px(14.0).into();
        child_style.corner_radii.bottom_right = px(14.0).into();
        child_style.corner_radii.bottom_left = px(14.0).into();
        tree.insert(
            child,
            Some(root),
            0,
            child_style,
            Some("Solid retained frame rendered by GPUI-CE".into()),
        )
        .unwrap();
        let _ = tree.commit();

        let bounds = Bounds::centered(None, size(px(760.0), px(480.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|_| Example { tree, root }),
        )
        .expect("open Wabou GPUI window");
        cx.activate(true);
    });
}
