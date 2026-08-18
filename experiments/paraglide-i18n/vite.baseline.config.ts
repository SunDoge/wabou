export default {
  build: {
    lib: {
      entry: "baseline.ts",
      formats: ["iife"],
      name: "WabouParaglideBaseline",
      fileName: () => "bundle.js",
    },
    minify: "esbuild",
    outDir: "dist-baseline",
    emptyOutDir: true,
  },
};
