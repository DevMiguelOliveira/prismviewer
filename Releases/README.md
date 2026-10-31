# PrysmView v2.0.0 - Release Notes

## 📦 Instaladores

### PrysmView Host (PDV)
- **Arquivo**: `PrysmView Host_2.0.0_x64_en-US.msi`
- **Tamanho**: 2.4 MB
- **Sistema**: Windows 10/11 x64
- **Função**: App que roda no PDV (Host UI + Backend Rust)

### PrysmView Admin (Operador)
- **Arquivo**: `PrysmView Admin_2.0.0_x64_en-US.msi`
- **Tamanho**: 2.4 MB
- **Sistema**: Windows 10/11 x64
- **Função**: App que roda no computador do operador (Admin Panel + Backend Rust)

## 🚀 Instalação

### No PDV (Host)

1. Faça download do arquivo `PrysmView Host_2.0.0_x64_en-US.msi`
2. Execute o instalador
3. Siga as instruções do wizard de instalação
4. O app será instalado como serviço Windows (executa em background)
5. Após a instalação, o Host será iniciado automaticamente

**Notas:**
- O Host é instalado como serviço Windows sob conta LocalSystem
- Gera automaticamente um par de chaves Ed25519 na primeira execução
- A chave pública é a identidade do PDV (exibida na interface)
- A chave privada é protegida por DPAPI (escopo de máquina)

### No Computador do Operador (Admin)

1. Faça download do arquivo `PrysmView Admin_2.0.0_x64_en-US.msi`
2. Execute o instalador
3. Siga as instruções do wizard de instalação
4. O app será instalado no menu Iniciar
5. Execute o "PrysmView Admin" para acessar o painel

**Notas:**
- O Admin é uma aplicação desktop normal (não é serviço)
- Requer login com credenciais de operador
- Conecta-se à API de sinalização HTTPS

## 🔧 Configuração

### Endpoint de API

Ambos os apps já estão configurados para usar:
- **API URL**: `https://prysmview.duckdns.org`

Para alterar o endpoint:
1. Abra o Admin Panel
2. Vá em "Configurações"
3. Altere o campo "API Endpoint"
4. Clique em "Salvar"

### Firewall

No PDV, certifique-se de que o firewall permite:
- **Saída**: Porta 443 (HTTPS) para conectar à API de sinalização
- **Entrada**: Portas UDP para WebRTC (TURN/STUN)

## 🔐 Segurança

### Chaves Ed25519
- O Host gera automaticamente um par de chaves Ed25519 na primeira execução
- A chave pública é a identidade do PDV (estilo TeamViewer: 117 029 2035)
- A chave privada é protegida por DPAPI (escopo de máquina)
- Chaves são armazenadas em: `%ProgramData%\PrysmView\keys\`

### Consentimento
- Toda sessão requer consentimento explícito do usuário no PDV
- Janela de consentimento mostra nome do operador e motivo
- Sem resposta em 60 segundos = sessão recusada
- Código de sessão de 8 dígitos (validade 120s, máx 3 tentativas)

### Criptografia
- WebRTC com DTLS 1.3
- SRTP para mídia
- SRTCP para controle
- Comunicação P2P com fallback via TURN

## 📡 Conexão

### Fluxo de Sessão

1. **Registro do Host**: O Host se registra automaticamente na API ao iniciar
2. **Login do Operador**: Operador faz login no Admin Panel
3. **Seleção do PDV**: Operador escolhe um PDV da lista
4. **Solicitação de Sessão**: API envia solicitação ao Host via WebSocket
5. **Consentimento**: Host mostra janela de consentimento no PDV
6. **Código de Sessão**: Ao aceitar, Host gera código de 8 dígitos
7. **Conexão P2P**: Operador digita código, conexão WebRTC é estabelecida
8. **Sessão Ativa**: Operador pode visualizar e controlar o PDV

### WebRTC
- STUN: `stun:stun.l.google.com:19302`
- TURN: Configurado na API (prysmview.duckdns.org:3340)
- Protocolo: DTLS 1.3 + SRTP

## 🐛 Troubleshooting

### Host não aparece na lista
- Verifique se o Host está rodando (serviço Windows)
- Verifique se o firewall permite saída HTTPS
- Verifique logs do Host: `%ProgramData%\PrysmView\logs\`

### Não consigo conectar
- Verifique se o código de sessão está correto
- Verifique se o código não expirou (120s)
- Verifique se não excedeu o limite de tentativas (3)
- Verifique logs do Admin para erros

### Erro de certificado SSL
- Verifique se o domínio `prysmview.duckdns.org` está configurado
- Verifique se o servidor Azure está rodando
- Execute: `curl https://prysmview.duckdns.org/health`

## 📞 Suporte

Para problemas ou dúvidas:
- Consulte a documentação em: `docs/`
- Verifique o runbook de disaster recovery: `docs/runbook-disaster-recovery.md`
- Logs do sistema: Windows Event Viewer

## 🔄 Atualizações

O Host suporta atualizações automáticas:
- Verifica periodicamente por novas releases
- Baixa automaticamente quando disponível
- Verifica assinatura Ed25519 antes de instalar
- Rollback automático em caso de falha

## 📄 Licença

MIT OR Apache-2.0

---

**Versão**: 2.0.0
**Data**: 30 de Outubro de 2026
**Plataforma**: Windows 10/11 x64
