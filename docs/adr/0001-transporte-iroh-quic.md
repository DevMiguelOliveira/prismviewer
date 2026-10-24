# ADR 0001 — Transporte P2P via iroh (QUIC + TLS 1.3)

## Status

Aceito

## Contexto

Necessidade de transporte P2P seguro, baixa latência, NAT traversal e fallback relay para conexão Host↔Viewer em redes de loja hostis.

## Decisão

Adotar `iroh` (Rust) que usa QUIC com TLS 1.3 nativamente, descoberta de pares via chaves públicas Ed25519, NAT traversal (STUN/TURN/UPnP), fallback automático para `iroh-relay`.

## Alternativas consideradas e rejeitadas

1. **WebRTC:** Mais pesado, stack complexa (ICE, DTLS, SCTP), dependências C++/NSPR, binários maiores, suporte a relay limitado.
2. **TCP/TLS próprio sobre websocket/TCP:** Reinventar QUIC (retransmissão, controle de congestionamento, 0-RTT, migração de conexão), NAT traversal manual, sem fallback relay nativo.

## Consequências

Binário único Rust, dependência bem mantida, TLS 1.3 por padrão, relay como serviço separado (`prysmview-relay`), endereçamento por chave pública. Zero custo de licença.
