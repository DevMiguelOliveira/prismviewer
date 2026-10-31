import React, { useState, useEffect, useRef } from 'react';
import {
  Shield,
  Eye,
  MousePointer2,
  Maximize2,
  Minimize2,
  RotateCcw,
  KeyRound,
  FileText,
  MessageSquare,
  X,
  ChevronDown,
  ChevronUp,
  Pin,
  PinOff,
  Send,
  AlertTriangle,
  FolderSync,
  Hash,
  Laptop,
  CheckCircle,
  Copy,
  Clock,
  Sparkles,
  Terminal,
} from 'lucide-react';
import { Device, FileTransferItem, ChatMessage } from '../types';

interface RemoteSessionProps {
  device: Device;
  operatorName: string;
  onEndSession: (durationSeconds: number, bytesTransferred: number) => void;
  onSendNotification: (msg: string, type?: 'success' | 'warning' | 'error') => void;
}

export const RemoteSession: React.FC<RemoteSessionProps> = ({
  device,
  operatorName,
  onEndSession,
  onSendNotification,
}) => {
  // Session Metrics & State
  const [secondsElapsed, setSecondsElapsed] = useState(0);
  const [bytesTransferred, setBytesTransferred] = useState(1450000);
  const [fps, setFps] = useState(30);
  const [rtt, setRtt] = useState(19);
  const [bitrate, setBitrate] = useState(840); // kbps

  // Toolbar Controls
  const [toolbarPinned, setToolbarPinned] = useState(true);
  const [toolbarCollapsed, setToolbarCollapsed] = useState(false);
  const [controlMode, setControlMode] = useState<'control' | 'viewOnly'>('control');
  const [screenScale, setScreenScale] = useState<'fit' | 'original'>('fit');
  const [activeMonitor, setActiveMonitor] = useState<number>(1);
  const [blockLocalInput, setBlockLocalInput] = useState(false);

  // Modals inside session
  const [showChat, setShowChat] = useState(false);
  const [showFileTransfer, setShowFileTransfer] = useState(false);
  const [showClipboard, setShowClipboard] = useState(false);
  const [showSasModal, setShowSasModal] = useState(false);
  const [showRebootModal, setShowRebootModal] = useState(false);
  const [clipboardText, setClipboardText] = useState('https://portal.empresa.com/diagnostics/session-491');

  // Interactive Workstation Simulator Data
  const [systemLogs, setSystemLogs] = useState<string[]>([
    '[SYSTEM] Windows 11 Pro build 22631.3447 inicializado.',
    '[SERVICES] PrysmView Host Service executando como LocalSystem (PID: 3120).',
    '[NETWORK] Adaptador Ethernet 1Gbps link ativo (192.168.1.101).',
    '[SECURITY] Criptografia de canal DTLS 1.3 ativa com handshake Ed25519.',
  ]);

  // Chat state
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([
    {
      id: '1',
      sender: 'host',
      senderName: 'Colaborador Local (Usuário)',
      text: 'Olá técnico, meu computador estava com lentidão ao abrir relatórios corporativos.',
      timestamp: '14:10',
    },
    {
      id: '2',
      sender: 'operator',
      senderName: operatorName,
      text: 'Boa tarde! Estou conectado via PrysmView com controle total. Vou verificar os serviços e processos em segundo plano.',
      timestamp: '14:11',
    },
  ]);
  const [newMessageText, setNewMessageText] = useState('');

  // File Transfer State
  const [transfers, setTransfers] = useState<FileTransferItem[]>([
    {
      id: 'ft-1',
      name: 'patch_sat_v2.4.dll',
      size: 482000,
      direction: 'upload',
      sha256: '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
      progress: 100,
      status: 'verified',
      timestamp: '14:12',
    },
  ]);

  // Timer loop
  useEffect(() => {
    const timer = setInterval(() => {
      setSecondsElapsed((prev) => prev + 1);
      // Simulate real-time bandwidth and slight jitter
      setBytesTransferred((prev) => prev + Math.floor(Math.random() * 45000 + 35000));
      setRtt(18 + Math.floor(Math.random() * 4));
      setFps(29 + Math.floor(Math.random() * 2));
      setBitrate(750 + Math.floor(Math.random() * 120));
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const formatDuration = (totalSec: number) => {
    const mins = Math.floor(totalSec / 60);
    const secs = totalSec % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  const handleSendMessage = () => {
    if (!newMessageText.trim()) return;
    const msg: ChatMessage = {
      id: Date.now().toString(),
      sender: 'operator',
      senderName: operatorName,
      text: newMessageText.trim(),
      timestamp: new Date().toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' }),
    };
    setChatMessages((prev) => [...prev, msg]);
    setNewMessageText('');

    // Simulate user reply after 2.5s
    setTimeout(() => {
      setChatMessages((prev) => [
        ...prev,
        {
          id: (Date.now() + 1).toString(),
          sender: 'host',
          senderName: 'Colaborador Local (Usuário)',
          text: 'Recebido! Estou acompanhando suas ações pela tela.',
          timestamp: new Date().toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' }),
        },
      ]);
    }, 2500);
  };

  const handleSendSAS = () => {
    setShowSasModal(false);
    onSendNotification('Comando Ctrl+Alt+Del (SAS) enviado com sucesso para a sessão segura do Windows.', 'success');
  };

  const handleRunDiagnostics = () => {
    if (controlMode === 'viewOnly') {
      onSendNotification('Modo Apenas Visualizar ativo: eventos de entrada remota estão suprimidos.', 'warning');
      return;
    }
    const timestamp = new Date().toLocaleTimeString();
    setSystemLogs((prev) => [
      ...prev,
      `[${timestamp}] DIAG: Integridade DISM & SFC verificada. Todos os serviços saudáveis.`,
    ]);
    onSendNotification('Diagnóstico do sistema operacional executado remotamente com sucesso!', 'success');
  };

  const handleClearCache = () => {
    if (controlMode === 'viewOnly') {
      onSendNotification('Modo Apenas Visualizar ativo: eventos de entrada remota estão suprimidos.', 'warning');
      return;
    }
    const timestamp = new Date().toLocaleTimeString();
    setSystemLogs((prev) => [
      ...prev,
      `[${timestamp}] CLEAN: 1.4 GB de arquivos temporários e cache liberados em C:\\Windows\\Temp.`,
    ]);
    onSendNotification('Limpeza de cache e temporários concluída no dispositivo remoto.', 'success');
  };

  return (
    <div className="relative w-full h-[calc(100vh-3.5rem)] bg-slate-950 flex flex-col overflow-hidden select-none">
      {/* =========================================================================
          RETRACTABLE FLOATING TOOLBAR (TeamViewer Style)
          ========================================================================= */}
      <div
        className={`absolute top-0 left-1/2 -translate-x-1/2 z-30 transition-all duration-300 ease-in-out ${
          toolbarCollapsed ? '-translate-y-[calc(100%-14px)] opacity-90 hover:opacity-100' : 'translate-y-0 opacity-100'
        }`}
      >
        <div className="bg-slate-900/95 backdrop-blur-md border border-slate-700/80 rounded-b-xl shadow-2xl px-3 py-1.5 flex flex-col items-center gap-1.5 text-xs text-slate-200">
          <div className="flex items-center gap-2 flex-wrap justify-center">
            {/* Device Info & Encryption Status */}
            <div className="flex items-center gap-2 pr-2 border-r border-slate-700/80">
              <span className="pulse-dot-green"></span>
              <span className="font-bold text-sky-400 font-mono">{device.id}</span>
              <span className="text-[10px] text-slate-400 hidden sm:inline">({device.store})</span>
              <span className="px-1.5 py-0.5 rounded bg-emerald-950/80 border border-emerald-800/80 text-[10px] text-emerald-400 flex items-center gap-1 font-mono">
                <Shield className="w-2.5 h-2.5" /> E2E DTLS 1.3
              </span>
            </div>

            {/* Live Metrics: RTT, FPS, Duration */}
            <div className="flex items-center gap-2 pr-2 border-r border-slate-700/80 text-[11px] font-mono">
              <span className="text-slate-400">
                RTT: <strong className="text-sky-300">{rtt}ms</strong>
              </span>
              <span className="text-slate-500">•</span>
              <span className="text-slate-400">
                FPS: <strong className="text-emerald-400">{fps}</strong>
              </span>
              <span className="text-slate-500">•</span>
              <span className="text-slate-400">
                Banda: <strong className="text-amber-300">{bitrate} kb/s</strong>
              </span>
              <span className="text-slate-500">•</span>
              <span className="text-slate-400 flex items-center gap-1">
                <Clock className="w-3 h-3 text-slate-400" />
                <strong className="text-white">{formatDuration(secondsElapsed)}</strong>
              </span>
            </div>

            {/* Mode Toggle: Full Control vs View-Only */}
            <div className="flex items-center bg-slate-950 rounded-lg p-0.5 border border-slate-800">
              <button
                onClick={() => {
                  setControlMode('control');
                  onSendNotification('Modo Controle Total ativado: mouse e teclado liberados.', 'success');
                }}
                className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] font-medium transition-colors ${
                  controlMode === 'control'
                    ? 'bg-sky-600 text-white shadow-sm'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
                title="Controle total sobre o computador remoto"
              >
                <MousePointer2 className="w-3 h-3" />
                <span>Controle</span>
              </button>
              <button
                onClick={() => {
                  setControlMode('viewOnly');
                  onSendNotification('Modo Apenas Visualizar ativado: entrada remota suprimida.', 'warning');
                }}
                className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] font-medium transition-colors ${
                  controlMode === 'viewOnly'
                    ? 'bg-amber-600 text-white shadow-sm'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
                title="Apenas visualizar sem interferir no usuário local"
              >
                <Eye className="w-3 h-3" />
                <span>Visualizar</span>
              </button>
            </div>

            {/* Quick Actions Buttons */}
            <div className="flex items-center gap-1">
              <button
                onClick={() => setScreenScale(screenScale === 'fit' ? 'original' : 'fit')}
                className="p-1.5 hover:bg-slate-800 rounded text-slate-300 hover:text-white"
                title={screenScale === 'fit' ? 'Tamanho Original (1:1)' : 'Ajustar à Tela (Fit)'}
              >
                {screenScale === 'fit' ? <Maximize2 className="w-3.5 h-3.5" /> : <Minimize2 className="w-3.5 h-3.5" />}
              </button>

              <button
                onClick={() => setShowSasModal(true)}
                className="flex items-center gap-1 px-2 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded transition-colors text-[11px]"
                title="Enviar combinação Ctrl+Alt+Del (SAS Windows)"
              >
                <KeyRound className="w-3 h-3 text-sky-400" />
                <span className="hidden md:inline">Ctrl+Alt+Del</span>
              </button>

              <button
                onClick={() => setShowFileTransfer(true)}
                className="flex items-center gap-1 px-2 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded transition-colors text-[11px]"
                title="Transferência de Arquivos com Verificação SHA-256"
              >
                <FolderSync className="w-3 h-3 text-emerald-400" />
                <span className="hidden md:inline">Arquivos</span>
              </button>

              <button
                onClick={() => setShowClipboard(true)}
                className="flex items-center gap-1 px-2 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded transition-colors text-[11px]"
                title="Sincronizar Área de Transferência (Clipboard < 1MB)"
              >
                <Copy className="w-3 h-3 text-cyan-400" />
                <span className="hidden md:inline">Clipboard</span>
              </button>

              <button
                onClick={() => setShowChat(!showChat)}
                className={`flex items-center gap-1 px-2 py-1 rounded transition-colors text-[11px] relative ${
                  showChat ? 'bg-sky-600 text-white' : 'bg-slate-800 hover:bg-slate-700 text-slate-200'
                }`}
                title="Chat de suporte em tempo real com o usuário local"
              >
                <MessageSquare className="w-3 h-3 text-amber-400" />
                <span className="hidden md:inline">Chat</span>
              </button>

              <button
                onClick={() => setShowRebootModal(true)}
                className="p-1.5 hover:bg-slate-800 rounded text-slate-300 hover:text-amber-400"
                title="Reiniciar dispositivo remoto com reconexão automática"
              >
                <RotateCcw className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Pin / Collapse Toolbar */}
            <div className="flex items-center gap-0.5 border-l border-slate-700/80 pl-2">
              <button
                onClick={() => setToolbarPinned(!toolbarPinned)}
                className={`p-1 rounded hover:bg-slate-800 ${toolbarPinned ? 'text-sky-400' : 'text-slate-500'}`}
                title={toolbarPinned ? 'Fixar barra sempre aberta' : 'Permitir recolhimento automático'}
              >
                {toolbarPinned ? <Pin className="w-3 h-3" /> : <PinOff className="w-3 h-3" />}
              </button>
              <button
                onClick={() => setToolbarCollapsed(!toolbarCollapsed)}
                className="p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-white"
                title={toolbarCollapsed ? 'Expandir barra' : 'Minimizar barra'}
              >
                {toolbarCollapsed ? <ChevronDown className="w-3 h-3" /> : <ChevronUp className="w-3 h-3" />}
              </button>

              {/* End Session Button */}
              <button
                onClick={() => onEndSession(secondsElapsed, bytesTransferred)}
                className="ml-1.5 flex items-center gap-1 px-2.5 py-1 bg-red-600 hover:bg-red-500 text-white rounded text-[11px] font-semibold transition-colors shadow-sm shadow-red-500/30"
                title="Encerrar sessão de acesso remoto e gravar auditoria"
              >
                <X className="w-3 h-3" />
                <span>Encerrar</span>
              </button>
            </div>
          </div>

          {/* Tiny grab handle when collapsed */}
          <div
            onClick={() => setToolbarCollapsed(false)}
            className="w-12 h-1 bg-slate-600 rounded-full cursor-pointer hover:bg-sky-400 transition-colors"
          ></div>
        </div>
      </div>

      {/* =========================================================================
          REMOTE DESKTOP CANVAS (VIRTUAL WORKSTATION WINDOWS 11 SCREEN)
          ========================================================================= */}
      <div className="flex-1 w-full h-full overflow-auto flex items-center justify-center p-2 sm:p-4 bg-slate-950">
        <div
          className={`relative bg-slate-900 border-2 border-slate-700/80 rounded-lg shadow-2xl overflow-hidden transition-all duration-200 ${
            screenScale === 'fit' ? 'w-full max-w-5xl aspect-[16/10] max-h-[85vh]' : 'w-[1280px] h-[800px] shrink-0'
          }`}
        >
          {/* Windows Window Titlebar for Remote Workstation */}
          <div className="h-7 bg-slate-800 border-b border-slate-700 flex items-center justify-between px-3 text-[11px] text-slate-300">
            <div className="flex items-center gap-2">
              <span className="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
              <span className="font-semibold text-slate-200">
                Área de Trabalho Remota • {device.name} [{device.store}]
              </span>
              <span className="text-slate-400 hidden sm:inline">• Usuário Local Ativo</span>
            </div>
            <div className="flex items-center gap-2 font-mono text-[10px] text-slate-400">
              <span>Windows 11 Pro 64-bit</span>
              <span className="text-slate-600">|</span>
              <span>Host IP: {device.ip}</span>
            </div>
          </div>

          {/* Desktop Interface */}
          <div className="h-[calc(100%-48px)] bg-[#0c1220] text-slate-100 flex flex-col sm:flex-row p-3 gap-3 overflow-hidden">
            {/* Left: Terminal Console / System Activity */}
            <div className="flex-1 bg-slate-950/90 border border-slate-800 rounded-lg p-3 flex flex-col justify-between">
              <div>
                <div className="flex items-center justify-between border-b border-slate-800 pb-2 mb-2">
                  <div className="flex items-center gap-2 text-xs font-bold text-sky-400 uppercase tracking-wide">
                    <Terminal className="w-3.5 h-3.5" />
                    <span>Console de Diagnóstico & Eventos do Host</span>
                  </div>
                  <div className="text-[10px] text-slate-400 font-mono">
                    Sessão Segura • DTLS 1.3
                  </div>
                </div>

                {/* System Logs Terminal Screen */}
                <div className="space-y-1.5 max-h-[220px] overflow-y-auto pr-1 font-mono text-xs text-slate-300 bg-slate-900/60 p-2.5 rounded-lg border border-slate-800/60">
                  {systemLogs.map((log, idx) => (
                    <div key={idx} className="flex items-start gap-1.5 leading-relaxed">
                      <span className="text-sky-400 font-bold shrink-0">&gt;</span>
                      <span className="text-slate-300">{log}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Resource Summary Bar */}
              <div className="mt-3 pt-3 border-t border-slate-800 bg-slate-900/40 -mx-3 -mb-3 p-3 rounded-b-lg">
                <div className="flex items-baseline justify-between mb-2">
                  <span className="text-xs text-slate-400 font-semibold uppercase">Status Geral do Dispositivo</span>
                  <span className="text-lg font-bold text-emerald-400 font-mono flex items-center gap-1.5">
                    <CheckCircle className="w-4 h-4 text-emerald-400" />
                    Operacional & Conectado
                  </span>
                </div>
                <div className="text-[10px] text-slate-500 flex items-center justify-between">
                  <span>Processos ativos: 142 • Tempo de atividade: {device.uptime}</span>
                  <span className="text-sky-400 font-mono">ID Host: {device.id}</span>
                </div>
              </div>
            </div>

            {/* Right: IT Action Commands & Hardware Diagnostics */}
            <div className="w-full sm:w-80 flex flex-col gap-2.5">
              <div className="bg-slate-950/80 border border-slate-800 rounded-lg p-3">
                <div className="text-xs font-bold text-slate-300 mb-2 flex items-center justify-between">
                  <span>Ferramentas de Suporte Rápido</span>
                  {controlMode === 'viewOnly' && (
                    <span className="text-[10px] text-amber-400 bg-amber-950/60 px-1.5 py-0.5 rounded border border-amber-800">
                      Entrada Bloqueada
                    </span>
                  )}
                </div>

                <div className="grid grid-cols-2 gap-2 text-xs">
                  <button
                    onClick={handleRunDiagnostics}
                    disabled={controlMode === 'viewOnly'}
                    className="flex flex-col items-center justify-center p-2.5 rounded-lg bg-sky-950/50 hover:bg-sky-900/60 border border-sky-800/60 text-sky-200 transition-colors disabled:opacity-50"
                  >
                    <Sparkles className="w-4 h-4 text-sky-400 mb-1" />
                    <span className="font-semibold text-[11px]">Diagnóstico</span>
                    <span className="text-[9px] text-sky-400/80">Verificar SO</span>
                  </button>

                  <button
                    onClick={handleClearCache}
                    disabled={controlMode === 'viewOnly'}
                    className="flex flex-col items-center justify-center p-2.5 rounded-lg bg-emerald-950/50 hover:bg-emerald-900/60 border border-emerald-800/60 text-emerald-200 transition-colors disabled:opacity-50"
                  >
                    <FolderSync className="w-4 h-4 text-emerald-400 mb-1" />
                    <span className="font-semibold text-[11px]">Limpar Cache</span>
                    <span className="text-[9px] text-emerald-400/80">Liberar Espaço</span>
                  </button>
                </div>
              </div>

              {/* Hardware Diagnostic Card */}
              <div className="bg-slate-950/80 border border-slate-800 rounded-lg p-3 text-[11px] space-y-1.5">
                <div className="text-xs font-bold text-slate-300 pb-1 border-b border-slate-800 flex items-center justify-between">
                  <span>Hardware & Telemetria</span>
                  <span className="text-[10px] text-emerald-400 font-mono">100% Saudável</span>
                </div>
                <div className="flex justify-between text-slate-400">
                  <span>CPU Remota:</span>
                  <span className="text-slate-200 font-mono">{device.cpu}</span>
                </div>
                <div className="flex justify-between text-slate-400">
                  <span>Memória RAM:</span>
                  <span className="text-slate-200 font-mono">{device.memory}</span>
                </div>
                <div className="flex justify-between text-slate-400">
                  <span>Armazenamento:</span>
                  <span className="text-slate-200 font-mono">{device.disk}</span>
                </div>
                <div className="flex justify-between text-slate-400">
                  <span>Monitores:</span>
                  <span className="text-emerald-400 font-mono">{device.monitors} tela(s) • {device.resolution}</span>
                </div>
              </div>
            </div>
          </div>

          {/* Windows 11 Taskbar Simulation */}
          <div className="h-8 bg-slate-950 border-t border-slate-800 px-3 flex items-center justify-between text-[11px] text-slate-400">
            <div className="flex items-center gap-2">
              <div className="w-5 h-5 rounded bg-sky-500/20 border border-sky-400/40 flex items-center justify-center text-sky-400 font-bold text-[10px]">
                ⊞
              </div>
              <div className="px-2 py-0.5 rounded bg-slate-800 text-sky-300 text-[10px] font-medium border border-slate-700">
                PrysmHost Service
              </div>
              <div className="px-2 py-0.5 rounded bg-slate-900 text-slate-400 text-[10px]">
                Terminal
              </div>
            </div>

            <div className="flex items-center gap-3 font-mono text-[10px]">
              <span className="flex items-center gap-1 text-emerald-400">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                PrysmHost Ativo (Porta 19533)
              </span>
              <span className="text-slate-500">POR • ABNT2</span>
              <span>14:15:32</span>
            </div>
          </div>
        </div>
      </div>

      {/* =========================================================================
          CHAT MODAL DRAWER
          ========================================================================= */}
      {showChat && (
        <div className="absolute right-4 bottom-4 w-80 sm:w-96 h-[420px] bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col z-40 animate-in fade-in slide-in-from-bottom-4">
          <div className="h-10 bg-slate-800 border-b border-slate-700 px-3 flex items-center justify-between rounded-t-xl">
            <div className="flex items-center gap-2 text-xs font-bold text-slate-200">
              <MessageSquare className="w-3.5 h-3.5 text-sky-400" />
              <span>Chat de Suporte com o Caixa</span>
            </div>
            <button
              onClick={() => setShowChat(false)}
              className="text-slate-400 hover:text-white p-1 rounded"
            >
              ✕
            </button>
          </div>

          <div className="flex-1 p-3 overflow-y-auto space-y-2.5 text-xs">
            {chatMessages.map((msg) => (
              <div
                key={msg.id}
                className={`flex flex-col ${
                  msg.sender === 'operator' ? 'items-end' : 'items-start'
                }`}
              >
                <div className="flex items-baseline gap-1 text-[10px] text-slate-400 mb-0.5">
                  <span className="font-semibold text-slate-300">{msg.senderName}</span>
                  <span>{msg.timestamp}</span>
                </div>
                <div
                  className={`max-w-[85%] px-3 py-2 rounded-xl text-xs ${
                    msg.sender === 'operator'
                      ? 'bg-sky-600 text-white rounded-tr-none'
                      : 'bg-slate-800 text-slate-200 border border-slate-700 rounded-tl-none'
                  }`}
                >
                  {msg.text}
                </div>
              </div>
            ))}
          </div>

          <div className="p-2 border-t border-slate-800 flex items-center gap-2">
            <input
              type="text"
              placeholder="Digite uma mensagem para o usuário local..."
              value={newMessageText}
              onChange={(e) => setNewMessageText(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSendMessage()}
              className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-sky-500"
            />
            <button
              onClick={handleSendMessage}
              className="p-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg transition-colors"
            >
              <Send className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      )}

      {/* =========================================================================
          FILE TRANSFER MODAL (WITH SHA-256 INTEGRITY)
          ========================================================================= */}
      {showFileTransfer && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-lg shadow-2xl p-5 text-xs text-slate-200">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800 mb-4">
              <div className="flex items-center gap-2">
                <FolderSync className="w-4 h-4 text-emerald-400" />
                <h3 className="font-bold text-sm text-white">Transferência de Arquivos Segura</h3>
              </div>
              <button
                onClick={() => setShowFileTransfer(false)}
                className="text-slate-400 hover:text-white"
              >
                ✕
              </button>
            </div>

            <p className="text-slate-400 mb-3 text-[11px]">
              Transferência de blocos via WebRTC com hash SHA-256 verificado no destino antes da gravação em disco.
            </p>

            <div className="space-y-2 mb-4 max-h-48 overflow-y-auto">
              {transfers.map((t) => (
                <div key={t.id} className="p-2.5 rounded-lg bg-slate-950 border border-slate-800">
                  <div className="flex items-center justify-between mb-1">
                    <span className="font-mono font-semibold text-slate-200">{t.name}</span>
                    <span className="text-[10px] text-emerald-400 font-mono flex items-center gap-1">
                      <CheckCircle className="w-3 h-3" /> Verificado SHA-256
                    </span>
                  </div>
                  <div className="text-[10px] text-slate-400 font-mono mb-1">
                    SHA256: {t.sha256.substring(0, 24)}...
                  </div>
                  <div className="w-full bg-slate-800 h-1.5 rounded-full overflow-hidden">
                    <div className="bg-emerald-500 h-full w-full"></div>
                  </div>
                </div>
              ))}
            </div>

            <div className="flex items-center gap-2 pt-3 border-t border-slate-800">
              <button
                onClick={() => {
                  const newF: FileTransferItem = {
                    id: Date.now().toString(),
                    name: 'diagnostico_sistema.zip',
                    size: 24500,
                    direction: 'upload',
                    sha256: '4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a',
                    progress: 100,
                    status: 'verified',
                    timestamp: new Date().toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' }),
                  };
                  setTransfers((prev) => [...prev, newF]);
                  onSendNotification('Arquivo diagnostico_sistema.zip enviado e verificado por SHA-256 no dispositivo remoto.', 'success');
                }}
                className="flex-1 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg font-semibold transition-colors flex items-center justify-center gap-1.5"
              >
                <FolderSync className="w-3.5 h-3.5" />
                <span>Enviar Arquivo ao Host</span>
              </button>
              <button
                onClick={() => setShowFileTransfer(false)}
                className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg transition-colors"
              >
                Fechar
              </button>
            </div>
          </div>
        </div>
      )}

      {/* =========================================================================
          CLIPBOARD SYNC MODAL
          ========================================================================= */}
      {showClipboard && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-md shadow-2xl p-5 text-xs text-slate-200">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800 mb-3">
              <div className="flex items-center gap-2">
                <Copy className="w-4 h-4 text-cyan-400" />
                <h3 className="font-bold text-sm text-white">Área de Transferência Remota</h3>
              </div>
              <button onClick={() => setShowClipboard(false)} className="text-slate-400 hover:text-white">✕</button>
            </div>
            <p className="text-[11px] text-slate-400 mb-2">
              Sincronize texto entre a sua máquina e a área de transferência do Windows do dispositivo remoto (limite máximo de 1 MB).
            </p>
            <textarea
              rows={4}
              value={clipboardText}
              onChange={(e) => setClipboardText(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-cyan-500 mb-3"
            />
            <div className="flex items-center gap-2">
              <button
                onClick={() => {
                  onSendNotification('Texto enviado para o clipboard remoto com sucesso!', 'success');
                  setShowClipboard(false);
                }}
                className="flex-1 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg font-semibold transition-colors"
              >
                Colar no Dispositivo Remoto
              </button>
              <button
                onClick={() => setShowClipboard(false)}
                className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg"
              >
                Cancelar
              </button>
            </div>
          </div>
        </div>
      )}

      {/* =========================================================================
          SAS (CTRL+ALT+DEL) CONFIRMATION MODAL
          ========================================================================= */}
      {showSasModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-sm shadow-2xl p-5 text-xs text-slate-200 text-center">
            <div className="w-10 h-10 rounded-full bg-sky-950 border border-sky-800 text-sky-400 flex items-center justify-center mx-auto mb-3">
              <KeyRound className="w-5 h-5" />
            </div>
            <h3 className="font-bold text-sm text-white mb-1">Enviar Ctrl+Alt+Del (SAS)</h3>
            <p className="text-[11px] text-slate-400 mb-4">
              Isso acionará a Sequência de Atenção Segura (SAS) através do serviço SYSTEM do PrysmView para desbloquear a tela ou abrir o Gerenciador de Tarefas no Windows.
            </p>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowSasModal(false)}
                className="flex-1 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg"
              >
                Cancelar
              </button>
              <button
                onClick={handleSendSAS}
                className="flex-1 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg font-semibold shadow-md"
              >
                Confirmar SAS
              </button>
            </div>
          </div>
        </div>
      )}

      {/* =========================================================================
          REBOOT DISPOSITIVO CONFIRMATION MODAL
          ========================================================================= */}
      {showRebootModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-sm shadow-2xl p-5 text-xs text-slate-200 text-center">
            <div className="w-10 h-10 rounded-full bg-amber-950 border border-amber-800 text-amber-400 flex items-center justify-center mx-auto mb-3">
              <AlertTriangle className="w-5 h-5" />
            </div>
            <h3 className="font-bold text-sm text-white mb-1">Reiniciar Dispositivo Remoto</h3>
            <p className="text-[11px] text-slate-400 mb-4">
              O sistema operacional da máquina ({device.id}) será reiniciado. O serviço do PrysmView Host subirá automaticamente em menos de 10s após o boot.
            </p>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowRebootModal(false)}
                className="flex-1 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg"
              >
                Cancelar
              </button>
              <button
                onClick={() => {
                  setShowRebootModal(false);
                  onSendNotification(`Comando shutdown /r /t 5 enviado para ${device.id}. Reconexão automática engatilhada.`, 'warning');
                }}
                className="flex-1 py-2 bg-amber-600 hover:bg-amber-500 text-white rounded-lg font-semibold shadow-md"
              >
                Reiniciar Agora
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
