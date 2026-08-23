# Cross-compilation

Status: experimentally verified. `wabou package --target` integrates Cargo
Zigbuild with native packaging; ordinary `wabou build` remains host-native.

Wabou's Rust host can be cross-compiled with
[`cargo-zigbuild`](https://github.com/rust-cross/cargo-zigbuild). Zig replaces
the target C compiler and linker; it does not provide target desktop system
libraries, execute the result, create a native installer, or sign it.

The commands and results below were verified with Cargo Zigbuild 0.23.0 and Zig
0.16.0. They are build/link evidence only unless a target-platform run is also
recorded.

## Linux glibc baseline

For a same-architecture Linux release, the glibc suffix constrains the symbols
that may appear in the executable:

```bash
ulimit -n 8192
cargo zigbuild \
  -p gallery \
  --release \
  --target x86_64-unknown-linux-gnu.2.28
```

The resulting `gallery` ELF was successfully linked and its highest required
glibc symbol version was `GLIBC_2.28`:

```bash
readelf --version-info target/x86_64-unknown-linux-gnu/release/gallery \
  | grep -oE 'GLIBC_[0-9]+\.[0-9]+' \
  | sort -Vu
```

This does not make the application statically linked. The target system must
still provide compatible shared libraries such as fontconfig, FreeType,
expat, libpng, Brotli, and zlib. Test the executable on the oldest supported
distribution; inspecting the ELF is not evidence that rendering, fonts,
dialogs, clipboard, or input work there.

glibc 2.28 is a practical preview baseline. Lower suffixes such as 2.27 or
2.17 require their own build and target-system run; do not advertise them from
the suffix alone.

## Windows GNU from Linux

The complete Gallery host has also been linked on Linux for Windows GNU:

```bash
rustup target add x86_64-pc-windows-gnu
ulimit -n 8192
cargo zigbuild \
  -p gallery \
  --release \
  --target x86_64-pc-windows-gnu
```

The result was a 64-bit PE executable. Linux-only GTK, GLib, Wayland,
fontconfig, and AT-SPI dependencies were correctly excluded by target `cfg`.
The Wabou host, QuickJS, winit, wgpu, Vello, rfd, and their required native C
code all compiled and linked.

Wabou's final link currently contains more than 1,000 object files. A shell
limit of 1,024 file descriptors caused Zig to fail with
`ProcessFdQuotaExceeded`; raising the limit to 8,192 fixed the link. This is a
build-host limit, not a Windows dependency failure.

This GNU build is useful as a CI link check and preview artifact. Official
Windows releases should still be built and exercised on a Windows runner with
the MSVC target, native packaging tools, and signing. A successfully linked PE
does not prove Windows rendering, HiDPI, IME, dialogs, notifications, or
clipboard behavior.

## Linux cross-architecture builds

An `x86_64` Linux host building `aarch64-unknown-linux-gnu.2.28` progressed
through Rust, winit, wgpu, Vello, and QuickJS, then stopped at fontconfig.
Because the target is still Linux, Linux desktop backends are expected to
compile. Install an aarch64 sysroot and configure cross-aware `pkg-config`, for
example through these target-specific variables:

```bash
export PKG_CONFIG_ALLOW_CROSS_aarch64_unknown_linux_gnu=1
export PKG_CONFIG_SYSROOT_DIR_aarch64_unknown_linux_gnu=/path/to/aarch64/sysroot
export PKG_CONFIG_PATH_aarch64_unknown_linux_gnu=/path/to/aarch64/sysroot/usr/lib/aarch64-linux-gnu/pkgconfig:/path/to/aarch64/sysroot/usr/share/pkgconfig
```

The exact paths depend on the distribution that supplies the sysroot. Do not
point cross `pkg-config` at host-architecture `.pc` files.

`RUST_FONTCONFIG_DLOPEN=1` is not currently a substitute: it changes
`yeslogic-fontconfig-sys` to a dynamic symbol table while Fontique still uses
the ordinary function declarations, producing an incompatible feature
combination.

## Wabou packaging boundary

Use a versioned GNU target to combine the Zig-linked Rust executable, release
JavaScript bundle, and a native package in one command:

```bash
cargo install cargo-zigbuild --locked
wabou package apps/example \
  --target x86_64-unknown-linux-gnu.2.28 \
  --format appimage
```

Native installers are still created on the build host. Cross-architecture
sysroots, target-platform runtime smoke tests, and signing remain the release
pipeline's responsibility.

Prebuilt native dependencies keep their original ABI requirements. For
example, Manga OCR currently consumes a downloaded ONNX Runtime C++ archive;
that archive cannot be relinked to an older libstdc++/glibc ABI merely by
adding a Zig target suffix. Such applications need a matching native artifact
or an old-distribution build in addition to Wabou's packaging support.
