# ADR 0002 — Escolha da Tecnologia do Instalador Windows

## Status
Aceito

## Contexto
O instalador Windows deve ser de custo zero e suportar:
- Instalação do serviço Windows `prysmview-svc` em `LocalSystem` com registro de serviço e recuperação em falha;
- Instalação do aplicativo de bandeja `prysmview-tray` na sessão do usuário logado;
- Geração de chave Ed25519 do PDV no primeiro boot (protegida por DPAPI);
- Desinstalação limpa com confirmação;
- Assinatura de código opcional (Authenticode);
- Tamanho do instalador abaixo de 30 MB.

## Decisão
**Inno Setup 6** (Grátis, open-source, script Pascal, amplamente usado).

## Alternativas Consideradas e Rejeitadas
1. **WiX Toolset v4:** Requer .NET SDK, curva de aprendizado XML/WSX mais íngeme, tamanho de build maior, assinatura Authenticode mais complexa sem certificado pago.

## Consequências
- Script `.iss` compacto, suporte nativo a serviços (`[Services]`), entradas de registro, arquivos por condição (32/64-bit);
- Fácil integração no CI GitHub Actions;
- Instalação silenciosa `/VERYSILENT` e desinstalador automático;
- Instalador resultante abaixo de 30 MB.