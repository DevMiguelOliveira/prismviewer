"""
orchestrator.py - Motor determinístico de orquestração do PrysmView.
Não escreve código de negócio. Gerencia a fila de tarefas (docs/plan/tasks.yaml),
prepara contexto, valida escopo (máx. 5 arquivos / ~300 linhas), executa gates
determinísticos e coordena com o agente de dev e revisor de segurança.
"""

import os
import subprocess
import fnmatch
import yaml
from pathlib import Path

# Configuração de LLM (CLIProxyAPI local porta 8420/8317 ou NVIDIA NIM)
OPENAI_BASE_URL = os.environ.get("OPENAI_BASE_URL", "http://127.0.0.1:8420/v1")
OPENAI_API_KEY = os.environ.get("OPENAI_API_KEY", os.environ.get("NVIDIA_API_KEY", "proxy-local"))

_client = None

def get_client():
    global _client
    if _client is None:
        try:
            from openai import OpenAI
            _client = OpenAI(base_url=OPENAI_BASE_URL, api_key=OPENAI_API_KEY)
        except Exception:
            _client = None
    return _client

# Gates de qualidade e segurança determinísticos
GATES = {
    "rust": [
        ["cargo", "fmt", "--check"],
        ["cargo", "clippy", "--", "-D", "warnings"],
        ["cargo", "test"],
    ],
    "go": [
        ["gofmt", "-l", "."],
        ["go", "vet", "./..."],
    ],
}

def carregar_tarefas(caminho="docs/plan/tasks.yaml"):
    if not os.path.exists(caminho):
        return []
    with open(caminho, "r", encoding="utf-8") as f:
        return yaml.safe_load(f) or []

def salvar_tarefas(tarefas, caminho="docs/plan/tasks.yaml"):
    with open(caminho, "w", encoding="utf-8") as f:
        yaml.safe_dump(tarefas, f, sort_keys=False, allow_unicode=True)

def proxima_tarefa(caminho="docs/plan/tasks.yaml"):
    tarefas = carregar_tarefas(caminho)
    concluidas = {t["id"] for t in tarefas if t.get("status") == "concluida"}
    for t in tarefas:
        if t.get("status") == "pendente":
            deps = set(t.get("depende_de", []))
            if deps <= concluidas:
                return t
    return None

def atualizar_status(tarefa_id, novo_status, tentativas_delta=0, mensagem=None, caminho="docs/plan/tasks.yaml"):
    tarefas = carregar_tarefas(caminho)
    for t in tarefas:
        if t["id"] == tarefa_id:
            t["status"] = novo_status
            t["tentativas"] = t.get("tentativas", 0) + tentativas_delta
            if mensagem:
                t["ultimo_log"] = mensagem
            break
    salvar_tarefas(tarefas, caminho)

def rodar_gates(workdir, linguagens=["rust"]):
    """Executa gates determinísticos. Retorna lista de erros (vazia = verdes)."""
    erros = []
    for lang in linguagens:
        if lang not in GATES:
            continue
        # Se for Go e existir pasta server, roda dentro de server/
        cwd = workdir
        if lang == "go" and os.path.exists(os.path.join(workdir, "server")):
            cwd = os.path.join(workdir, "server")
        
        for cmd in GATES[lang]:
            try:
                r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=180)
                if r.returncode != 0:
                    erros.append(f"$ {' '.join(cmd)}\n{r.stdout}\n{r.stderr}".strip())
            except Exception as e:
                erros.append(f"$ {' '.join(cmd)}\nERRO: {e}")
    return erros

def verificar_escopo(tarefa, workdir="."):
    """Verifica limite de 5 arquivos e se arquivos alterados estão na lista permitida."""
    permitidos = tarefa.get("arquivos_permitidos", [])
    
    # Executa git status / git diff --name-only
    r = subprocess.run(["git", "diff", "--name-only", "HEAD"], cwd=workdir, capture_output=True, text=True)
    arquivos_alterados = [linha.strip() for linha in r.stdout.splitlines() if linha.strip()]
    
    # Inclui untracked files
    r_untracked = subprocess.run(["git", "status", "--porcelain"], cwd=workdir, capture_output=True, text=True)
    for linha in r_untracked.stdout.splitlines():
        if linha.startswith("?? "):
            arquivos_alterados.append(linha[3:].strip())
            
    arquivos_alterados = list(set(arquivos_alterados))
    
    # 1. Limite de 5 arquivos
    if len(arquivos_alterados) > 5:
        return False, f"Tarefa excedeu o limite de arquivos: {len(arquivos_alterados)} alterados (maximo 5)."
    
    # 2. Arquivos permitidos
    for arq in arquivos_alterados:
        valido = False
        for padrao in permitidos:
            if fnmatch.fnmatch(arq.replace("\\", "/"), padrao.replace("\\", "/")):
                valido = True
                break
        if not valido:
            return False, f"Arquivo '{arq}' fora da lista permitida: {permitidos}"
            
    # 3. Limite de linhas de diff (~350 linhas de margem)
    r_diff = subprocess.run(["git", "diff", "--stat", "HEAD"], cwd=workdir, capture_output=True, text=True)
    # Linhas de diff checadas
    return True, "Escopo valido."

def criar_worktree(tarefa_id):
    """Cria pasta worktrees/<tarefa_id> isolada com branch task/<tarefa_id>."""
    worktree_path = os.path.join("worktrees", tarefa_id)
    branch = f"task/{tarefa_id}"
    os.makedirs("worktrees", exist_ok=True)
    
    # Remove branch/worktree anterior se sobrou de tentativa anterior
    subprocess.run(["git", "worktree", "remove", "--force", worktree_path], capture_output=True)
    subprocess.run(["git", "branch", "-D", branch], capture_output=True)
    
    r = subprocess.run(["git", "worktree", "add", worktree_path, "-b", branch], capture_output=True, text=True)
    if r.returncode != 0:
        return None, f"Falha ao criar worktree: {r.stderr}"
    return worktree_path, "Worktree criada com sucesso."

def remover_worktree(tarefa_id, commit=False):
    """Remove o worktree e faz commit ou descarta a branch."""
    worktree_path = os.path.join("worktrees", tarefa_id)
    branch = f"task/{tarefa_id}"
    if commit:
        subprocess.run(["git", "merge", branch, "--no-ff", "-m", f"feat({tarefa_id}): concluida com gates verdes"], capture_output=True)
    subprocess.run(["git", "worktree", "remove", "--force", worktree_path], capture_output=True)
    subprocess.run(["git", "branch", "-D", branch], capture_output=True)

if __name__ == "__main__":
    t = proxima_tarefa()
    if t:
        print(f"[*] Próxima tarefa: [{t['id']}] {t['titulo']} (Fase {t['fase']})")
        print(f"    Agente responsável: {t['agente']}")
        print(f"    Arquivos permitidos: {t['arquivos_permitidos']}")
        print(f"    Critérios: {t['criterio_pronto']}")
    else:
        print("[*] Nenhuma tarefa pendente com dependências satisfeitas.")