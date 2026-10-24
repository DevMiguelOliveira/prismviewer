# Threat Model - PrysmView

## Ativos
- Conexao remota entre PDV e servidor central
- Dados de transacao (vendas, pagamentos, fechamento de caixa)
- Credenciais de autenticacao do operador
- Sessao de controle remoto (teclado, mouse, tela)

## Ameacas
- Interceptacao de tráfego (MITM) na rede local da loja
- Acesso nao autorizado a sessao ativa do PDV
- Injecao de comandos remotos maliciosos
- Vazamento de dados sensiveis de pagamento

## Controles de seguranca
- Criptografia de canal (TLS/mTLS) para todo o trafego remoto
- Autenticacao multifator do operador antes de iniciar sessao
- Isolamento de sessao: cada conexao remota eh efemera e auditada
- Timeout automatico de sessao sem atividade
