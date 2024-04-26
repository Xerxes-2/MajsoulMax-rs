use clap::Parser;
use once_cell::sync::Lazy;
use settings::Settings;

pub mod helper;
pub mod lq_config;
pub mod modder;
pub mod parser;
pub mod settings;
pub mod sheets;
pub mod base;

pub static SETTINGS: Lazy<Settings> = Lazy::new(Settings::new);
pub const ARBITRARY_MD5: &str = "0123456789abcdef0123456789abcdef";
pub static ARG: Lazy<Arg> = Lazy::new(Arg::parse);

#[derive(Parser, Debug)]
pub struct Arg {
    #[clap(short, long, default_value = "./liqi_config/")]
    config_dir: String,
}
