# Guia de Provisionamento e Revogação de Técnicos via Microsoft Entra ID (D-06 / S-08 / S-09)

Este guia descreve o processo operacional padrão para cadastrar, habilitar, gerenciar e revogar o acesso de técnicos de suporte ao console administrativo **PrysmAdmin** utilizando autenticação corporativa via **Microsoft Entra ID (antigo Azure AD)**.

---

## 1. Arquitetura de Identidade

- **Provedor de Identidade (IdP):** Microsoft Entra ID (Microsoft 365 / Azure AD).
- **Protocolo:** OpenID Connect (OIDC) com PKCE.
- **Grupo de Segurança Central:** `prysmview-technicians`.
- **MFA:** Obrigatório via política de Acesso Condicional (Conditional Access) do tenant.
- **Provisionamento:** Just-In-Time (JIT) automático na API Go do PrysmView no primeiro login bem-sucedido.

---

## 2. Passo a Passo: Provisionar Novo Técnico

### Passo 1: Cadastro da Conta Corporativa no Tenant
O operador de TI deve possuir uma conta de usuário corporativa ativa no domínio da empresa (ex: `tecnico.suporte@empresa.com.br`).

### Passo 2: Atribuição ao Grupo de Segurança Autorizado
No Portal do Microsoft Entra ID:
1. Acesse **Entra ID** > **Grupos** > localize o grupo `prysmview-technicians`.
2. Clique em **Membros** > **Adicionar membros**.
3. Selecione o usuário técnico e confirme a inclusão.

*(Ou via PowerShell do Microsoft Graph:)*
```powershell
Add-MgGroupMember -GroupId "<OBJECT_ID_DO_GRUPO_PRYSMVIEW_TECHNICIANS>" -DirectoryObjectId "<USER_ID_DO_TECNICO>"
```

### Passo 3: Primeiro Acesso no PrysmAdmin
1. O técnico abre o executável `PrysmAdmin.exe`.
2. Na tela de login, clica em **"Entrar com Microsoft 365 (Entra ID)"**.
3. O navegador conclui o fluxo OIDC com validação de MFA (Microsoft Authenticator / FIDO2).
4. A API do PrysmView valida:
   - Assinatura criptográfica do token JWT.
   - Validade e emissor (`iss: https://login.microsoftonline.com/...`).
   - Presença da claim `groups` ou `roles` contendo o grupo autorizado `prysmview-technicians`.
5. O técnico recebe um token de sessão opaco e tem acesso liberado imediatamente aos PDVs das lojas autorizadas.

---

## 3. Passo a Passo: Revogação Imediata de Acesso (Desligamento / Mudança de Função)

Em caso de demissão, suspeita de comprometimento ou troca de setor, a revogação é central e com efeito em tempo real:

### Modo 1: Remoção do Grupo ou Desativação no Entra ID
1. No Entra ID, remova o usuário do grupo `prysmview-technicians` ou desative sua conta corporativa.
2. No próximo refresh de token (vida curta de 8 horas), o login será recusado com código HTTP 401:
   ```json
   {"error": "operador fora do grupo autorizado 'prysmview-technicians'"}
   ```

### Modo 2: Revogação Instantânea de Sessões Ativas (Kill-Switch em Tempo Real)
Para invalidar sessões e chamados em andamento sem aguardar a expiração do token:
1. O administrador aciona o endpoint de revogação da API:
   ```bash
   curl -X POST https://api.prysmview.com/api/v1/admin/revoke \
     -H "Authorization: Bearer <TOKEN_ADMIN>" \
     -d '{"id": "tecnico.suporte@empresa.com.br", "type": "operator", "reason": "Desligamento imediato"}'
   ```
2. A API insere o identificador na tabela `revocacoes`.
3. Todos os PDVs que executam o heartbeat a cada 30 segundos recebem a lista de revogações e encerram imediatamente qualquer sessão ativa ou ticket vinculado àquele técnico.
