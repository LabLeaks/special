/**
@module SPECIAL.LANGUAGE_PACKS.REGISTRY_GENERATION
Generates the compile-time language-pack registry consumed by the shared pack loader.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.REGISTRY_GENERATION
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::Compression;
use flate2::write::GzEncoder;

fn main() {
    println!("cargo:rerun-if-changed=src/language_packs");
    println!("cargo:rerun-if-changed=proof/lean");
    println!("cargo:rerun-if-env-changed=SPECIAL_BUILD_LEAN_KERNEL");
    println!("cargo:rustc-check-cfg=cfg(special_embedded_lean_kernel)");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let packs_dir = manifest_dir.join("src/language_packs");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("out dir"));
    let output_path = out_dir.join("language_pack_registry.rs");

    let mut pack_modules = fs::read_dir(&packs_dir)
        .expect("language packs dir should exist")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                return None;
            }
            let stem = path.file_stem()?.to_str()?;
            // Python stays in-tree as dormant work, but it is intentionally
            // excluded from the shipped built-in pack registry for 0.7.0 until
            // its toolchain and bridge honesty are hardened enough to ship.
            (stem != "mod" && stem != "python").then(|| stem.to_string())
        })
        .collect::<Vec<_>>();
    pack_modules.sort();

    let mut generated = String::new();
    for module in &pack_modules {
        let path = packs_dir.join(format!("{module}.rs"));
        generated.push_str(&format!(
            "#[path = \"{}\"]\npub(crate) mod {module};\n",
            rust_path_attr_literal(&path)
        ));
    }
    generated.push('\n');
    generated
        .push_str("pub(crate) const REGISTERED_LANGUAGE_PACKS: &[&LanguagePackDescriptor] = &[\n");
    for module in &pack_modules {
        generated.push_str(&format!("    &{module}::DESCRIPTOR,\n"));
    }
    generated.push_str("];\n");

    fs::write(output_path, generated).expect("write generated language pack registry");

    if build_lean_kernel_enabled() {
        build_embedded_lean_kernel(&manifest_dir, &out_dir);
    }
}

fn rust_path_attr_literal(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn build_lean_kernel_enabled() -> bool {
    matches!(
        env::var("SPECIAL_BUILD_LEAN_KERNEL").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn build_embedded_lean_kernel(manifest_dir: &Path, out_dir: &Path) {
    if !lean_kernel_target_matches_host() {
        return;
    }

    let lean_root = manifest_dir.join("proof/lean");
    let exe_suffix = if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        ".exe"
    } else {
        ""
    };
    let embedded_kernel = out_dir.join(format!("special_traceability_kernel{exe_suffix}"));
    let compressed_kernel = out_dir.join(format!("special_traceability_kernel{exe_suffix}.gz"));
    let cache_dir = lean_kernel_cache_dir(manifest_dir, &lean_root, exe_suffix);
    let cached_kernel = cache_dir.join(format!("special_traceability_kernel{exe_suffix}"));
    let cached_compressed_kernel =
        cache_dir.join(format!("special_traceability_kernel{exe_suffix}.gz"));

    if cached_kernel.is_file() && cached_compressed_kernel.is_file() {
        fs::copy(&cached_kernel, &embedded_kernel).unwrap_or_else(|error| {
            panic!(
                "copy cached Lean traceability kernel from {} to {}: {error}",
                cached_kernel.display(),
                embedded_kernel.display()
            )
        });
        fs::copy(&cached_compressed_kernel, &compressed_kernel).unwrap_or_else(|error| {
            panic!(
                "copy cached compressed Lean traceability kernel from {} to {}: {error}",
                cached_compressed_kernel.display(),
                compressed_kernel.display()
            )
        });
        let uncompressed_len = file_len(&embedded_kernel, "cached Lean traceability kernel");
        emit_embedded_lean_kernel_env(&compressed_kernel, exe_suffix, uncompressed_len);
        return;
    }

    run_lake(
        &lean_root,
        &["clean"],
        "clean Lean traceability kernel build",
    );
    run_lake(
        &lean_root,
        &["-Krelease", "build", "special_traceability_kernel"],
        "build Lean traceability kernel",
    );

    let built_kernel = lean_root
        .join(".lake/build/bin")
        .join(format!("special_traceability_kernel{exe_suffix}"));
    fs::copy(&built_kernel, &embedded_kernel).unwrap_or_else(|error| {
        panic!(
            "copy Lean traceability kernel from {} to {}: {error}",
            built_kernel.display(),
            embedded_kernel.display()
        )
    });
    strip_embedded_lean_kernel(&embedded_kernel);
    let uncompressed_len = file_len(&embedded_kernel, "stripped Lean traceability kernel");
    gzip_file(&embedded_kernel, &compressed_kernel);
    fs::create_dir_all(&cache_dir).unwrap_or_else(|error| {
        panic!(
            "create Lean traceability kernel cache directory {}: {error}",
            cache_dir.display()
        )
    });
    fs::copy(&embedded_kernel, &cached_kernel).unwrap_or_else(|error| {
        panic!(
            "cache Lean traceability kernel at {}: {error}",
            cached_kernel.display()
        )
    });
    fs::copy(&compressed_kernel, &cached_compressed_kernel).unwrap_or_else(|error| {
        panic!(
            "cache compressed Lean traceability kernel at {}: {error}",
            cached_compressed_kernel.display()
        )
    });

    emit_embedded_lean_kernel_env(&compressed_kernel, exe_suffix, uncompressed_len);
}

fn lean_kernel_target_matches_host() -> bool {
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();
    if target == host {
        return true;
    }
    println!(
        "cargo:warning=skipping embedded Lean traceability kernel for cross target {target}; \
         Lake builds a host executable on {host}, so this target will need \
         SPECIAL_TRACEABILITY_KERNEL_EXE at runtime for Lean traceability"
    );
    false
}

fn emit_embedded_lean_kernel_env(
    compressed_kernel: &Path,
    exe_suffix: &str,
    uncompressed_len: u64,
) {
    println!("cargo:rustc-cfg=special_embedded_lean_kernel");
    println!(
        "cargo:rustc-env=SPECIAL_EMBEDDED_LEAN_KERNEL_PATH={}",
        compressed_kernel.display()
    );
    println!(
        "cargo:rustc-env=SPECIAL_EMBEDDED_LEAN_KERNEL_FILENAME=special_traceability_kernel{exe_suffix}"
    );
    println!("cargo:rustc-env=SPECIAL_EMBEDDED_LEAN_KERNEL_UNCOMPRESSED_LEN={uncompressed_len}");
}

fn file_len(path: &Path, description: &str) -> u64 {
    fs::metadata(path)
        .unwrap_or_else(|error| {
            panic!("read {description} metadata at {}: {error}", path.display())
        })
        .len()
}

fn lean_kernel_cache_dir(manifest_dir: &Path, lean_root: &Path, exe_suffix: &str) -> PathBuf {
    target_dir(manifest_dir)
        .join("special-lean-kernel-cache")
        .join(lean_kernel_cache_key(lean_root, exe_suffix))
}

fn target_dir(manifest_dir: &Path) -> PathBuf {
    match env::var("CARGO_TARGET_DIR") {
        Ok(path) => {
            let path = PathBuf::from(path);
            if path.is_absolute() {
                path
            } else {
                manifest_dir.join(path)
            }
        }
        Err(_) => manifest_dir.join("target"),
    }
}

fn lean_kernel_cache_key(lean_root: &Path, exe_suffix: &str) -> String {
    let mut hasher = Fnv64::new();
    hasher.write_str("special-lean-traceability-kernel-cache-v1");
    hasher.write_str(exe_suffix);
    hasher.write_str(&env::var("TARGET").unwrap_or_default());
    hasher.write_str(&lean_toolchain(lean_root));
    for path in lean_kernel_input_files(lean_root) {
        let relative = path.strip_prefix(lean_root).unwrap_or(&path);
        hasher.write_str(&relative.to_string_lossy());
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("read Lean kernel input {}: {error}", path.display()));
        hasher.write_bytes(&bytes);
    }
    format!("{:016x}", hasher.finish())
}

fn lean_kernel_input_files(lean_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_lean_kernel_input_files(lean_root, &mut files);
    files.sort();
    files
}

fn collect_lean_kernel_input_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(dir)
        .unwrap_or_else(|error| {
            panic!(
                "read Lean kernel input directory {}: {error}",
                dir.display()
            )
        })
        .map(|entry| {
            entry
                .expect("Lean kernel input directory entry should be readable")
                .path()
        })
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.file_name().and_then(|name| name.to_str()) == Some(".lake") {
            continue;
        }
        if path.is_dir() {
            collect_lean_kernel_input_files(&path, files);
            continue;
        }
        let file_name = path.file_name().and_then(|name| name.to_str());
        let extension = path.extension().and_then(|extension| extension.to_str());
        if file_name == Some("lean-toolchain")
            || matches!(extension, Some("lean" | "toml" | "json"))
        {
            files.push(path);
        }
    }
}

struct Fnv64(u64);

impl Fnv64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write_str(&mut self, value: &str) {
        self.write_bytes(value.as_bytes());
        self.write_bytes(&[0]);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

fn strip_embedded_lean_kernel(path: &Path) {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        return;
    }
    let status = Command::new("strip")
        .arg(path)
        .status()
        .expect("strip embedded Lean traceability kernel");
    if !status.success() {
        panic!(
            "strip embedded Lean traceability kernel at {} failed with {status}",
            path.display()
        );
    }
}

fn gzip_file(source: &Path, destination: &Path) {
    let bytes = fs::read(source).unwrap_or_else(|error| {
        panic!(
            "read Lean traceability kernel for compression at {}: {error}",
            source.display()
        )
    });
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(&bytes)
        .unwrap_or_else(|error| panic!("gzip Lean traceability kernel bytes: {error}"));
    let compressed = encoder
        .finish()
        .unwrap_or_else(|error| panic!("finish gzip Lean traceability kernel: {error}"));
    fs::write(destination, compressed).unwrap_or_else(|error| {
        panic!(
            "write compressed Lean traceability kernel to {}: {error}",
            destination.display()
        )
    });
}

fn run_lake(lean_root: &Path, lake_args: &[&str], description: &str) {
    let toolchain = lean_toolchain(lean_root);
    let mut command = Command::new("mise");
    command
        .args(["exec", "--", "elan", "run"])
        .arg(&toolchain)
        .arg("lake")
        .args(lake_args)
        .current_dir(lean_root)
        .env_remove("PROFILE")
        .env_remove("DEBUG")
        .env_remove("OPT_LEVEL")
        .env_remove("OUT_DIR");
    for (key, _) in env::vars() {
        if key.starts_with("CARGO_") && key != "CARGO_HOME" {
            command.env_remove(key);
        }
    }
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("{description}: {error}"));
    if !status.success() {
        panic!("{description} failed with {status}");
    }
}

fn lean_toolchain(lean_root: &Path) -> String {
    fs::read_to_string(lean_root.join("lean-toolchain"))
        .expect("read Lean toolchain file")
        .trim()
        .to_string()
}
