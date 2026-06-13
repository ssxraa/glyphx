# Tectonic engine

Glyph compiles LaTeX → PDF by driving the [Tectonic](https://tectonic-typesetting.github.io)
binary as a subprocess (the in-process crate needs vcpkg/native libs on Windows,
so the binary keeps the build simple and cross-platform).

The Rust command (`src/compile.rs`) resolves the binary in this order:

1. **`GLYPH_TECTONIC_BIN`** environment variable (absolute path), then
2. a **sidecar** next to the app executable (`tectonic[.exe]`), then
3. **`tectonic` on `PATH`**.

## Getting Tectonic

The quickest path during development is to put it on `PATH`:

- **Windows:** `choco install tectonic` (or `scoop install tectonic`)
- **macOS:** `brew install tectonic`
- **Linux:** `cargo install tectonic`, or your distro's package

…or download a release binary from
<https://github.com/tectonic-typesetting/tectonic/releases> and either drop it
in this folder, set `GLYPH_TECTONIC_BIN` to its path, or add it to `PATH`.

> First compile downloads Tectonic's TeX bundle (~once, cached). After that it
> works fully offline.

## Bundling for distribution

To ship Tectonic inside the installer, add it as a Tauri **sidecar**: place the
binary here named `tectonic-<target-triple>[.exe]` and add
`"externalBin": ["binaries/tectonic"]` under `bundle` in `tauri.conf.json`.
