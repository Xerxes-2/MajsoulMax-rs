use std::io::Result;

fn main() -> Result<()> {
    prost_build::Config::new()
        .out_dir("src/proto")
        .type_attribute(
            "lq.ViewSlot",
            "#[derive(::serde::Serialize, ::serde::Deserialize)]",
        )
        .compile_protos(&["proto/liqi.proto"], &["proto/"])?;
    // cargo fmt
    std::process::Command::new("cargo").arg("fmt").status()?;
    Ok(())
}
