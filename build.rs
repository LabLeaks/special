/**
@module SPECIAL.LANGUAGE_PACKS.REGISTRY_GENERATION
Generates the compile-time language-pack registry consumed by the shared pack loader.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.REGISTRY_GENERATION
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/language_packs");

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
}

fn rust_path_attr_literal(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
