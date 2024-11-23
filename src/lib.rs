use anyhow::{Context, Result};
use handler::Handler;
use helper::helper_worker;
use hudsucker::{
    certificate_authority::RcgenAuthority,
    rcgen::{CertificateParams, KeyPair},
    rustls, Proxy,
};
use modder::Modder;
use settings::Settings;
use std::{future::Future, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::mpsc::channel;
use tracing::info;

mod handler;
mod helper;
mod modder;
mod parser;
pub(crate) mod proto;
mod settings;

pub mod prelude {
    pub use anyhow::Result;
    pub use tokio::sync::RwLock;
    pub use tracing::{info, warn};

    pub use crate::{
        build_and_start_proxy, init_trace,
        modder::Modder,
        settings::{ModSettings, Settings},
    };
}

pub(crate) const ARBITRARY_MD5: &str = "0123456789abcdef0123456789abcdef";

pub fn init_trace() {
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new("%H:%M:%S%.3f".to_string());
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::WARN.into())
        .from_env()
        .unwrap_or_default()
        .add_directive("majsoul_max_rs=info".parse().unwrap_or_default());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_timer(timer)
        .compact()
        .init();
}

fn generate_ca() -> Result<RcgenAuthority> {
    const KEY_PAIR: &str = include_str!("./ca/hudsucker.key");
    const CA_CERT: &str = include_str!("./ca/hudsucker.cer");
    let key_pair = KeyPair::from_pem(KEY_PAIR).context("Failed to parse key pair")?;
    let ca_cert = CertificateParams::from_ca_cert_pem(CA_CERT)
        .context("Failed to parse CA certificate")?
        .self_signed(&key_pair)
        .context("Failed to sign CA certificate")?;

    let ca = RcgenAuthority::new(
        key_pair,
        ca_cert,
        1_000,
        rustls::crypto::aws_lc_rs::default_provider(),
    );
    Ok(ca)
}

pub async fn build_and_start_proxy<F>(
    settings: &'static Settings,
    modder: Option<Modder>,
    graceful_shutdown: F,
) -> Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let ca = generate_ca()?;

    let proxy_addr = SocketAddr::from_str(settings.proxy_addr.as_str())
        .context("Failed to parse proxy address")?;
    let modder = modder.map(Arc::new);

    let tx = if settings.helper_on() {
        let (tx, rx) = channel(32);
        // start helper worker
        info!("Helper worker started");
        tokio::spawn(helper_worker(rx, settings));
        Some(tx)
    } else {
        None
    };
    let proxy = Proxy::builder()
        .with_addr(proxy_addr)
        .with_ca(ca)
        .with_rustls_client(rustls::crypto::aws_lc_rs::default_provider())
        .with_websocket_handler(Handler::new(tx, modder, settings))
        .with_graceful_shutdown(graceful_shutdown)
        .build()
        .context("Failed to build proxy")?;

    proxy.start().await.context("Failed to start proxy")
}
