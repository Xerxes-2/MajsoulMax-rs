use anyhow::Result;
use tokio::sync::RwLock;

use majsoul_max_rs::{
    build_and_start_proxy, init_trace,
    modder::Modder,
    settings::{ModSettings, Settings},
};
use tracing::{info, warn};

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() -> Result<()> {
    init_trace();

    // print red declaimer text
    println!(
        "
    MajsoulMax-rs {}
    \x1b[31m
    本项目完全免费开源，如果您购买了此程序，请立即退款！
    项目地址: https://github.com/Xerxes-2/MajsoulMax-rs

    本程序仅供学习交流使用，严禁用于商业用途！
    请遵守当地法律法规，对于使用本程序所产生的任何后果，作者概不负责！
    \x1b[0m",
        env!("CARGO_PKG_VERSION")
    );

    let settings = Box::new(Settings::new()?);
    let settings: &'static Settings = Box::leak(settings);
    let mod_settings = RwLock::new(ModSettings::new(settings)?);

    // show mod and helper switch status, green for on, red for off
    println!(
        "\n\x1b[{}mmod: {}\x1b[0m\n\x1b[{}mhelper: {}\x1b[0m\n",
        if settings.mod_on() { 32 } else { 31 },
        if settings.mod_on() { "on" } else { "off" },
        if settings.helper_on() { 32 } else { 31 },
        if settings.helper_on() { "on" } else { "off" }
    );

    if settings.auto_update() {
        info!("自动更新liqi已开启");
        let mut new_settings = settings.clone();
        match new_settings.update().await {
            Err(e) => warn!("更新liqi失败: {e}"),
            Ok(true) => {
                info!("liqi更新成功, 请重启程序");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                return Ok(());
            }
            _ => (),
        }
    }

    let modder = if settings.mod_on() {
        // start mod worker
        info!("Mod worker started");
        if mod_settings.read().await.auto_update() {
            info!("自动更新mod已开启");
            let mut new_mod_settings = mod_settings.read().await.clone();
            match new_mod_settings.get_lqc().await {
                Err(e) => warn!("更新mod失败: {e}"),
                Ok(true) => {
                    info!("mod更新成功, 请重启程序");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    return Ok(());
                }
                Ok(false) => (),
            }
        }
        Some(Modder::new(mod_settings).await?)
    } else {
        None
    };

    build_and_start_proxy(settings, modder, shutdown_signal()).await
}
