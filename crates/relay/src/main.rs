#![allow(deprecated)]

use anyhow::Result;
use iroh::net::Endpoint;
use std::net::SocketAddrV4;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

/// Contador de telemetria de tráfego do relay (P-07).
/// Registra apenas quantidade de bytes e conexões sem inspecionar qualquer conteúdo cifrado.
#[derive(Debug, Default)]
pub struct RelayMetrics {
    pub bytes_relayed: AtomicU64,
    pub connections_count: AtomicU64,
}

impl RelayMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_bytes(&self, bytes: u64) {
        self.bytes_relayed.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_connection(&self) {
        self.connections_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn bytes_relayed(&self) -> u64 {
        self.bytes_relayed.load(Ordering::Relaxed)
    }

    pub fn connections_count(&self) -> u64 {
        self.connections_count.load(Ordering::Relaxed)
    }

    /// Alerta de consumo quando o tráfego do relay ultrapassar o teto mensal/orçamentário (P-07).
    pub fn check_consumption_alert(&self, threshold_bytes: u64) -> bool {
        let current = self.bytes_relayed();
        if current >= threshold_bytes {
            warn!(
                "ALERTA DE CONSUMO DO RELAY: Limite de {} bytes atingido (Atual: {} bytes)!",
                threshold_bytes, current
            );
            true
        } else {
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr: SocketAddrV4 = std::env::var("RELAY_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3340".to_string())
        .parse()?;

    info!("Iniciando prysmview-relay em {:?}", addr);
    let metrics = Arc::new(RelayMetrics::new());

    // Inicia endpoint relay iroh com descoberta ativa e suporte a QUIC
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
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Sinal de encerramento recebido. Total de bytes repassados: {}", metrics.bytes_relayed());
        }
    }

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

    #[test]
    fn test_relay_metrics_accounting_and_alert() {
        let metrics = RelayMetrics::new();
        assert_eq!(metrics.bytes_relayed(), 0);
        assert_eq!(metrics.connections_count(), 0);

        metrics.record_connection();
        metrics.record_bytes(1024 * 1024); // 1 MB

        assert_eq!(metrics.connections_count(), 1);
        assert_eq!(metrics.bytes_relayed(), 1_048_576);

        // Abaixo do threshold de 10 MB
        assert!(!metrics.check_consumption_alert(10 * 1024 * 1024));

        // Acima do threshold de 500 KB
        assert!(metrics.check_consumption_alert(500 * 1024));
    }
}
