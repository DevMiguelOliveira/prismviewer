#![allow(deprecated)]

use iroh::net::Endpoint;
use std::time::Instant;

const PROTOCOL_VERSION: u16 = 1;
const ALPN: &[u8] = b"prysmview/s3/1";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("S3 iroh P2P connection spike");
    println!("Protocol version: {}", PROTOCOL_VERSION);

    let start = Instant::now();
    let mut direct = 0u32;

    let ep_a = Endpoint::builder().discovery_n0().bind().await?;
    let ep_b = Endpoint::builder().discovery_n0().bind().await?;

    let addr_b = ep_b.node_addr().await?;

    let accept_handle = tokio::spawn(async move {
        if let Some(incoming) = ep_b.accept().await {
            let conn = incoming.await?;
            let (mut send, mut recv) = conn.accept_bi().await?;
            let mut buf = [0u8; 2];
            recv.read_exact(&mut buf).await?;
            let remote_ver = u16::from_be_bytes(buf);
            let local_ver = PROTOCOL_VERSION.to_be_bytes();
            send.write_all(&local_ver).await?;
            send.finish()?;
            Ok::<u16, anyhow::Error>(remote_ver)
        } else {
            Err(anyhow::anyhow!("no incoming connection"))
        }
    });

    match ep_a.connect(addr_b, ALPN).await {
        Ok(conn) => {
            let (mut send, mut recv) = conn.open_bi().await?;
            let ver = PROTOCOL_VERSION.to_be_bytes();
            send.write_all(&ver).await?;
            send.finish()?;
            let mut buf = [0u8; 2];
            recv.read_exact(&mut buf).await?;
            let remote_ver = u16::from_be_bytes(buf);
            if remote_ver == PROTOCOL_VERSION {
                println!("Handshake OK, remote version {}", remote_ver);
            }
            direct += 1;
            println!("Conexao direta estabelecida");
            let _ = conn;
        }
        Err(e) => {
            println!("Connection failed: {}", e);
        }
    }

    let _ = accept_handle.await;

    println!("\nResumo S3:");
    println!("  diretas: {}", direct);
    println!("  tempo total: {:?}", start.elapsed());

    let _ = ep_a.close(0u32.into(), b"done").await;
    Ok(())
}
