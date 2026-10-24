# ADR 0003 — Código de Sessão de 8 Dígitos e Modelo de Consentimento

## Status
Aceito

## Contexto
O PDV de loja processa pagamentos e transações críticas. Nenhum acesso remoto pode ocorrer de forma invisível ou sem consentimento explícito, prevenindo que credenciais de operador vazadas permitam espionagem ou invasão de caixas.

## Decisão
Implementar um mecanismo de autenticação e consentimento de dois fatores em nível de aplicação:

1. **Janela de Consentimento no PDV:**
   - Ao receber pedido de sessão, o host aciona o aplicativo da bandeja (`prysmview-tray`) via named pipe local restrito.
   - Uma janela modal em primeiro plano exibe o nome do operador e o motivo declarado.
   - O usuário do caixa tem 60 segundos para clicar em **Aceitar** ou **Recusar**. Sem resposta em 60 segundos, o acesso é recusado automaticamente.
   - Usuários locais podem cadastrar operadores específicos em uma "lista de confiança" local para suporte recorrente autorizado.

2. **Código de Sessão Efêmero:**
   - Ao aceitar, o host gera um código numérico de **8 dígitos** aleatório (`OsRng`).
   - O código é exibido na tela do caixa para que o operador o digite no Viewer.
   - Validade máxima: **120 segundos**.
   - Tentativas máximas: **3 tentativas**. Após 3 erros, o código é invalidado imediatamente.
   - Validação criptográfica: O código é derivado via **HMAC-SHA256** e verificado usando comparação em tempo constante (`subtle::ConstantTimeEq`) para eliminar ataques de canal lateral (timing attacks).
   - Ao descartar o código, a memória RAM associada é limpa compulsoriamente via `zeroize`.

## Consequências
- Impossibilita conexões fantasma ou acessos remotos não autorizados.
- Proteção criptográfica robusta contra força bruta e timing attacks.
