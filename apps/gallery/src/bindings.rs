//! Shared binding declarations for the gallery.
pub use gallery_api::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_method_and_manifest_share_names() {
        let output = manifest().render();
        assert!(output.contains(&format!("readonly {}", CAPABILITY.name())));
        assert!(output.contains(&format!("{}(request: string)", DESCRIBE_PALETTE.name())));
        assert!(output.contains(
            "describePalette(request: DescribePaletteRequest): Promise<DescribePaletteResponse>"
        ));
    }

    #[test]
    fn generated_host_bindings_are_current() {
        manifest()
            .check(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/ui/generated/host-bindings.ts"
            ))
            .expect("run `bun run gen` to refresh Gallery host bindings");
    }
}
