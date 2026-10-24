#![allow(deprecated)]

use anyhow::Result;
use iroh::net::Endpoint;
use std::net::SocketAddrV4;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr: SocketAddrV4 = std::env::var("RELAY_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3340".to_string())
        .parse()?;

    info!("Iniciando prysmview-relay em {:?}", addr);

    // Inicia endpoint relay iroh com descoberta ativa
    let ep = Endpoint::builder()
        .discovery_n0()
        .bind_addr_v4(addr)
        .bind()
        .await?;

    info!(
        "prysmview-relay escutando com sucesso: {:?}",
        ep.node_addr().await?
    );
    info!("Chave publica do relay: {:?}", ep.node_id());

    // Mantem o relay ativo ate sinal de interrupcao
    tokio::signal::ctrl_c().await?;
    info!("Encerrando prysmview-relay...");

    let _ = ep.close(0u32.into(), b"shutdown").await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_relay_port() {
        let addr: SocketAddrV4 = "0.0.0.0:3340".parse().unwrap();
        assert_eq!(addr.port(), 3340);
    }
}
