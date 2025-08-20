use majsoul_max_rs::*;
use std::sync::Arc;
use tokio::sync::watch;

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

    loop {
        match run_application().await {
            Ok(should_restart) => {
                if !should_restart {
                    break;
                }
                info!("重新启动应用程序...");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                warn!("应用程序运行出错: {e}");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                break;
            }
        }
    }

    Ok(())
}

async fn run_application() -> Result<bool> {
    let settings = Box::new(Settings::new(std::path::Path::new("./liqi_config"))?);
    let settings: &'static Settings = Box::leak(settings);
    let mod_settings = Arc::new(RwLock::new(ModSettings::new(settings)?));

    // show mod and helper switch status, green for on, red for off
    println!(
        "\n\x1b[{}mmod: {}\x1b[0m\n\x1b[{}mhelper: {}\x1b[0m\n",
        if settings.mod_on() { 32 } else { 31 },
        if settings.mod_on() { "on" } else { "off" },
        if settings.helper_on() { 32 } else { 31 },
        if settings.helper_on() { "on" } else { "off" }
    );

    let mut should_restart = false;

    // 创建重启信号通道
    let (restart_tx, mut restart_rx) = watch::channel(false);

    // 初始更新检查
    if settings.auto_update() {
        info!("自动更新liqi已开启");
        let mut new_settings = settings.clone();
        match new_settings.update().await {
            Err(e) => warn!("更新liqi失败: {e}"),
            Ok(true) => {
                info!("liqi更新成功, 重新实例化helper和modder");
                should_restart = true;
                return Ok(should_restart);
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
            match new_mod_settings.get_lqc(&settings.get_request_client()).await {
                Err(e) => warn!("更新mod失败: {e}"),
                Ok(true) => {
                    info!("mod更新成功, 重新实例化helper和modder");
                    should_restart = true;
                    return Ok(should_restart);
                }
                Ok(false) => (),
            }
        }
        let modder_mod_settings = RwLock::new(mod_settings.read().await.clone());
        Some(Modder::new(modder_mod_settings).await?)
    } else {
        None
    };

    // 启动定期更新检查任务
    if settings.auto_update() || (settings.mod_on() && mod_settings.read().await.auto_update()) {
        let update_settings = settings.clone();
        let update_mod_settings = mod_settings.clone();
        let update_restart_tx = restart_tx.clone();
        
        tokio::spawn(async move {
            periodic_update_check(update_settings, update_mod_settings, update_restart_tx).await;
        });
    }

    // 启动代理服务器
    let proxy_task = tokio::spawn(async move {
        build_and_start_proxy(settings, modder, shutdown_signal()).await
    });

    // 等待重启信号或代理服务器完成
    tokio::select! {
        _ = restart_rx.changed() => {
            if *restart_rx.borrow() {
                info!("收到重启信号，停止当前服务");
                should_restart = true;
            }
        }
        result = proxy_task => {
            match result {
                Ok(proxy_result) => {
                    if let Err(e) = proxy_result {
                        warn!("代理服务器出错: {e}");
                    }
                }
                Err(e) => warn!("代理任务出错: {e}"),
            }
        }
    }

    Ok(should_restart)
}

async fn periodic_update_check(
    settings: Settings,
    mod_settings: Arc<RwLock<ModSettings>>,
    restart_tx: watch::Sender<bool>,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5分钟
    info!("定期更新检查已启动，每5分钟检查一次");

    loop {
        interval.tick().await;
        
        let mut needs_restart = false;
        
        // 检查 liqi 更新
        if settings.auto_update() {
            let mut new_settings = settings.clone();
            match new_settings.update().await {
                Err(e) => warn!("定期检查liqi更新失败: {e}"),
                Ok(true) => {
                    info!("定期检查发现liqi更新，触发重启");
                    needs_restart = true;
                }
                Ok(false) => (),
            }
        }
        
        // 检查 mod 更新
        if settings.mod_on() && mod_settings.read().await.auto_update() {
            let mut new_mod_settings = mod_settings.read().await.clone();
            match new_mod_settings.get_lqc(&settings.get_request_client()).await {
                Err(e) => warn!("定期检查mod更新失败: {e}"),
                Ok(true) => {
                    info!("定期检查发现mod更新，触发重启");
                    // 更新 mod_settings 中的数据
                    *mod_settings.write().await = new_mod_settings;
                    needs_restart = true;
                }
                Ok(false) => (),
            }
        }
        
        if needs_restart {
            if let Err(e) = restart_tx.send(true) {
                warn!("发送重启信号失败: {e}");
            }
            break;
        }
    }
}
