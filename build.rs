use std::io::Result;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();
    config.out_dir("src/proto");
    config.type_attribute(
        "lq.ViewSlot",
        "#[derive(::serde::Serialize, ::serde::Deserialize)]",
    );
    config.compile_protos(&["liqi_config/liqi.proto"], &["liqi_config/", "proto/"])?;
    // cargo fmt
    std::process::Command::new("cargo").arg("fmt").status()?;
    Ok(())
}
