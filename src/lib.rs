//! Vendored libopus compiled to a standalone wasm artifact for browser
//! clients. See `vendor/` for the original source (BSD-3-Clause).
//!
//! Native targets do nothing here — desktop/Android keep their existing
//! `audiopus` (system libopus FFI) integration. On `wasm32` the build
//! script runs `emcc` (Emscripten) to produce a standalone `.wasm` that
//! exports the Opus public C API; the bytes are baked into this crate
//! via [`wasm_module_bytes`] and a JS shim instantiates them at runtime.

#![allow(rustdoc::broken_intra_doc_links)]

/// The standalone wasm artifact emitted by `build.rs` from the vendored C
/// source. Loaded by the browser driver as a `WebAssembly.Module`.
///
/// Exports (callable from JS once instantiated):
///
/// - Encoder: `opus_encoder_create`, `opus_encoder_destroy`, `opus_encode`,
///   `opus_encoder_ctl`, `opus_encoder_get_size`, `opus_encoder_init`
/// - Decoder: `opus_decoder_create`, `opus_decoder_destroy`, `opus_decode`,
///   `opus_decoder_ctl`, `opus_decoder_get_size`, `opus_decoder_init`
/// - Packet introspection: `opus_packet_get_nb_samples`,
///   `opus_packet_get_nb_frames`, `opus_packet_get_samples_per_frame`,
///   `opus_packet_get_nb_channels`, `opus_packet_get_bandwidth`
/// - Utility: `opus_strerror`, `opus_get_version_string`
/// - Memory: `malloc`, `free` (for marshalling buffers across the JS↔wasm
///   boundary)
///
/// Built in `FIXED_POINT` mode — the wasm has no float-only entry points
/// (`opus_encode_float`, `opus_decode_float`). JS callers convert i16↔f32.
///
/// One required import: `env.emscripten_notify_memory_growth(idx: u32)`,
/// which the JS shim provides as a no-op.
#[cfg(target_arch = "wasm32")]
pub fn wasm_module_bytes() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/libopus.wasm"))
}
