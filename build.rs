//! Builds the vendored libopus (Xiph.Org) to a standalone wasm artifact via
//! emscripten. Only runs when the consuming crate targets wasm32 — on
//! native (the desktop/Android path) we don't compile anything here, the
//! existing `audiopus` (system libopus FFI) link continues to work.
//!
//! Built in FIXED_POINT mode: smaller wasm, deterministic across browsers,
//! and the analysis layer (psychoacoustic mode switching, float-only) isn't
//! needed — VoIP callers pass `OPUS_APPLICATION_VOIP` and the encoder picks
//! SILK appropriately without it.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }

    if Command::new("emcc").arg("--version").output().is_err() {
        panic!(
            "libopus-src: building for wasm32 but `emcc` (emscripten) is \
             not in PATH. Install it (`pacman -S emscripten` on Arch) and \
             source its profile (`source /etc/profile.d/emscripten.sh`)."
        );
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let vendor = manifest.join("vendor");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let obj_dir = out_dir.join("opus_obj");
    std::fs::create_dir_all(&obj_dir).unwrap();

    // Include paths mirror upstream's autotools build.
    let inc = [
        "include",
        "celt",
        "silk",
        "silk/fixed",
        "src",
        "",
    ];
    let inc_args: Vec<String> = inc
        .iter()
        .map(|p| format!("-I{}", vendor.join(p).display()))
        .collect();

    // Compile defines. The standard libopus fixed-point build:
    // - FIXED_POINT — use Q15/Q30 fixed-point DSP throughout
    // - VAR_ARRAYS — stack-allocate scratch buffers via C99 VLAs (clang/emcc
    //   supports them; avoids the alloca.h portability dance that
    //   USE_ALLOCA pulls in)
    // - OPUS_BUILD — we're building the library, not consuming it
    // - DISABLE_FLOAT_API — drop opus_*_float() from the public surface;
    //   our shim converts f32 ↔ i16 in JS instead
    // - PACKAGE_VERSION — surfaces via opus_get_version_string()
    // - HAVE_LRINTF / HAVE_LRINT — emscripten libc provides both
    // - HAVE_STDINT_H — for opus_types.h's intptr_t handling
    let defines: &[&str] = &[
        "-DFIXED_POINT=1",
        "-DVAR_ARRAYS=1",
        "-DOPUS_BUILD=1",
        "-DDISABLE_FLOAT_API=1",
        "-DPACKAGE_VERSION=\"1.5.2\"",
        "-DHAVE_LRINTF=1",
        "-DHAVE_LRINT=1",
        "-DHAVE_STDINT_H=1",
    ];

    // Source file selection. Match the upstream Makefile.am variables but
    // hand-rolled (no autoconf): we want CELT_SOURCES + SILK_SOURCES +
    // SILK_SOURCES_FIXED + OPUS_SOURCES, and explicitly NOT OPUS_SOURCES_FLOAT
    // (analysis.c, mlp.c, mlp_data.c — the float-only psychoacoustic layer).
    let skip_in_src = |name: &str| -> bool {
        matches!(
            name,
            "analysis.c" | "mlp.c" | "mlp_data.c" | "opus_demo.c" | "opus_compare.c"
        )
    };
    let mut sources: Vec<PathBuf> = Vec::new();
    for sub in ["celt", "silk", "silk/fixed", "src"] {
        let dir = vendor.join(sub);
        for entry in std::fs::read_dir(&dir).expect("read source dir") {
            let p = entry.unwrap().path();
            if p.extension().and_then(|e| e.to_str()) != Some("c") {
                continue;
            }
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if sub == "src" && skip_in_src(&name) {
                continue;
            }
            sources.push(p);
        }
    }

    // Compile each .c separately. -O2 keeps the wasm small without paying
    // the LTO cost; emcc's link step does its own dead-code elimination.
    let mut objects: Vec<PathBuf> = Vec::with_capacity(sources.len());
    for src in &sources {
        // Use the .c file's parent dir as a disambiguator: silk/CNG.c and
        // celt/cwrs.c won't clash, but `find_LPC_FIX.c` lives only in
        // silk/fixed so it's fine. Use a hash-free suffix derived from the
        // path components below `vendor/` to keep object names stable.
        let rel = src.strip_prefix(&vendor).unwrap();
        let flat = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("__");
        let obj = obj_dir.join(format!("{flat}.o"));
        let status = Command::new("emcc")
            .args(&inc_args)
            .args(defines)
            .args(["-std=c99", "-O2", "-c"])
            .arg(src)
            .arg("-o")
            .arg(&obj)
            .status()
            .expect("emcc invocation failed");
        if !status.success() {
            panic!("emcc failed compiling {}", src.display());
        }
        objects.push(obj);
    }

    // Link a standalone wasm artifact with the public Opus API exported.
    // We export the create/destroy/encode/decode entry points plus the
    // ctl helpers (so the JS shim can set bitrate, complexity, etc.), the
    // packet-introspection helpers (so the decoder side can size buffers),
    // and malloc/free (for argument marshalling from JS).
    let wasm_out = out_dir.join("libopus.wasm");
    let exports = [
        "_opus_encoder_get_size",
        "_opus_encoder_init",
        "_opus_encoder_create",
        "_opus_encoder_destroy",
        "_opus_encode",
        "_opus_encoder_ctl",
        "_opus_decoder_get_size",
        "_opus_decoder_init",
        "_opus_decoder_create",
        "_opus_decoder_destroy",
        "_opus_decode",
        "_opus_decoder_ctl",
        "_opus_packet_get_nb_samples",
        "_opus_packet_get_nb_frames",
        "_opus_packet_get_samples_per_frame",
        "_opus_packet_get_nb_channels",
        "_opus_packet_get_bandwidth",
        "_opus_strerror",
        "_opus_get_version_string",
        "_malloc",
        "_free",
    ]
    .join(",");
    let mut link = Command::new("emcc");
    link.arg("-O2")
        .arg("-sSTANDALONE_WASM")
        .arg(format!("-sEXPORTED_FUNCTIONS={exports}"))
        .arg("-sEXPORTED_RUNTIME_METHODS=")
        .arg("-sALLOW_MEMORY_GROWTH=1")
        .arg("--no-entry")
        .args(&objects)
        .arg("-o")
        .arg(&wasm_out);
    let status = link.status().expect("emcc link failed");
    if !status.success() {
        panic!("emcc link failed");
    }
    println!("cargo:rustc-env=LIBOPUS_WASM={}", wasm_out.display());
}
