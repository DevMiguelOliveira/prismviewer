# MIGRATION.md - Migração para React + Tailwind + Tauri

## Visão Geral

Este documento descreve a migração do PrysmView da arquitetura original (Rust + eframe/egui) para a nova arquitetura (React + Tailwind + Tauri).

## Motivação

A migração para React + Tailwind + Tauri foi motivada por:

1. **Desenvolvimento mais rápido**: React e Tailwind CSS são tecnologias web modernas com ecossistema rico e curva de aprendizado suave.
2. **UI/UX mais flexível**: Tailwind CSS permite um design system customizado e responsivo com menos esforço.
3. **Componentes reutilizáveis**: React facilita a criação de componentes compartilhados entre Host e Admin.
4. **Manutenibilidade**: TypeScript e React tornam o código mais legível e manutenível.
5. ** Comunidade**: Grande comunidade e disponibilidade de bibliotecas para UI/UX.

## Mudanças Principais

### Stack Tecnológica

| Antigo (v1.0) | Novo (v2.0) |
|--------------|-------------|
| Rust + eframe/egui | React + TypeScript + Tailwind CSS + Tauri |
| HTML/CSS/JavaScript vanilla | React components com TypeScript |
| iroh/QUIC para P2P | WebRTC (TURN/STUN) |
| Oracle Cloud Always Free | Azure VM (com créditos) |
| Go para sinalização | Go ou Node.js (a definir) |

### Estrutura de Arquivos

**Antigo:**
```
prysmview/
├── crates/
│ ├── core/
│ ├── host/
│ ├── tray/
│ ├── viewer/
│ └── relay/
├── server/ (Go)
└── crates/viewer/ui/ (HTML/CSS/JS vanilla)
```

**Novo:**
```
prysmview/
├── apps/
│ ├── host/ (Tauri app)
│ └── admin/ (Tauri app)
├── packages/
│ ├── ui/ (Componentes compartilhados)
│ ├── config/ (Configurações compartilhadas)
│ └── types/ (Tipos compartilhados)
├── server/ (Go/Node.js)
└── docs/
```

### Interface do Usuário

**Antigo:**
- HTML/CSS/JavaScript vanilla em `crates/viewer/ui/`
- Sem gerenciamento de estado formal
- Navegação manual por tabs
- Botões com manipuladores inline

**Novo:**
- React components com TypeScript
- Zustand para estado global
- React Router para navegação
- Componentes reutilizáveis do pacote `@prysmview/ui`
- Design system consistente com Tailwind CSS

## Passos da Migração

### Fase 1: Setup Inicial (✅ Automatizado)

O script `setup_prysmview.ps1` automatiza:

1. ✅ Criação da estrutura de diretórios
2. ✅ Inicialização do monorepo com pnpm workspace
3. ✅ Configuração do Turbo para orquestração de builds
4. ✅ Inicialização dos apps Host e Admin com Tauri
5. ✅ Criação do pacote de componentes compartilhados
6. ✅ Instalação de dependências

**Comando:**
```powershell
.\setup_prysmview.ps1
```

### Fase 2: Migração da UI

#### 2.1 Design System

1. Converter o CSS atual em `crates/viewer/ui/style.css` para Tailwind CSS
2. Criar variáveis CSS customizadas para tema claro/escuro
3. Definir paleta de cores baseada no design atual (Cyber Slate)
4. Criar componentes base no pacote `@prysmview/ui`:
   - Button (com variantes: primary, secondary, ghost, danger)
   - Input (com validação)
   - Tabs (com navegação corrigida)
   - Modal
   - Sidebar
   - Header

#### 2.2 Navegação por Tabs

**Problema no código atual:**
- Navegação manual por classes CSS
- Estado não gerenciado de forma centralizada
- Falhas na renderização ao trocar de aba

**Solução React:**
```typescript
// Usando Radix UI Tabs + Zustand
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@radix-ui/react-tabs'
import { useTabStore } from '../stores/tabStore'

function App() {
  const { activeTab, setActiveTab } = useTabStore()

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab}>
      <TabsList>
        <TabsTrigger value="dashboard">Dispositivos</TabsTrigger>
        <TabsTrigger value="direct">Conexão Rápida</TabsTrigger>
        <TabsTrigger value="stores">Grupos & Clientes</TabsTrigger>
      </TabsList>
      <TabsContent value="dashboard">
        <DashboardView />
      </TabsContent>
      <TabsContent value="direct">
        <DirectConnectionView />
      </TabsContent>
      <TabsContent value="stores">
        <StoresView />
      </TabsContent>
    </Tabs>
  )
}
```

#### 2.3 Botões e Manipuladores de Eventos

**Problema no código atual:**
- Manipuladores inline no HTML
- Estados visuais (hover, focus, disabled) inconsistentes
- Falta de feedback visual (loading states)

**Solução React:**
```typescript
import { Button } from '@prysmview/ui'

function ConnectionForm() {
  const [loading, setLoading] = useState(false)
  const [partnerId, setPartnerId] = useState('')

  const handleConnect = async () => {
    setLoading(true)
    try {
      await connectToHost(partnerId)
    } catch (error) {
      console.error('Connection failed:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Button
      variant="primary"
      loading={loading}
      disabled={!partnerId || loading}
      onClick={handleConnect}
    >
      Conectar
    </Button>
  )
}
```

