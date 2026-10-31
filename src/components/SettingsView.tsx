import React, { useState } from 'react';
import { Settings } from '../types';
import {
  Monitor,
  Shield,
  Sliders,
  Laptop,
  Check,
  RefreshCw,
  Eye,
  EyeOff,
  Lightbulb,
  CheckCircle2,
  FileText,
  User,
  Zap,
} from 'lucide-react';

interface SettingsViewProps {
  settings: Settings;
  onUpdateSettings: (newSettings: Settings) => void;
  onSendNotification: (msg: string, type?: 'success' | 'warning' | 'error') => void;
  onOpenDiagnostics?: () => void;
}

type TabType = 'geral' | 'seguranca' | 'controle' | 'avancado';

export const SettingsView: React.FC<SettingsViewProps> = ({
  settings,
  onUpdateSettings,
  onSendNotification,
  onOpenDiagnostics,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('geral');
  const [formData, setFormData] = useState<Settings>({
    ...settings,
    contactName: settings.contactName || 'NLUISOLIVEIRA',
    startWithWindows: settings.startWithWindows ?? true,
    theme: settings.theme || 'dark',
    closeToTray: settings.closeToTray ?? true,
    lanConnections: settings.lanConnections || 'deactivated',
    displayQuality: settings.displayQuality || 'speed',
    defaultScale: settings.defaultScale || 'fit',
    removeRemoteWallpaper: settings.removeRemoteWallpaper ?? true,
    sendKeyCombinations: settings.sendKeyCombinations ?? true,
    blockClientInput: settings.blockClientInput ?? false,
    accessControl: settings.accessControl || 'full',
    requireConsent: settings.requireConsent ?? true,
    unattendedPassword: settings.unattendedPassword || '••••••••',
    logRetentionDays: settings.logRetentionDays || 30,
    cleanTempOnExit: settings.cleanTempOnExit ?? true,
  });

  const [showPassword, setShowPassword] = useState(false);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [updateMessage, setUpdateMessage] = useState<string | null>(null);

  const handleApply = () => {
    onUpdateSettings(formData);
    onSendNotification('Configurações salvas e aplicadas com sucesso!', 'success');
  };

  const handleCancel = () => {
    setFormData({
      ...settings,
      contactName: settings.contactName || 'NLUISOLIVEIRA',
    });
    onSendNotification('Alterações descartadas.', 'warning');
  };

  const handleCheckUpdate = () => {
    setIsCheckingUpdate(true);
    setUpdateMessage(null);
    setTimeout(() => {
      setIsCheckingUpdate(false);
      setUpdateMessage('Você já está executando a versão mais recente (PrysmView v2.0.0 estável).');
      onSendNotification('PrysmView está atualizado.', 'success');
    }, 1200);
  };

  const tabs: { id: TabType; label: string }[] = [
    { id: 'geral', label: 'Geral' },
    { id: 'seguranca', label: 'Segurança' },
    { id: 'controle', label: 'Controle remoto' },
    { id: 'avancado', label: 'Avançado' },
  ];

  return (
    <div className="max-w-4xl mx-auto animate-in fade-in duration-200">
      {/* TeamViewer Style Dialog Container */}
      <div className="bg-slate-900 border border-slate-700/80 rounded-xl shadow-2xl overflow-hidden flex flex-col text-xs text-slate-200">
        
        {/* Title Bar (Estilo Janela Nativa TeamViewer) */}
        <div className="bg-slate-950 px-4 py-2.5 border-b border-slate-800 flex items-center justify-between select-none">
          <div className="flex items-center gap-2">
            <div className="w-5 h-5 rounded-full bg-sky-600 flex items-center justify-center text-white">
              <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none">
                <path d="M7 16V4M7 4L3 8M7 4L11 8M17 8V20M17 20L21 16M17 20L13 16" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </div>
            <span className="font-semibold text-slate-200 text-xs">Opções PrysmView</span>
          </div>
          <span className="text-[11px] text-slate-500 font-mono">v2.0.0</span>
        </div>

        {/* Dialog Body (Sidebar + Content Panel) */}
        <div className="flex flex-col md:flex-row min-h-[500px]">
          
          {/* Left Sidebar Menu */}
          <div className="w-full md:w-56 bg-slate-950/60 border-b md:border-b-0 md:border-r border-slate-800 flex flex-col shrink-0 select-none py-2">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                type="button"
                onClick={() => setActiveTab(tab.id)}
                className={`w-full text-left px-4 py-2.5 text-xs font-medium transition-colors flex items-center justify-between ${
                  activeTab === tab.id
                    ? 'bg-[#0078d4] text-white font-semibold'
                    : 'text-slate-300 hover:bg-slate-800/60 hover:text-white'
                }`}
              >
                <span>{tab.label}</span>
                {activeTab === tab.id && <span className="w-1.5 h-1.5 rounded-full bg-white"></span>}
              </button>
            ))}
          </div>

          {/* Right Content Panel */}
          <div className="flex-1 p-5 md:p-6 overflow-y-auto space-y-5 bg-[#0b101c]">
            
            {/* ==============================================================
                ABA: GERAL
               ============================================================== */}
            {activeTab === 'geral' && (
              <div className="space-y-5 animate-in fade-in duration-150">
                {/* Dica do TeamViewer */}
                <div className="flex items-center gap-2 text-[11px] text-sky-300 bg-sky-950/40 border border-sky-800/50 p-2.5 rounded-lg">
                  <Lightbulb className="w-4 h-4 text-sky-400 shrink-0" />
                  <span>Passe o mouse sobre as opções ou campos para obter orientações de suporte.</span>
                </div>

                {/* Seção: Opções importantes */}
                <div className="space-y-3">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Opções importantes para trabalhar com o PrysmView
                  </div>

                  <div className="space-y-2.5 px-1">
                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Seu nome de contato:</label>
                      <div className="sm:col-span-2 relative">
                        <input
                          type="text"
                          value={formData.contactName}
                          onChange={(e) => setFormData({ ...formData, contactName: e.target.value })}
                          placeholder="Ex: NLUISOLIVEIRA"
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white focus:outline-none focus:border-[#0078d4] transition-colors"
                        />
                      </div>
                    </div>

                    <label className="flex items-center gap-2 cursor-pointer pt-1">
                      <input
                        type="checkbox"
                        checked={formData.startWithWindows}
                        onChange={(e) => setFormData({ ...formData, startWithWindows: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200 font-medium">Iniciar o PrysmView com o Windows</span>
                    </label>

                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2 pt-1">
                      <label className="text-slate-300 font-medium">Escolha um tema:</label>
                      <div className="sm:col-span-2">
                        <select
                          value={formData.theme}
                          onChange={(e) => setFormData({ ...formData, theme: e.target.value as 'dark' | 'light' })}
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white focus:outline-none focus:border-[#0078d4]"
                        >
                          <option value="dark">Escuro</option>
                          <option value="light">Claro</option>
                        </select>
                      </div>
                    </div>

                    <label className="flex items-center gap-2 cursor-pointer pt-1">
                      <input
                        type="checkbox"
                        checked={formData.closeToTray}
                        onChange={(e) => setFormData({ ...formData, closeToTray: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200">Minimizar para a área de notificação do Windows ao fechar</span>
                    </label>
                  </div>
                </div>

                {/* Seção: Configurações de rede */}
                <div className="space-y-3 pt-2">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Configurações de rede local
                  </div>

                  <div className="space-y-2.5 px-1">
                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <span className="text-slate-300 font-medium">Porta local de serviço:</span>
                      <div className="sm:col-span-2 flex items-center gap-2">
                        <span className="font-mono text-white bg-slate-950 px-3 py-1.5 rounded border border-slate-800">
                          TCP / UDP 19533
                        </span>
                        <span className="text-[11px] text-emerald-400 font-mono flex items-center gap-1">
                          <span className="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                          Ativo
                        </span>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Conexões LAN recebidas:</label>
                      <div className="sm:col-span-2">
                        <select
                          value={formData.lanConnections}
                          onChange={(e) => setFormData({ ...formData, lanConnections: e.target.value as 'deactivated' | 'accept' })}
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white focus:outline-none focus:border-[#0078d4]"
                        >
                          <option value="deactivated">desativado</option>
                          <option value="accept">aceitar conexões diretas na rede local</option>
                        </select>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Modo de streaming:</label>
                      <div className="sm:col-span-2">
                        <select
                          value={formData.displayQuality}
                          onChange={(e) => setFormData({ ...formData, displayQuality: e.target.value as 'auto' | 'speed' | 'quality' })}
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white focus:outline-none focus:border-[#0078d4]"
                        >
                          <option value="speed">Otimizar velocidade (Fluidez 60 FPS com baixa latência)</option>
                          <option value="quality">Otimizar qualidade (Alta Definição HD)</option>
                          <option value="auto">Automático (Adaptar conforme largura de banda)</option>
                        </select>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Seção: Atribuição da conta */}
                <div className="space-y-3 pt-2">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Atribuição do dispositivo ao Console
                  </div>

                  <p className="text-slate-400 text-[11px] px-1">
                    Ao atribuir este dispositivo a uma conta técnica PrysmView, ele poderá ser gerenciado e monitorado remotamente em qualquer localidade.
                  </p>

                  <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between">
                    <div>
                      <span className="text-[10px] text-slate-400 uppercase tracking-wider block">Técnico Vinculado</span>
                      <strong className="text-white text-xs">{formData.contactName} (TECH-01)</strong>
                    </div>
                    <span className="px-2.5 py-1 rounded bg-emerald-950/80 text-emerald-400 border border-emerald-800 text-[10px] font-mono">
                      Autenticado
                    </span>
                  </div>
                </div>
              </div>
            )}

            {/* ==============================================================
                ABA: SEGURANÇA
               ============================================================== */}
            {activeTab === 'seguranca' && (
              <div className="space-y-5 animate-in fade-in duration-150">
                <div className="space-y-3">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Controle de Acesso às Sessões Remotas
                  </div>

                  <div className="space-y-2.5 px-1">
                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Regra de Acesso Padrão:</label>
                      <div className="sm:col-span-2">
                        <select
                          value={formData.accessControl}
                          onChange={(e) => setFormData({ ...formData, accessControl: e.target.value as 'full' | 'confirm' | 'view' })}
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white focus:outline-none focus:border-[#0078d4]"
                        >
                          <option value="full">Acesso total (Controle de mouse, teclado e arquivos)</option>
                          <option value="confirm">Confirmar tudo (Exigir confirmação manual para cada comando)</option>
                          <option value="view">Apenas visualizar (Modo observador sem interação remota)</option>
                        </select>
                      </div>
                    </div>

                    <label className="flex items-center gap-2 cursor-pointer pt-1">
                      <input
                        type="checkbox"
                        checked={formData.requireConsent}
                        onChange={(e) => setFormData({ ...formData, requireConsent: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200 font-medium">
                        Exigir consentimento local na tela do computador antes de iniciar a sessão (tempo limite de 60s)
                      </span>
                    </label>
                  </div>
                </div>

                {/* Seção: Acesso Desassistido */}
                <div className="space-y-3 pt-2">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Acesso Desassistido (Senha Fixa)
                  </div>

                  <p className="text-slate-400 text-[11px] px-1">
                    Configure a senha permanente para permitir conexões de suporte fora do horário comercial sem necessidade de alguém na máquina remota:
                  </p>

                  <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2 px-1">
                    <label className="text-slate-300 font-medium">Senha permanente:</label>
                    <div className="sm:col-span-2 relative">
                      <input
                        type={showPassword ? 'text' : 'password'}
                        value={formData.unattendedPassword}
                        onChange={(e) => setFormData({ ...formData, unattendedPassword: e.target.value })}
                        className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 pr-10 text-xs font-mono text-white focus:outline-none focus:border-[#0078d4]"
                      />
                      <button
                        type="button"
                        onClick={() => setShowPassword(!showPassword)}
                        className="absolute inset-y-0 right-0 pr-3 flex items-center text-slate-400 hover:text-white"
                      >
                        {showPassword ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                      </button>
                    </div>
                  </div>

                  <div className="p-2.5 rounded bg-slate-950 border border-slate-800 text-[11px] text-slate-400">
                    <span className="text-emerald-400 font-semibold">Proteção Criptográfica Ativa:</span> A senha é gravada de forma criptografada pelo serviço nativo através da DPAPI (Data Protection API) do Windows em escopo de máquina LocalSystem.
                  </div>
                </div>
              </div>
            )}

            {/* ==============================================================
                ABA: CONTROLE REMOTO
               ============================================================== */}
            {activeTab === 'controle' && (
              <div className="space-y-5 animate-in fade-in duration-150">
                <div className="space-y-3">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Parâmetros de Exibição da Tela
                  </div>

                  <div className="space-y-2.5 px-1">
                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Escala padrão de exibição:</label>
                      <div className="sm:col-span-2">
                        <select
                          value={formData.defaultScale}
                          onChange={(e) => setFormData({ ...formData, defaultScale: e.target.value as 'fit' | 'original' })}
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white focus:outline-none focus:border-[#0078d4]"
                        >
                          <option value="fit">Ajustar à tela do operador (Fit to Screen)</option>
                          <option value="original">Tamanho original 1:1 com rolagem fluida</option>
                        </select>
                      </div>
                    </div>

                    <label className="flex items-center gap-2 cursor-pointer pt-1">
                      <input
                        type="checkbox"
                        checked={formData.removeRemoteWallpaper}
                        onChange={(e) => setFormData({ ...formData, removeRemoteWallpaper: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200">
                        Ocultar papel de parede remoto durante o acesso (economiza até 40% de banda de internet)
                      </span>
                    </label>
                  </div>
                </div>

                {/* Seção: Interação e Periféricos */}
                <div className="space-y-3 pt-2">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Interação do Operador & Periféricos
                  </div>

                  <div className="space-y-2 px-1">
                    <label className="flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={formData.sendKeyCombinations}
                        onChange={(e) => setFormData({ ...formData, sendKeyCombinations: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200">
                        Transmitir combinações de teclas do sistema (Alt+Tab, Tecla Windows, Win+R)
                      </span>
                    </label>

                    <label className="flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={formData.clipboardSync}
                        onChange={(e) => setFormData({ ...formData, clipboardSync: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200">
                        Sincronização bidirecional da área de transferência (Copiar e colar texto &lt; 1 MB)
                      </span>
                    </label>

                    <label className="flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={formData.fileTransferEnabled}
                        onChange={(e) => setFormData({ ...formData, fileTransferEnabled: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200">
                        Habilitar transferência segura de arquivos com verificação obrigatória de hash SHA-256
                      </span>
                    </label>

                    <label className="flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={formData.blockClientInput}
                        onChange={(e) => setFormData({ ...formData, blockClientInput: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200">
                        Permitir bloqueio de entrada local (mouse e teclado do usuário cliente durante intervenções críticas)
                      </span>
                    </label>
                  </div>
                </div>
              </div>
            )}

            {/* ==============================================================
                ABA: AVANÇADO
               ============================================================== */}
            {activeTab === 'avancado' && (
              <div className="space-y-5 animate-in fade-in duration-150">
                {/* Seção: Rede e Sinalização Nuvem Azure */}
                <div className="space-y-3">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4] flex items-center justify-between">
                    <span>Rede e Sinalização na Nuvem Azure (Produção Real)</span>
                    <span className="text-[10px] text-emerald-400 font-mono font-normal">VM Azure 20.121.137.44</span>
                  </div>

                  <div className="space-y-3 px-1">
                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Servidor de Sinalização:</label>
                      <div className="sm:col-span-2">
                        <input
                          type="text"
                          value={formData.apiEndpoint}
                          onChange={(e) => setFormData({ ...formData, apiEndpoint: e.target.value })}
                          placeholder="https://prysmview.duckdns.org"
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white font-mono focus:outline-none focus:border-[#0078d4]"
                        />
                        <span className="text-[10px] text-slate-400 block mt-1">
                          Terminal Caddy reverso com TLS 1.3 Let&apos;s Encrypt ativo na VM Azure.
                        </span>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Servidor STUN (NAT Discovery):</label>
                      <div className="sm:col-span-2">
                        <input
                          type="text"
                          value={formData.stunServer}
                          onChange={(e) => setFormData({ ...formData, stunServer: e.target.value })}
                          placeholder="stun.l.google.com:19302"
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white font-mono focus:outline-none focus:border-[#0078d4]"
                        />
                      </div>
                    </div>

                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Servidor TURN / Coturn:</label>
                      <div className="sm:col-span-2">
                        <input
                          type="text"
                          value={formData.turnServer}
                          onChange={(e) => setFormData({ ...formData, turnServer: e.target.value })}
                          placeholder="turn:20.121.137.44:3478"
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white font-mono focus:outline-none focus:border-[#0078d4]"
                        />
                        <span className="text-[10px] text-slate-400 block mt-1">
                          Relay P2P DTLS criptografado para transposição de firewalls corporativos.
                        </span>
                      </div>
                    </div>

                    <div className="pt-1 flex items-center gap-2">
                      {onOpenDiagnostics && (
                        <button
                          type="button"
                          onClick={onOpenDiagnostics}
                          className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-sky-400 border border-sky-800/50 rounded font-semibold text-xs flex items-center gap-1.5 transition-colors"
                        >
                          <Zap className="w-3.5 h-3.5" />
                          <span>Executar Diagnóstico WebRTC & Sinalização</span>
                        </button>
                      )}
                    </div>
                  </div>
                </div>

                <div className="space-y-3">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Auditoria e Conformidade de Segurança (AGENTS.md)
                  </div>

                  <div className="space-y-2.5 px-1">
                    <div className="grid grid-cols-1 sm:grid-cols-3 items-center gap-2">
                      <label className="text-slate-300 font-medium">Retenção de logs no disco:</label>
                      <div className="sm:col-span-2">
                        <select
                          value={formData.logRetentionDays}
                          onChange={(e) => setFormData({ ...formData, logRetentionDays: Number(e.target.value) })}
                          className="w-full bg-slate-950 border border-slate-700 rounded px-3 py-1.5 text-xs text-white focus:outline-none focus:border-[#0078d4]"
                        >
                          <option value={30}>30 dias de histórico (Padrão)</option>
                          <option value={60}>60 dias de histórico</option>
                          <option value={90}>90 dias de histórico corporativo</option>
                        </select>
                      </div>
                    </div>

                    <label className="flex items-center gap-2 cursor-pointer pt-1">
                      <input
                        type="checkbox"
                        checked={formData.cleanTempOnExit}
                        onChange={(e) => setFormData({ ...formData, cleanTempOnExit: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200">
                        Excluir automaticamente arquivos temporários e blocos parciais ao encerrar a sessão
                      </span>
                    </label>

                    <label className="flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={formData.notifications}
                        onChange={(e) => setFormData({ ...formData, notifications: e.target.checked })}
                        className="rounded bg-slate-950 border-slate-700 text-[#0078d4] focus:ring-0"
                      />
                      <span className="text-slate-200">
                        Emitir alertas visuais na barra de tarefas ao iniciar e finalizar sessões de suporte
                      </span>
                    </label>
                  </div>
                </div>

                {/* Seção: Atualização do Software */}
                <div className="space-y-3 pt-2">
                  <div className="bg-slate-800/80 px-3 py-1.5 rounded text-[11px] font-bold text-slate-200 uppercase tracking-wide border-l-2 border-[#0078d4]">
                    Atualizações do Software
                  </div>

                  <div className="p-3 bg-slate-950 border border-slate-800 rounded-lg flex items-center justify-between">
                    <div>
                      <span className="text-slate-400 text-[10px] block">Versão Instalada:</span>
                      <strong className="text-white text-xs font-mono">v{formData.installedVersion} (Windows 10/11 x64)</strong>
                    </div>

                    <button
                      type="button"
                      onClick={handleCheckUpdate}
                      disabled={isCheckingUpdate}
                      className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded border border-slate-700 flex items-center gap-1.5 transition-colors font-medium"
                    >
                      <RefreshCw className={`w-3.5 h-3.5 text-sky-400 ${isCheckingUpdate ? 'animate-spin' : ''}`} />
                      <span>{isCheckingUpdate ? 'Consultando...' : 'Verificar Atualizações'}</span>
                    </button>
                  </div>

                  {updateMessage && (
                    <div className="p-2.5 rounded bg-emerald-950/80 border border-emerald-800 text-emerald-300 flex items-center gap-2 text-[11px]">
                      <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                      <span>{updateMessage}</span>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer Buttons (Inspired directly by the TeamViewer Options Dialog) */}
        <div className="bg-slate-950 px-5 py-3 border-t border-slate-800 flex items-center justify-end gap-2.5 select-none">
          <button
            type="button"
            onClick={handleApply}
            className="px-6 py-1.5 bg-[#0078d4] hover:bg-[#006cbd] text-white font-semibold rounded text-xs transition-colors shadow-sm"
          >
            OK
          </button>
          <button
            type="button"
            onClick={handleCancel}
            className="px-5 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 font-semibold rounded text-xs border border-slate-700 transition-colors"
          >
            Cancelar
          </button>
          <button
            type="button"
            onClick={handleApply}
            className="px-5 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 font-semibold rounded text-xs border border-slate-700 transition-colors"
          >
            Aplicar
          </button>
        </div>
      </div>
    </div>
  );
};
