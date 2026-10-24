# ADR 0001 - Estrutura do projeto PrysmView

## Status
Aceito

## Contexto
O PrysmView eh um software de acesso remoto para PDVs. Precisa de suporte a duas linguagens: Rust (cliente/agente no PDV) e Go (servidor/central).

## Decisao
- Crate Rust raiz em Cargo.toml para logica de cliente no PDV.
- Modulo Go em go.mod para logica de servidor central.
- Gates automáticos separados por linguagem.
- Orquestrador gerencia fila de tarefas via docs/plan/tasks.yaml.

## Consequencias
- Duas toolchains necessarias (Rust + Go).
- CI separada por linguagem, mas orquestrada por um unico script.
