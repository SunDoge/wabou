fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&wabou_style::manifest()).expect("serialize utility manifest")
    );
}
