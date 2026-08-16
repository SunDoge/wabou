//! Print Wabou's versioned style capability manifest as JSON.

fn main() {
    let mut theme = wabou_style::Theme::default();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        if argument != "--theme" {
            panic!("unknown argument `{argument}`; expected `--theme <path>`");
        }
        let path = args.next().expect("--theme requires a JSON file path");
        let patch: wabou_style::Theme =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read Wabou theme file"))
                .expect("parse Wabou theme JSON");
        theme.spacing.extend(patch.spacing);
        theme.colors.extend(patch.colors);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&wabou_style::manifest_with_theme(&theme))
            .expect("serialize utility manifest")
    );
}
