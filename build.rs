use std::{env, io::Result, path::PathBuf};

fn main() -> Result<()> {
    prost_build::Config::new()
        .out_dir("src/proto")
        .type_attribute(
            "lq.ViewSlot",
            "#[derive(::serde::Serialize, ::serde::Deserialize)]",
        )
        .file_descriptor_set_path(
            PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"))
                .join("liqi_desc.bin"),
        )
        .compile_protos(&["proto/liqi.proto"], &["proto/"])?;

    Ok(())
}
