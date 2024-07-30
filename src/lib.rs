use clap::Parser;

pub mod helper;
pub mod modder;
pub mod parser;
pub mod proto;
pub mod settings;

pub const ARBITRARY_MD5: &str = "0123456789abcdef0123456789abcdef";

#[derive(Parser, Debug)]
pub struct Arg {
    #[clap(short, long, default_value = "./liqi_config/")]
    config_dir: String,
}
