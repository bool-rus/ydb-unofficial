use std::error::Error;

use std::path::PathBuf;


use walkdir::{WalkDir, DirEntry};

const DST_FOLDER: &str = "src/generated";

const COMPILE_FILES: &[&str] = &[
    "ydb_scheme_v1.proto",
    "ydb_discovery_v1.proto",
    "ydb_table_v1.proto",
    "ydb_topic_v1.proto",
    "ydb_auth_v1.proto"
];

const INCLUDE_DIRS: &[&str] = &["ydb-api-protos"];

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var("CARGO_FEATURE_REGENERATE_SOURCES").unwrap_or_else(|_| "0".into()) != "1" {
        println!("skip regenerate sources");
        return Ok(());
    };
    println!("cargo:rerun-if-changed=ydb-api-protos");
    //clean_dst_dir(DST_FOLDER)?;

    let descriptor_file = PathBuf::from(DST_FOLDER).join("../descriptors.bin");

    tonic_build::compile_protos("ydb-api-protos/ydb_auth_v1.proto")?;
    /*
    let mut cfg = prost_build::Config::default();
    cfg.compile_well_known_types()
        .type_attribute(".Ydb", "#[derive(serde::Serialize, serde::Deserialize)]")
        // .extern_path(".google.protobuf", "::pbjson_types")
        .file_descriptor_set_path(&descriptor_file);

    tonic_build::configure()
        .build_server(false)
        .out_dir(DST_FOLDER)
        .compile_well_known_types(true)
        .compile_with_config(config, protos, includes);*/
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir(DST_FOLDER)
        //.include_file("mod.rs")
        .compile_well_known_types(true)
        .extern_path(".ydb.auth.v1", "ydb::auth::v1")
        /* 
        // the serialize attributes is workaround
        // in future need to find/write good serialization for the types
        .type_attribute(
            "google.protobuf.Timestamp",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "google.protobuf.Empty",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "google.protobuf.Duration",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "google.protobuf.Any",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )*/
        .compile(COMPILE_FILES, INCLUDE_DIRS)?;
    make_modules()?;
    Ok(())
}

fn make_modules() -> Result<(), Box<dyn Error>> {
    for entry in WalkDir::new(DST_FOLDER) {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            let name = entry.file_name().to_string_lossy();
            if name.ends_with(".rs") && name != "mod.rs" {
                let module_name: String = name.chars().take(name.len()-3).collect();
                let module_path = DST_FOLDER.to_owned() + "/" + module_name.split('.').collect::<Vec<_>>().join("/").as_str() + "/mod.rs";
                safety_move(entry, &module_path)?;
                attach_mod(&module_path)?;
            }
        }
    }
    Ok(())
}

fn safety_move(entry: DirEntry, module_file: &str) -> Result<(), Box<dyn Error>> {
    use std::fs::{create_dir_all, rename};
    let path = PathBuf::from(module_file);
    if let Some(dir) = path.parent() {
        create_dir_all(dir)?;
    }
    rename(entry.path(), path)?;
    Ok(())
}

fn attach_mod(module_file: &str) -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(module_file);
    let module_folder = path.parent().unwrap();
    let module_name = module_folder.file_name().unwrap().to_string_lossy();
    let mut parent = module_folder.parent().unwrap().to_path_buf();
    parent.push("mod.rs");
    println!("update file: {}", parent.to_string_lossy());
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(parent)
        .unwrap();
    use std::io::prelude::*;
    writeln!(file, "\npub mod {module_name};")?;
    Ok(())
}