### Fase 3: Backend Rust (Tauri)

O backend Rust migra de crates separados para o backend do Tauri:

**Antigo:**
- `crates/host/` - serviço Windows
- `crates/viewer/` - viewer Rust
- `crates/core/` - tipos compartilhados

**Novo:**
- `apps/host/src-tauri/src/` - backend do Host
- `apps/admin/src-tauri/src/` - backend do Admin
- `packages/types/src/` - tipos TypeScript compartilhados

**Funcionalidades a migrar:**
1. Captura de tela (DXGI Desktop Duplication)
2. Injeção de entrada (SendInput)
3. Criptografia (Ed25519, HMAC-SHA256)
4. WebRTC (via Tauri IPC commands)
5. Serviço Windows (via Tauri system tray)

### Fase 4: Protocolo de Comunicação

**Antigo:**
- iroh/QUIC com cabeçalho customizado
- Streams binários com postcard

**Novo:**
- WebRTC data channels
- Mensagens JSON ou binárias (a definir)

**Migração do protocolo:**
```typescript
// Antigo (Rust)
struct FrameHeader {
    magic: [u8; 4],
    version: u16,
    msg_type: u8,
    payload_size: u32,
}

// Novo (TypeScript)
interface FrameHeader {
    magic: string, // "PRSM"
    version: number,
    msgType: number,
    payloadSize: number,
}
```

### Fase 5: Backend de Sinalização

**Antigo:**
- Go com chi router
- SQLite com modernc.org/sqlite

**Novo:**
- Opção A: Go (manter consistência)
- Opção B: Node.js (alinhado com stack frontend)

**Decisão:** A definir em ADR 0002.

## Compatibilidade

### Arquivos Mantidos

- `docs/PROJETO.md` - documentação geral (atualizada)
- `docs/threat-model.md` - modelo de ameaças (mantido)
- `docs/protocolo.md` - protocolo (adaptado para WebRTC)
- `server/` - backend Go (mantido ou migrado para Node.js)

### Arquivos Removidos

- `crates/` - estrutura Rust antiga
- `crates/viewer/ui/` - UI HTML/CSS/JS vanilla
- `Cargo.toml` - workspace Rust antigo
- `rust-toolchain.toml` - versão Rust fixa (agora via Tauri)

### Arquivos Novos

- `apps/` - estrutura de apps Tauri
- `packages/` - pacotes compartilhados
- `pnpm-workspace.yaml` - workspace pnpm
- `turbo.json` - orquestração de builds
- `setup_prysmview.ps1` - script de setup
- `build_prysmview.ps1` - script de build
- `AGENTS.md` - regras atualizadas para React/Tailwind

## Checklist de Migração

- [x] Criar novo AGENTS.md
- [x] Criar script de setup
- [x] Criar script de build
- [x] Criar documento de migração
- [ ] Executar setup_prysmview.ps1
- [ ] Migrar CSS para Tailwind
- [ ] Criar componentes base no pacote @prysmview/ui
- [ ] Migrar navegação por tabs para React
- [ ] Migrar botões e manipuladores para React
- [ ] Migrar backend Rust para Tauri
- [ ] Adaptar protocolo para WebRTC
- [ ] Migrar backend de sinalização (se necessário)
- [ ] Testar builds de produção
- [ ] Testar instaladores Tauri
- [ ] Atualizar documentação

## Riscos e Mitigações

### Risco 1: Perda de performance

**Descrição:** React pode ser mais lento que eframe/egui para renderização de vídeo em tempo real.

**Mitigação:**
- Usar React apenas para UI de controle
- Renderização de vídeo via canvas direto (WebRTC video element)
- Otimizar re-renders com React.memo e useMemo
- Considerar usar Rust para renderização crítica via Tauri IPC

### Risco 2: Complexidade aumentada

**Descrição:** Adicionar React + TypeScript aumenta a complexidade do stack.

**Mitigação:**
- Manter estrutura monorepo simples
- Usar componentes compartilhados para evitar duplicação
- Documentar convenções de código
- Usar TypeScript strict mode para segurança de tipos

### Risco 3: Curva de aprendizado

**Descrição:** Equipe precisa aprender React, Tailwind e Tauri.

**Mitigação:**
- Documentação detalhada em AGENTS.md
- Componentes prontos no pacote @prysmview/ui
- Scripts automatizados para setup e build
- Exemplos de código no docs/

## Timeline Estimada

- **Fase 1 (Setup):** 1 dia (automatizado)
- **Fase 2 (UI):** 3-5 dias
- **Fase 3 (Backend Rust):** 5-7 dias
- **Fase 4 (Protocolo):** 2-3 dias
- **Fase 5 (Sinalização):** 2-3 dias
- **Testes e ajustes:** 3-5 dias

**Total:** 16-24 dias de trabalho

## Suporte

Para dúvidas sobre a migração:

1. Consulte [AGENTS.md](../AGENTS.md) para regras e convenções
2. Consulte [docs/PROJETO.md](PROJETO.md) para visão geral
3. Abra uma issue no GitHub para perguntas específicas

## Referências

- [React Documentation](https://react.dev/)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [Tauri Documentation](https://tauri.app/)
- [Radix UI](https://www.radix-ui.com/)
- [Zustand](https://zustand-demo.pmnd.rs/)
