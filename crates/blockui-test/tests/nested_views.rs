use blockui::*;
use blockui_test::Screen;

struct Nested;
impl Actor for Nested {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    #[blockui::view]
    fn view(_: &Cx<'_>, outer_ui: &Ui) -> Result<View> {
        #[blockui::view]
        fn inner(inner_ui: &Ui) -> View {
            #[node]
            Text::new("inner").key("inner.label");
        }
        #[node]
        {
            Column::default();
            #[node]
            inner();
            #[node]
            Text::new("outer").key("outer.label");
        }
    }
}

#[test]
fn nested_view_builders_mount_their_own_nodes() -> std::result::Result<(), String> {
    let screen = Screen::render::<Nested>()?;
    let inner = screen.get_by_text("inner")?;
    let outer = screen.get_by_text("outer")?;
    assert_ne!(inner, outer);
    assert_eq!(screen.root_evaluations(), 1);
    Ok(())
}
