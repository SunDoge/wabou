export default {
  build: {
    lib: {
      entry: "entry.ts",
      formats: ["iife"],
      name: "WabouParaglideExperiment",
      fileName: () => "bundle.js",
    },
    minify: "esbuild",
    outDir: "dist",
    emptyOutDir: true,
  },
};
