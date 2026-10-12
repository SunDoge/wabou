import { defineConfig, rustPlugin } from "@wabou/vite/config";
export default defineConfig({
  window: { title: "Rust plugin hot reload", width: 520, height: 420 },
  plugins: [
    rustPlugin({
      name: "app",
      manifest: "./native/Cargo.toml",
      targetDirectory: "./target/native",
      capabilities: [
        {
          name: "counter",
          version: 1,
          methods: [
            { name: "increment", hasRequest: false, json: false },
            { name: "read", hasRequest: false, json: false },
          ],
        },
      ],
      widgets: [{ tag: "reload-counter", version: 1, initialState: 40 }],
    }),
  ],
});
