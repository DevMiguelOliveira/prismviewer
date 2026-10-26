# ADR 0008: Modelo de Identidade por Chave Pública Ed25519 e Autenticação por Entra ID

- **Status:** Aprovado
- **Data:** 2026-10-25
- **Autores:** Arquiteto de Software & Engenheiro de Segurança PrysmView
- **Contexto:** S-07, S-08, S-09, Q-06 do Roadmap Master

---

## 1. Contexto e Problema

O ecossistema PrysmView atende a mais de 1.000 caixas de PDVs distribuídos em redes de franquias pelo Brasil. Esse ambiente possui características peculiares:
1. **PDVs descentralizados:** As máquinas das lojas franqueadas **não** fazem parte do Active Directory (AD) corporativo, e os operadores locais possuem privilégios de administrador local no caixa.
2. **Técnicos centralizados:** A equipe de suporte e técnicos de TI é composta por colaboradores da empresa com contas corporativas no Microsoft 365 (Entra ID).
3. **Risco de colisão de nomes:** Diferentes lojas podem nomear seus caixas de forma idêntica (ex: "Caixa 01", "PDV-01").
4. **Necessidade de desligamento imediato:** Se um técnico for desligado da empresa, seu acesso a todos os caixas da rede deve ser cancelado **em tempo real**, sem depender de intervenção nas máquinas físicas.

---

## 2. Decisão Arquitetural

### 2.1 Identidade Criptográfica do PDV (Host) — S-07
- **Chave Pública Ed25519:** A verdadeira e imutável identidade do PDV na rede é a chave pública Ed25519 gerada na primeira execução do serviço `prysmview-svc`.
- **Proteção da Chave Privada:** A chave privada nunca sai da máquina local e é cifrada no disco usando DPAPI no escopo de máquina (`CryptProtectData`), em diretório de sistema com ACL restrita a `SYSTEM` e Administradores.
- **Identificador de Exibição:** O identificador amigável (`PDV-101`, `Loja Centro`) é tratado estritamente como metadado. O banco SQLite na API impõe restrição de unicidade (`UNIQUE`) em `(store_id, pos_number)` e gera um ID curto de exibição globalmente único indexado pela chave pública.

### 2.2 Autenticação do Técnico via Microsoft Entra ID — S-08
- **Protocolo:** OpenID Connect (OIDC) com Authorization Code Flow + PKCE (RFC 7636).
- **Mapeamento de Grupos:** O acesso ao console `PrysmAdmin` exige que a conta do técnico pertença ao grupo corporativo de segurança:
  - `prysmview-technicians`: Acesso para visualização, controle remoto e chat com tickets válidos.
  - `prysmview-admins`: Gestão de frota, auditoria completa e emissão de permissões.
- **MFA Obrigatório:** A política de acesso condicional do tenant Microsoft 365 impõe autenticação multifator (MFA) sem exceção.
- **Eliminação de Senhas Locais:** Não existem senhas locais de técnicos salvas no banco da API. Uma única conta de emergência (*break-glass*) é permitida, protegida com hash `argon2id` (tempo 3, memória 64 MB, paralelismo 4) e mantida em cofre de segurança.

### 2.3 Revogação Imediata e Centralizada — S-09
- A remoção de um colaborador do grupo no Entra ID ou a desativação da sua conta de e-mail impede imediatamente a emissão de novos tokens de sinalização.
- Tokens de acesso à API possuem vida útil curta (máximo 8 horas) e são opacos com hash SHA-256 no banco.
- O endpoint de emissão de tickets valida em tempo real o status da conta do técnico no banco e no provedor.

---

## 3. Consequências

### Positivas
- **Impossibilidade de personificação de máquinas:** Nenhuma máquina consegue fingir ser outro PDV, pois cada requisição é assinada com a chave Ed25519 privada e validada pela chave pública.
- **Zero segredos estáticos no cliente:** Técnicos não possuem senhas salvas no aplicativo `PrysmAdmin`.
- **Desligamento em 1 clique:** Ao bloquear o técnico no Microsoft 365, todos os acessos a PDVs são revogados instantaneamente.
- **Auditoria vinculada à identidade real:** Os logs de auditoria registram o UPN (User Principal Name) do técnico autenticado pelo Entra ID (`nome.sobrenome@empresa.com.br`).

### Limitações e Mitigações
- **Operação sem conexão à internet do Admin:** O técnico precisa de conexão para autenticar no Entra ID. Caso a internet da central caia, utiliza-se a conta break-glass local sob protocolo de emergência documentado.
