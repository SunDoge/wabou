# WGPU + Vello background experiment

This isolated experiment proves that Wabou can render a custom WGSL background
and a transparent Vello UI with the same `wgpu::Device` and `wgpu::Queue`, then
compose both textures on the GPU without a CPU readback between layers.

Run it with:

```bash
cargo run --release --manifest-path experiments/wgpu-vello-background/Cargo.toml
```

The output is written to `experiments/wgpu-vello-background/out/composed.png`.
The fixed time uniform can be changed in `main.rs`; a windowed implementation
would update the same buffer once per frame.

This only validates a full-window GPU background below one Vello scene. It does
not provide arbitrary GPU widgets interleaved with Vello nodes.
