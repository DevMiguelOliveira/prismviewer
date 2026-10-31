# Guia de Instalação - PrysmView v2.0.0

## Pré-requisitos

### Sistema Operacional
- Windows 10 (64-bit) ou superior
- Windows 11 (64-bit)

### Requisitos de Rede
- Conexão com internet
- Porta 443 (HTTPS) aberta para saída
- Portas UDP para WebRTC (automático)

### Requisitos de Hardware
- **Mínimo**: CPU Dual-core, 4GB RAM, 10GB disco
- **Recomendado**: CPU Quad-core, 8GB RAM, 20GB disco

---

## Instalação do PrysmView Host (PDV)

### Passo 1: Download
1. Acesse a pasta de releases
2. Faça download do arquivo: `PrysmView Host_2.0.0_x64_en-US.msi`

### Passo 2: Executar Instalador
1. Clique duas vezes no arquivo MSI
2. Clique em "Install" quando solicitado pelo UAC (User Account Control)
3. Aguarde a instalação completar
4. Clique em "Finish" ao finalizar

### Passo 3: Verificar Instalação
1. Abra o "Gerenciador de Tarefas"
2. Vá para a aba "Serviços"
3. Procure por "PrysmView Host"
4. Verifique se o status é "Running"

### Passo 4: Obter ID do PDV
1. O Host iniciará automaticamente após a instalação
2. A interface do Host mostrará o ID do PDV (estilo TeamViewer)
3. Anote este ID (ex: 117 029 2035)
4. Este ID será usado para identificar o PDV no Admin Panel

### Passo 5: Configurar Firewall (se necessário)
Se o Windows Firewall estiver ativado, adicione uma regra:
1. Abra "Windows Defender Firewall"
2. Clique em "Configurações Avançadas"
3. Regras de Entrada > Nova Regra
4. Programa: Caminho do executável do Host
5. Permitir conexão
6. Selecione todos os perfis (Domínio, Privado, Público)
7. Nome: "PrysmView Host"

---

## Instalação do PrysmView Admin (Operador)

### Passo 1: Download
1. Acesse a pasta de releases
2. Faça download do arquivo: `PrysmView Admin_2.0.0_x64_en-US.msi`

### Passo 2: Executar Instalador
1. Clique duas vezes no arquivo MSI
2. Clique em "Install" quando solicitado pelo UAC
3. Aguarde a instalação completar
4. Clique em "Finish" ao finalizar

### Passo 3: Iniciar Aplicação
1. Abra o Menu Iniciar
2. Procure por "PrysmView Admin"
3. Clique para abrir

### Passo 4: Primeiro Login
1. Na tela de login, insira suas credenciais
2. Clique em "Entrar"
3. Você será redirecionado para o Dashboard

---

## Primeiro Uso

### 1. Registrar PDV

Após instalar o Host em um PDV:
1. O Host se registrará automaticamente na API
2. Aguarde alguns segundos
3. Abra o Admin Panel
4. Vá para "Dispositivos"
5. O PDV aparecerá na lista com status "online"

### 2. Conectar ao PDV

1. No Admin Panel, selecione o PDV desejado
2. Clique em "Conectar"
3. No PDV, aparecerá uma janela de consentimento
4. O usuário no PDV deve clicar em "Aceitar"
5. Um código de 8 dígitos será exibido no PDV
6. Digite este código no Admin Panel
7. A conexão será estabelecida

### 3. Controlar o PDV

Após conectar:
- Você verá a tela do PDV em tempo real
- Pode mover o mouse e digitar
- Pode transferir arquivos
- Pode acessar o chat de texto
- Pode usar o clipboard compartilhado

---

## Desinstalação

### Desinstalar Host
1. Abra "Painel de Controle" > "Programas e Recursos"
2. Procure por "PrysmView Host"
3. Clique com botão direito > "Desinstalar"
4. Siga as instruções
5. O serviço será parado e removido
6. As chaves e logs permanecerão em `%ProgramData%\PrysmView\`

### Desinstalar Admin
1. Abra "Painel de Controle" > "Programas e Recursos"
2. Procure por "PrysmView Admin"
3. Clique com botão direito > "Desinstalar"
4. Siga as instruções

---

## Atualização

### Atualização Automática (Host)
O Host verifica automaticamente por atualizações:
- Verifica a cada 24 horas
- Baixa automaticamente quando disponível
- Verifica assinatura Ed25519
- Instala automaticamente
- Rollback em caso de falha

### Atualização Manual (Admin)
1. Baixe a nova versão do MSI
2. Execute o instalador
3. A versão anterior será substituída
4. Suas configurações serão preservadas

---

## Solução de Problemas

### Host não inicia
1. Verifique o Gerenciador de Tarefas > Serviços
2. Verifique logs em `%ProgramData%\PrysmView\logs\`
3. Verifique se a porta 8080 não está em uso
4. Reinstale se necessário

### Admin não conecta
1. Verifique sua conexão com internet
2. Verifique se o endpoint está correto: `https://prysmview.duckdns.org`
3. Verifique suas credenciais de login
4. Verifique se o servidor está online: `curl https://prysmview.duckdns.org/health`

### Erro de código de sessão
1. Verifique se o código está correto
2. Verifique se não expirou (120 segundos)
3. Verifique se não excedeu 3 tentativas
4. Peça um novo código no PDV

### Erro de certificado SSL
1. Verifique se o domínio está configurado corretamente
2. Verifique se o servidor Azure está rodando
3. Verifique se a data/hora do sistema está correta

---

## Suporte

Para mais informações:
- Documentação: `docs/`
- Release Notes: `Releases/README.md`
- Disaster Recovery: `docs/runbook-disaster-recovery.md`
