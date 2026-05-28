# libopus-src

[![crates.io](https://img.shields.io/crates/v/libopus-src.svg)](https://crates.io/crates/libopus-src)
[![docs.rs](https://docs.rs/libopus-src/badge.svg)](https://docs.rs/libopus-src)

Vendored [libopus](https://opus-codec.org/) (BSD-3-Clause, by Xiph.Org /
Skype Limited / Broadcom / Octasic / Jean-Marc Valin and contributors)
compiled to a standalone WebAssembly artifact via
[Emscripten](https://emscripten.org/). Originally written for
[Voicetastic](https://git.cha-sam.re/voicetastic)'s browser client, but
useful for any Rust + wasm project that needs Opus encode/decode without
depending on the browser's `WebCodecs.AudioEncoder`.

The source under [`vendor/`](./vendor) is **unmodified** upstream — see
[`vendor/COPYING`](./vendor/COPYING). This crate adds only:

- [`build.rs`](./build.rs) — drives `emcc` over the C source on wasm32
  targets (no-op on native), selecting the upstream `OPUS_SOURCES` +
  `CELT_SOURCES` + `SILK_SOURCES` + `SILK_SOURCES_FIXED` file lists.
- [`src/lib.rs`](./src/lib.rs) — `wasm_module_bytes()` hands the resulting
  `.wasm` to consumers.

Built in **FIXED_POINT mode** with **DISABLE_FLOAT_API**: smaller wasm
(~250 KB vs ~400 KB for float), deterministic across browsers, no
analysis-based mode switching (the encoder picks SILK/CELT/hybrid from
the `application` arg — pass `OPUS_APPLICATION_VOIP` for voice). The DNN
noise-suppression layer is not compiled.

The crate version tracks upstream libopus releases (currently `1.5.2`,
Apr 2024). Local changes on top of the build glue, if any, are tagged
with semver build metadata (e.g. `1.5.2+vt.1`).

## Usage

```toml
[dependencies]
libopus-src = "1.5"
```

```rust
// On wasm32 only: bytes of a standalone `.wasm` module exporting the
// Opus C API. Instantiate from JS with one no-op env import
// (`emscripten_notify_memory_growth`); see voicetastic-core's
// `codec/opus_shim.js` for a working example.
#[cfg(target_arch = "wasm32")]
let bytes: &'static [u8] = libopus_src::wasm_module_bytes();
```

## Build requirements

Only does work when the consuming crate targets `wasm32-unknown-unknown`.
On native (desktop / Android) the build script returns early — native
consumers keep their existing libopus integration (e.g. `audiopus`,
`opus` crate against system `libopus.so`).

For the wasm build you need Emscripten in `PATH`:

```sh
# Arch:
sudo pacman -S emscripten
source /etc/profile.d/emscripten.sh

# Debian/Ubuntu:
sudo apt install emscripten

# Otherwise:
# https://emscripten.org/docs/getting_started/downloads.html
```

Then any consumer's `cargo build --target wasm32-unknown-unknown` will
run this build script automatically; the resulting `libopus.wasm`
(~250 KB) lands under `OUT_DIR` and is read by `wasm_module_bytes()` via
`include_bytes!`.

## Wire-compat note

Opus is a single standardised codec (RFC 6716) — bytes on the wire are
identical whether the encode side is the system `libopus.so` (desktop /
Android) or this wasm artifact (browser). FIXED_POINT vs floating-point
affects encoder-side bit allocation marginally (typical SNR difference
< 0.5 dB at the same bitrate); the resulting bitstream is decoded
identically by any conformant Opus decoder.

## License

- This crate's build glue (`build.rs`, `src/lib.rs`, `Cargo.toml`,
  `README.md`): BSD-3-Clause (to match upstream).
- The vendored libopus source under `vendor/`: BSD-3-Clause, © Xiph.Org
  Foundation / Skype Limited / Broadcom / Octasic / Jean-Marc Valin and
  contributors. See [`vendor/COPYING`](./vendor/COPYING) and
  [`vendor/AUTHORS`](./vendor/AUTHORS).
- A copy of the licence text also lives at [`LICENSE`](./LICENSE) at the
  crate root for crates.io metadata.

Opus is royalty-free per the IETF policy at the time of standardisation
(RFC 6716); patent holders contributed grants. Apple, Microsoft, Skype,
Xiph, Broadcom, and others all hold patents declared essential and
licensed under the IETF terms. This crate is BSD-3-Clause but those
patent grants are separate from the source licence.
