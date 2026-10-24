import subprocess, os, yaml

# Cliente OpenAI opcional — só criado quando NVIDIA_API_KEY estiver configurado
client = None
def _get_client():
    global client
    if client is None and os.environ.get("NVIDIA_API_KEY"):
        try:
            from openai import OpenAI
            client = OpenAI(
                base_url="https://integrate.api.nvidia.com/v1",
                api_key=os.environ.get("NVIDIA_API_KEY"),
            )
        except ImportError:
            pass
    return client

GATES = {
    "rust": [
        ["cargo", "fmt", "--check"],
        ["cargo", "clippy", "--", "-D", "warnings"],
        ["cargo", "test"],
        ["cargo", "audit"],
    ],
    "go": [
        ["gofmt", "-l", "."],
        ["go", "vet", "./..."],
        ["govulncheck", "./..."],
        ["gosec", "./..."],
    ],
}

def rodar_gates(workdir, linguagens):
    """Executa gates determinísticos. Retorna lista de erros (lista vazia = verdes)."""
    erros = []
    for lang in linguagens:
        if lang not in GATES:
            continue
        for cmd in GATES[lang]:
            try:
                r = subprocess.run(cmd, cwd=workdir, capture_output=True, text=True, timeout=120)
                if r.returncode != 0:
                    erros.append(f"$ {' '.join(cmd)}\n{r.stdout}{r.stderr}")
            except Exception as e:
                erros.append(f"$ {' '.join(cmd)}\nERRO EXECUCAO: {e}")
    return erros  # lista vazia = gates verdes

def proxima_tarefa(caminho="docs/plan/tasks.yaml"):
    """Retorna a próxima tarefa pendente cujas dependências estão concluídas, ou None."""
    if not os.path.exists(caminho):
        return None
    with open(caminho, "r", encoding="utf-8") as f:
        tarefas = yaml.safe_load(f)
    if not tarefas:
        return None
    concluidas = {t["id"] for t in tarefas if t.get("status") == "concluida"}
    for t in tarefas:
        if t.get("status") == "pendente" and set(t.get("depende_de", [])) <= concluidas:
            return t
    return None

def chamar_agente(modelo, system, user):
    if not modelo:
        return None
    cli = _get_client()
    if not cli:
        return None
    r = cli.chat.completions.create(
        model=modelo,
        messages=[{"role": "system", "content": system},
                  {"role": "user", "content": user}],
        temperature=0.2,
    )
    return r.choices[0].message.content

def ciclo(tarefa, linguagens=["rust"]):
    """Executa um ciclo completo: gates -> se falhar devolve ao agente (max 3) -> se passar -> revisor.
    Retorna status final: 'gates_verdes', 'bloqueada' ou 'revisao_pendente'."""
    tentativas = 0
    while tentativas < 3:
        erros = rodar_gates("worktrees/" + tarefa["id"], linguagens)
        if not erros:
            return "gates_verdes"
        tentativas += 1
        # Devolve somente o erro ao agente de desenvolvimento (sem comentários de nossa parte)
        # Aqui o orquestrador entraria com chamar_agente, mas para o teste simples não fazemos isso
        # Na prática, após receber corrigido, voltamos ao loop
    return "bloqueada"

# Bloco de teste se rodado como script principal
if __name__ == "__main__":
    # Teste rápido: tenta proxima_tarefa
    t = proxima_tarefa()
    if t:
        print(f"Proxima tarefa: {t['id']} - {t['titulo']} (fase {t['fase']})")
    else:
        print("Nenhuma tarefa pendente com dependencias satisfeitas.")
    # Teste de gates: verifica se o script encontra o cargo disponível
    errs = rodar_gates(".", ["rust"])
    if errs:
        print(f"Gates encontrados {len(errs)} erro(s):")
        for e in errs[:3]:
            print(e[:120])
    else:
        print("Gates rust passaram (ou cargo nao configurado).")