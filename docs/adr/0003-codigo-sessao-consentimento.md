# ADR 0003 - Código de sessão e modelo de consentimento

## Status

Aceito

## Contexto

O PDV exige que qualquer sessão remota inicie com consentimento visível do usuário local, exceto para operadores já marcados como confiáveis. O fluxo é: o operador solicita uma sessão; o host exibe uma janela de consentimento no PDV com nome do operador, motivo informado e botões Aceitar e Recusar, com timeout de 60 segundos que resulta em recusa. Se aceito, o host gera um código de sessão de 8 dígitos e o exibe na própria tela do PDV. O operador digita esse código no viewer. A verificação ocorre no host via HMAC-SHA256 com comparação em tempo constante. O código é válido por 120 segundos, com no máximo 3 tentativas.

## Decisão

- Código de sessão de 8 dígitos numéricos, gerado via HMAC-SHA256(chave_sessão, timestamp), truncado para 8 dígitos decimais.
- Validação no host com `subtle::ConstantTimeEq` (comparação em tempo constante), impedindo ataques de tempo.
- Validade de 120 segundos e limite de 3 tentativas por código.
- Lista de operadores confiáveis, definida pelo usuário local no PDV, que pula a janela de consentimento.
- Geração e validação implementadas no `prysmview-svc`; exibição da janela de consentimento e indicador de sessão no `prysmview-tray`, via pipe local com ACL restrita ao usuário logado e SYSTEM.
- Auditoria registra o resultado do consentimento e as tentativas de código, sem gravar tela, teclas ou o próprio código.

## Justificativa de segurança

- 8 dígitos numéricos = 100 milhões de combinações possíveis.
- 120 segundos de validade somados a no máximo 3 tentativas reduzem drasticamente a janela para brute-force online.
- HMAC-SHA256 impede que um atacante preveja códigos futuros sem a chave de sessão.
- Comparação em tempo constante (`subtle::ConstantTimeEq`) elimina timing attacks na validação.
- A chave de sessão é protegida em disco por DPAPI (escopo de máquina) e nunca deixa o host.
- Consentimento visível no PDV impede acesso furtivo; o usuário local pode recusar ou encerrar a sessão a qualquer momento.

## Consequências

- Implementação dividida entre `prysmview-svc` (geração, validação HMAC, aplicação do limite de tentativas e expiração) e `prysmview-tray` (janela de consentimento, indicador de sessão, botão de encerramento).
- Comunicação serviço-tray por named pipe local com ACL restrita ao usuário logado e SYSTEM.
- Logs de auditoria incluem operador, PDV, horário, resultado do consentimento e quantidade de tentativas de código; nunca registram conteúdo de tela, teclas ou o código em si.
- Dependências adicionais: `hmac`, `sha2`, `subtle` e `rand` (já previstas na seção 6.1 do AGENTS.md).
- Timeout de 60 segundos na janela de consentimento pode gerar recusa por inatividade do usuário local, protegendo contra sessões esquecidas.

## Alternativas consideradas

- Código alfanumérico maior: rejeitado, pois dificulta a digitação pelo operador no viewer e não acrescenta segurança suficiente dada a janela de 120s e 3 tentativas.
- Código gerado puro aleatório sem HMAC: rejeitado, pois seria previsível se o gerador fosse comprometido e não oferece bind com a sessão.
- Consentimento apenas remoto (no viewer): rejeitado, viola o requisito de consentimento visível no PDV (seção 9).
