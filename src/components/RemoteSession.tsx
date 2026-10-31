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
  Activity,
  Monitor,
  Minus,
  Square,
  Folder,
  Trash2,
  Globe,
  Settings as SettingsIcon,
  HelpCircle,
  Image as ImageIcon,
  Check,
  PhoneCall,
  Edit3,
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
  // Session Metrics & Counters
  const [secondsElapsed, setSecondsElapsed] = useState(0);
  const [bytesTransferred, setBytesTransferred] = useState(1450000);
  const [fps, setFps] = useState(30);
  const [rtt, setRtt] = useState(18);
  const [bitrate, setBitrate] = useState(820); // kbps

  // Viewer Controls & States
  const [screenScale, setScreenScale] = useState<'fit' | 'original'>('fit');
  const [controlMode, setControlMode] = useState<'control' | 'viewOnly'>('control');
  const [hideWallpaper, setHideWallpaper] = useState(true); // TeamViewer defaults to black background for low latency!
  const [activeMonitor, setActiveMonitor] = useState<number>(1);
  const [qualityMode, setQualityMode] = useState<'speed' | 'quality'>('speed');
  const [isRibbonCollapsed, setIsRibbonCollapsed] = useState(false);
  const [isWidgetMinimized, setIsWidgetMinimized] = useState(false);
  const [isStartMenuOpen, setIsStartMenuOpen] = useState(false);

  // Active Dropdown Menu in TeamViewer Ribbon
  const [activeMenu, setActiveMenu] = useState<'acoes' | 'ver' | 'comunicar' | 'arquivos' | null>(null);

  // Modals & Panels
  const [showChat, setShowChat] = useState(false);
  const [showFileTransfer, setShowFileTransfer] = useState(false);
  const [showClipboard, setShowClipboard] = useState(false);
  const [showSasModal, setShowSasModal] = useState(false);
  const [showRebootModal, setShowRebootModal] = useState(false);
  const [showSysInfoModal, setShowSysInfoModal] = useState(false);

  // Clipboard & Chat States
  const [clipboardText, setClipboardText] = useState('https://portal.empresa.com/suporte/ticket-4921');
  const [newMessageText, setNewMessageText] = useState('');
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([
    {
      id: '1',
      sender: 'host',
      senderName: 'Usuário Remoto (' + (device.name.split(' ')[0] || 'Cliente') + ')',
      text: 'Olá! A conexão remota foi autorizada e estou acompanhando a tela.',
      timestamp: '14:10',
    },
    {
      id: '2',
      sender: 'operator',
      senderName: operatorName,
      text: 'Boa tarde! Conexão estabelecida com sucesso via PrysmView. Vou iniciar o suporte.',
      timestamp: '14:11',
    },
  ]);

  // File Transfer State
  const [transfers, setTransfers] = useState<FileTransferItem[]>([
    {
      id: 'ft-1',
      name: 'suporte_diagnostico.log',
      size: 482000,
      direction: 'upload',
      sha256: '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
      progress: 100,
      status: 'verified',
      timestamp: '14:12',
    },
  ]);

  // Virtual Desktop Icons selection
  const [selectedIcon, setSelectedIcon] = useState<string | null>(null);

  // Timer loop for metrics
  useEffect(() => {
    const timer = setInterval(() => {
      setSecondsElapsed((prev) => prev + 1);
      setBytesTransferred((prev) => prev + Math.floor(Math.random() * 42000 + 35000));
      setRtt(17 + Math.floor(Math.random() * 5));
      setFps(qualityMode === 'speed' ? 30 : 25);
      setBitrate(qualityMode === 'speed' ? 780 + Math.floor(Math.random() * 80) : 1850 + Math.floor(Math.random() * 200));
    }, 1000);
    return () => clearInterval(timer);
  }, [qualityMode]);

  // Close menus on outside click
  useEffect(() => {
    const handleDocClick = () => {
      setActiveMenu(null);
    };
    if (activeMenu) {
      document.addEventListener('click', handleDocClick);
      return () => document.removeEventListener('click', handleDocClick);
    }
  }, [activeMenu]);

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

    setTimeout(() => {
      setChatMessages((prev) => [
        ...prev,
        {
          id: (Date.now() + 1).toString(),
          sender: 'host',
          senderName: 'Usuário Remoto',
          text: 'Recebido! O problema foi resolvido.',
          timestamp: new Date().toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' }),
        },
      ]);
    }, 2500);
  };

  const handleSendSAS = () => {
    setShowSasModal(false);
    onSendNotification('Comando Ctrl+Alt+Del (SAS) enviado com sucesso ao Windows remoto.', 'success');
  };

  const handleEnd = () => {
    onEndSession(secondsElapsed, bytesTransferred);
  };

  // Mock desktop icons arranged like Windows 10/11 in TeamViewer
  const desktopIcons = [
    { id: 'recycle', name: 'Lixeira', icon: Trash2, color: 'text-slate-300' },
    { id: 'pc', name: 'Este Computador', icon: Monitor, color: 'text-sky-400' },
    { id: 'terminal', name: 'PowerShell / Terminal', icon: Terminal, color: 'text-blue-400' },
    { id: 'vscode', name: 'Visual Studio Code', icon: FileText, color: 'text-sky-500' },
    { id: 'projects', name: 'Projetos Dev', icon: Folder, color: 'text-amber-400' },
    { id: 'browser', name: 'Google Chrome', icon: Globe, color: 'text-red-400' },
    { id: 'docs', name: 'Documentos', icon: Folder, color: 'text-amber-300' },
    { id: 'settings', name: 'Painel de Controle', icon: SettingsIcon, color: 'text-slate-400' },
    { id: 'app', name: 'PrysmView Host', icon: Laptop, color: 'text-emerald-400' },
    { id: 'audit', name: 'Auditoria_2026.log', icon: FileText, color: 'text-emerald-300' },
  ];

  return (
    <div className="relative w-full h-[calc(100vh-3.5rem)] bg-black flex flex-col overflow-hidden select-none font-sans">
      
      {/* =========================================================================
          1. WINDOW NATIVE-LIKE TOP TAB BAR (Estilo Janela do TeamViewer)
          ========================================================================= */}
      <div className="h-8 bg-[#182030] border-b border-[#242f45] px-2 flex items-center justify-between select-none shrink-0 z-40">
        {/* Left: Session Tabs */}
        <div className="flex items-center gap-1 h-full pt-1">
          <div className="h-full px-3 bg-[#0d1424] border-t-2 border-[#0078d4] text-white text-xs font-medium rounded-t flex items-center gap-2 shadow-sm">
            {/* TeamViewer Style Dual-Arrow Icon */}
            <div className="w-4 h-4 rounded-full bg-[#0078d4] flex items-center justify-center text-white shrink-0">
              <svg className="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none">
                <path d="M7 16V4M7 4L3 8M7 4L11 8M17 8V20M17 20L21 16M17 20L13 16" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </div>
            <span className="font-semibold text-xs tracking-tight">{device.name.split(' ')[0] || device.id}</span>
            <span className="text-[10px] text-slate-400 font-mono">({device.id})</span>
            <button
              onClick={handleEnd}
              className="hover:bg-red-500/80 hover:text-white text-slate-400 rounded p-0.5 ml-1 transition-colors"
              title="Encerrar esta sessão remota"
            >
              <X className="w-3 h-3" />
            </button>
          </div>

          <button
            onClick={() => onSendNotification('Abrir nova aba de conexão remota...')}
            className="p-1 text-slate-400 hover:text-white hover:bg-slate-800 rounded transition-colors"
            title="Conectar a outro dispositivo em nova aba"
          >
            +
          </button>
        </div>

        {/* Right: Window Controls */}
        <div className="flex items-center gap-1 text-slate-400">
          <button className="p-1 hover:bg-slate-800 hover:text-white rounded" title="Minimizar">
            <Minus className="w-3 h-3" />
          </button>
          <button
            onClick={() => setScreenScale(screenScale === 'fit' ? 'original' : 'fit')}
            className="p-1 hover:bg-slate-800 hover:text-white rounded"
            title="Maximizar / Restaurar Janela"
          >
            <Square className="w-2.5 h-2.5" />
          </button>
          <button
            onClick={handleEnd}
            className="p-1 hover:bg-red-600 hover:text-white rounded transition-colors"
            title="Fechar e Desconectar"
          >
            <X className="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* =========================================================================
          2. TEAMVIEWER TOP RIBBON TOOLBAR (Barra de Menus e Ações Exata da Imagem)
          ========================================================================= */}
      {!isRibbonCollapsed && (
        <div className="h-10 bg-slate-900 border-b border-slate-700/80 px-3 flex items-center justify-between text-xs text-slate-200 select-none shadow-md shrink-0 z-30">
          
          {/* Left Menus: Ações, Ver, Comunicar, Arquivos & extras */}
          <div className="flex items-center gap-1 relative" onClick={(e) => e.stopPropagation()}>
            {/* Menu: Ações */}
            <div className="relative">
              <button
                onClick={() => setActiveMenu(activeMenu === 'acoes' ? null : 'acoes')}
                className={`px-2.5 py-1 rounded flex items-center gap-1 hover:bg-slate-800 transition-colors ${
                  activeMenu === 'acoes' ? 'bg-[#0078d4] text-white' : 'text-slate-200'
                }`}
              >
                <span>Ações</span>
                <ChevronDown className="w-3 h-3 text-slate-400" />
              </button>

              {activeMenu === 'acoes' && (
                <div className="absolute top-full left-0 mt-1 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1 z-50 animate-in fade-in">
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setShowSasModal(true);
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center gap-2"
                  >
                    <KeyRound className="w-3.5 h-3.5 text-sky-400" />
                    <span>Enviar Ctrl+Alt+Del (SAS)</span>
                  </button>
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setShowRebootModal(true);
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center gap-2"
                  >
                    <RotateCcw className="w-3.5 h-3.5 text-amber-400" />
                    <span>Reiniciar Computador Remoto</span>
                  </button>
                  <div className="my-1 border-t border-slate-800"></div>
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setControlMode(controlMode === 'control' ? 'viewOnly' : 'control');
                      onSendNotification(
                        controlMode === 'control' ? 'Modo Apenas Visualizar ativado.' : 'Controle total ativado.',
                        'warning'
                      );
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center gap-2"
                  >
                    {controlMode === 'control' ? <Eye className="w-3.5 h-3.5 text-amber-400" /> : <MousePointer2 className="w-3.5 h-3.5 text-emerald-400" />}
                    <span>{controlMode === 'control' ? 'Ativar Modo Apenas Visualizar' : 'Retomar Controle Total'}</span>
                  </button>
                </div>
              )}
            </div>

            {/* Menu: Ver */}
            <div className="relative">
              <button
                onClick={() => setActiveMenu(activeMenu === 'ver' ? null : 'ver')}
                className={`px-2.5 py-1 rounded flex items-center gap-1 hover:bg-slate-800 transition-colors ${
                  activeMenu === 'ver' ? 'bg-[#0078d4] text-white' : 'text-slate-200'
                }`}
              >
                <span>Ver</span>
                <ChevronDown className="w-3 h-3 text-slate-400" />
              </button>

              {activeMenu === 'ver' && (
                <div className="absolute top-full left-0 mt-1 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1 z-50 animate-in fade-in">
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setScreenScale(screenScale === 'fit' ? 'original' : 'fit');
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center justify-between"
                  >
                    <span>Escala: {screenScale === 'fit' ? 'Ajustar à Tela (Fit)' : 'Tamanho Original (1:1)'}</span>
                    <span className="text-[10px] text-sky-400">Alternar</span>
                  </button>
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setHideWallpaper(!hideWallpaper);
                      onSendNotification(
                        hideWallpaper ? 'Papel de parede remoto exibido.' : 'Papel de parede ocultado (Modo Economia).',
                        'success'
                      );
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center justify-between"
                  >
                    <span>Ocultar Papel de Parede</span>
                    {hideWallpaper && <Check className="w-3.5 h-3.5 text-emerald-400" />}
                  </button>
                  <div className="my-1 border-t border-slate-800"></div>
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setQualityMode(qualityMode === 'speed' ? 'quality' : 'speed');
                      onSendNotification(
                        qualityMode === 'speed' ? 'Qualidade HD (Alta Resolução) ativada.' : 'Fluidez 60 FPS ativada.',
                        'success'
                      );
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center justify-between"
                  >
                    <span>Perfil: {qualityMode === 'speed' ? 'Fluidez (60 FPS)' : 'Alta Resolução (HD)'}</span>
                    <span className="text-[10px] text-sky-400">Alternar</span>
                  </button>
                </div>
              )}
            </div>

            {/* Menu: Comunicar */}
            <div className="relative">
              <button
                onClick={() => setActiveMenu(activeMenu === 'comunicar' ? null : 'comunicar')}
                className={`px-2.5 py-1 rounded flex items-center gap-1 hover:bg-slate-800 transition-colors ${
                  activeMenu === 'comunicar' ? 'bg-[#0078d4] text-white' : 'text-slate-200'
                }`}
              >
                <span>Comunicar</span>
                <ChevronDown className="w-3 h-3 text-slate-400" />
              </button>

              {activeMenu === 'comunicar' && (
                <div className="absolute top-full left-0 mt-1 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1 z-50 animate-in fade-in">
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setShowChat(true);
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center gap-2"
                  >
                    <MessageSquare className="w-3.5 h-3.5 text-sky-400" />
                    <span>Abrir Chat com o Usuário Remoto</span>
                  </button>
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setShowClipboard(true);
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center gap-2"
                  >
                    <Copy className="w-3.5 h-3.5 text-cyan-400" />
                    <span>Sincronizar Área de Transferência</span>
                  </button>
                </div>
              )}
            </div>

            {/* Menu: Arquivos & extras */}
            <div className="relative">
              <button
                onClick={() => setActiveMenu(activeMenu === 'arquivos' ? null : 'arquivos')}
                className={`px-2.5 py-1 rounded flex items-center gap-1 hover:bg-slate-800 transition-colors ${
                  activeMenu === 'arquivos' ? 'bg-[#0078d4] text-white' : 'text-slate-200'
                }`}
              >
                <span>Arquivos & extras</span>
                <ChevronDown className="w-3 h-3 text-slate-400" />
              </button>

              {activeMenu === 'arquivos' && (
                <div className="absolute top-full left-0 mt-1 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1 z-50 animate-in fade-in">
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setShowFileTransfer(true);
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center gap-2"
                  >
                    <FolderSync className="w-3.5 h-3.5 text-emerald-400" />
                    <span>Transferência de Arquivos Segura (SHA-256)</span>
                  </button>
                  <button
                    onClick={() => {
                      setActiveMenu(null);
                      setShowSysInfoModal(true);
                    }}
                    className="w-full text-left px-3 py-1.5 hover:bg-[#0078d4] hover:text-white flex items-center gap-2"
                  >
                    <Laptop className="w-3.5 h-3.5 text-cyan-400" />
                    <span>Informações do Sistema Remoto</span>
                  </button>
                </div>
              )}
            </div>
          </div>

          {/* Right Action Icons (Identicos aos ícones da imagem do TeamViewer) */}
          <div className="flex items-center gap-1.5">
            {/* SAS Ctrl+Alt+Del Quick Icon */}
            <button
              onClick={() => setShowSasModal(true)}
              className="p-1.5 hover:bg-slate-800 rounded text-slate-300 hover:text-white"
              title="Enviar combinação Ctrl+Alt+Del (SAS)"
            >
              <KeyRound className="w-4 h-4 text-sky-400" />
            </button>

            {/* File Transfer Quick Icon */}
            <button
              onClick={() => setShowFileTransfer(true)}
              className="p-1.5 hover:bg-slate-800 rounded text-slate-300 hover:text-white"
              title="Transferência de arquivos bidirecional com SHA-256"
            >
              <FolderSync className="w-4 h-4 text-emerald-400" />
            </button>

            {/* Refresh Frame Quick Icon */}
            <button
              onClick={() => onSendNotification('Frame de vídeo e tiles WebRTC recarregados.')}
              className="p-1.5 hover:bg-slate-800 rounded text-slate-300 hover:text-white"
              title="Solicitar keyframe completo de atualização"
            >
              <RotateCcw className="w-4 h-4 text-slate-300" />
            </button>

            {/* Toggle Wallpaper Quick Icon */}
            <button
              onClick={() => {
                setHideWallpaper(!hideWallpaper);
                onSendNotification(
                  hideWallpaper ? 'Papel de parede exibido.' : 'Papel de parede ocultado (Fundo preto de alto desempenho).',
                  'success'
                );
              }}
              className={`p-1.5 rounded transition-colors ${
                hideWallpaper ? 'bg-slate-800 text-sky-400' : 'hover:bg-slate-800 text-slate-400'
              }`}
              title={hideWallpaper ? 'Restaurar papel de parede remoto' : 'Ocultar papel de parede remoto (Mais rápido)'}
            >
              <ImageIcon className="w-4 h-4" />
            </button>

            {/* Quality Switcher Quick Icon */}
            <button
              onClick={() => {
                setQualityMode(qualityMode === 'speed' ? 'quality' : 'speed');
                onSendNotification(
                  qualityMode === 'speed' ? 'Modo Alta Resolução ativado.' : 'Modo 60 FPS ativado.',
                  'success'
                );
              }}
              className="p-1.5 hover:bg-slate-800 rounded text-slate-300 hover:text-white"
              title={qualityMode === 'speed' ? 'Modo atual: Fluidez (60 FPS)' : 'Modo atual: Alta Definição (HD)'}
            >
              <Sparkles className="w-4 h-4 text-amber-400" />
            </button>

            <span className="h-4 w-[1px] bg-slate-700 mx-0.5"></span>

            {/* Clipboard Quick Icon */}
            <button
              onClick={() => setShowClipboard(true)}
              className="p-1.5 hover:bg-slate-800 rounded text-slate-300 hover:text-white"
              title="Sincronizar área de transferência (Clipboard)"
            >
              <Copy className="w-4 h-4 text-cyan-400" />
            </button>

            {/* Chat Quick Icon */}
            <button
              onClick={() => setShowChat(!showChat)}
              className={`p-1.5 rounded transition-colors relative ${
                showChat ? 'bg-[#0078d4] text-white' : 'hover:bg-slate-800 text-slate-300'
              }`}
              title="Chat com o usuário remoto"
            >
              <MessageSquare className="w-4 h-4 text-sky-400" />
              <span className="absolute top-1 right-1 w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
            </button>

            {/* Telemetry Indicator */}
            <div
              className="hidden sm:flex items-center gap-1.5 px-2 py-0.5 bg-slate-950 rounded border border-slate-800 text-[10px] font-mono text-slate-300 cursor-pointer"
              onClick={() => setShowSysInfoModal(true)}
              title="Latência RTT, FPS e Banda WebRTC"
            >
              <Activity className="w-3 h-3 text-emerald-400" />
              <span className="text-emerald-400 font-bold">{rtt}ms</span>
              <span className="text-slate-600">|</span>
              <span>{fps}fps</span>
              <span className="text-slate-600">|</span>
              <span>{bitrate}kbps</span>
            </div>

            {/* Fullscreen Toggle */}
            <button
              onClick={() => setScreenScale(screenScale === 'fit' ? 'original' : 'fit')}
              className="p-1.5 hover:bg-slate-800 rounded text-slate-300 hover:text-white"
              title={screenScale === 'fit' ? 'Tamanho 1:1' : 'Ajustar à tela'}
            >
              <Maximize2 className="w-4 h-4" />
            </button>

            {/* Collapse Ribbon Button */}
            <button
              onClick={() => setIsRibbonCollapsed(true)}
              className="p-1.5 hover:bg-slate-800 rounded text-slate-400 hover:text-white"
              title="Recolher barra de ferramentas"
            >
              <ChevronUp className="w-4 h-4" />
            </button>

            {/* Red Disconnect Button (Exato da imagem do TeamViewer) */}
            <button
              onClick={handleEnd}
              className="ml-1 px-2 py-1 bg-red-950/60 hover:bg-red-600 border border-red-700/80 hover:border-red-500 text-red-200 hover:text-white rounded transition-colors flex items-center gap-1 shadow-sm font-semibold text-[11px]"
              title="Encerrar conexão remota imediatamente"
            >
              <X className="w-3.5 h-3.5" />
              <span className="hidden md:inline">Desconectar</span>
            </button>
          </div>
        </div>
      )}

      {/* Floating expand chevron when ribbon is collapsed */}
      {isRibbonCollapsed && (
        <div className="absolute top-8 left-1/2 -translate-x-1/2 z-40">
          <button
            onClick={() => setIsRibbonCollapsed(false)}
            className="px-3 py-0.5 bg-slate-900 border border-slate-700 border-t-0 rounded-b text-slate-300 hover:text-white hover:bg-slate-800 shadow-lg text-[10px] flex items-center gap-1 transition-all"
            title="Expandir barra de ferramentas do TeamViewer"
          >
            <span>Barra de Ações</span>
            <ChevronDown className="w-3 h-3" />
          </button>
        </div>
      )}

      {/* =========================================================================
          3. REAL-TIME REMOTE DESKTOP CANVAS (Exato ao visual da tela TeamViewer)
          ========================================================================= */}
      <div
        className={`flex-1 w-full relative overflow-auto flex flex-col justify-between ${
          hideWallpaper
            ? 'bg-[#000000]' /* Fundo preto de alto desempenho do TeamViewer */
            : 'bg-gradient-to-br from-slate-950 via-[#0a1122] to-slate-900'
        }`}
        onClick={() => setIsStartMenuOpen(false)}
      >
        {/* Remote Desktop Icons Area */}
        <div className="flex-1 p-4 grid grid-flow-col auto-cols-[85px] grid-rows-[repeat(auto-fill,85px)] gap-2 content-start select-none">
          {desktopIcons.map((ic) => {
            const IconComp = ic.icon;
            const isSel = selectedIcon === ic.id;
            return (
              <div
                key={ic.id}
                onClick={(e) => {
                  e.stopPropagation();
                  setSelectedIcon(ic.id);
                }}
                onDoubleClick={() => {
                  if (ic.id === 'terminal') {
                    onSendNotification('Terminal remoto do Windows aberto.');
                  } else if (ic.id === 'app') {
                    onSendNotification('PrysmView Host em execução com privilégios SYSTEM.', 'success');
                  } else {
                    onSendNotification(`Abrindo '${ic.name}' na máquina remota...`);
                  }
                }}
                className={`w-[82px] h-[82px] rounded flex flex-col items-center justify-center p-1 cursor-pointer transition-colors text-center ${
                  isSel
                    ? 'bg-[#0078d4]/30 border border-[#0078d4]/60 text-white'
                    : 'hover:bg-white/10 text-slate-200'
                }`}
              >
                <IconComp className={`w-8 h-8 mb-1 drop-shadow-md ${ic.color}`} />
                <span className="text-[10px] leading-tight line-clamp-2 px-1 font-medium drop-shadow-md">
                  {ic.name}
                </span>
              </div>
            );
          })}
        </div>

        {/* =========================================================================
            4. FLOATING PRYSMVIEW WIDGET (O Widget do canto inferior direito na imagem)
            ========================================================================= */}
        <div className="absolute right-4 bottom-12 z-30 select-none shadow-2xl">
          <div className="w-64 bg-[#0d162a]/95 backdrop-blur-md border border-[#1e2e4e] rounded-t-lg overflow-hidden text-xs text-slate-200 shadow-2xl">
            {/* Widget Title Bar */}
            <div className="bg-[#0b1222] px-3 py-2 border-b border-[#1e2e4e] flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-3.5 h-3.5 rounded-full bg-[#0078d4] flex items-center justify-center text-white">
                  <svg className="w-2 h-2" viewBox="0 0 24 24" fill="none">
                    <path d="M7 16V4M7 4L3 8M7 4L11 8M17 8V20M17 20L21 16M17 20L13 16" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                </div>
                <span className="font-bold text-xs text-white tracking-wide">PrysmView</span>
              </div>
              <div className="flex items-center gap-1">
                <button
                  onClick={() => setIsWidgetMinimized(!isWidgetMinimized)}
                  className="p-1 hover:bg-slate-800 text-slate-400 hover:text-white rounded"
                >
                  <Minus className="w-3 h-3" />
                </button>
              </div>
            </div>

            {/* Widget Body */}
            {!isWidgetMinimized && (
              <div className="p-3 space-y-2.5">
                <div className="flex items-center gap-1.5 text-[11px] text-emerald-400 font-semibold">
                  <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                  <span>Sessão Remota Ativa</span>
                </div>

                <div className="bg-slate-950/70 p-2 rounded border border-slate-800 space-y-1 text-[11px]">
                  <div className="flex justify-between text-slate-400">
                    <span>Técnico Conectado:</span>
                    <strong className="text-white">{operatorName}</strong>
                  </div>
                  <div className="flex justify-between text-slate-400">
                    <span>Tempo de Sessão:</span>
                    <span className="font-mono text-sky-400">{formatDuration(secondsElapsed)}</span>
                  </div>
                  <div className="flex justify-between text-slate-400">
                    <span>Criptografia E2E:</span>
                    <span className="font-mono text-emerald-400">DTLS 1.3 / DPAPI</span>
                  </div>
                </div>

                <div className="flex items-center gap-1 pt-1">
                  <button
                    onClick={() => setShowChat(true)}
                    className="flex-1 py-1.5 bg-[#0078d4] hover:bg-[#006cbd] text-white font-medium rounded text-[11px] flex items-center justify-center gap-1"
                  >
                    <MessageSquare className="w-3 h-3" />
                    <span>Chat</span>
                  </button>
                  <button
                    onClick={handleEnd}
                    className="flex-1 py-1.5 bg-red-600/80 hover:bg-red-500 text-white font-medium rounded text-[11px] flex items-center justify-center gap-1"
                  >
                    <X className="w-3 h-3" />
                    <span>Encerrar</span>
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* =========================================================================
            5. WINDOWS 10/11 TASKBAR (Exato ao rodapé da imagem do TeamViewer)
            ========================================================================= */}
        <div className="h-10 bg-[#0c121e]/95 backdrop-blur-md border-t border-slate-800/80 px-2 flex items-center justify-between select-none z-20 shrink-0">
          
          {/* Start Menu & Search Bar */}
          <div className="flex items-center gap-1.5 h-full">
            {/* Windows Logo Button */}
            <button
              onClick={(e) => {
                e.stopPropagation();
                setIsStartMenuOpen(!isStartMenuOpen);
              }}
              className={`w-9 h-full flex items-center justify-center hover:bg-white/10 rounded transition-colors ${
                isStartMenuOpen ? 'bg-white/15' : ''
              }`}
              title="Iniciar"
            >
              <div className="w-4 h-4 grid grid-cols-2 gap-0.5">
                <div className="bg-[#0078d4] rounded-[1px]"></div>
                <div className="bg-[#0078d4] rounded-[1px]"></div>
                <div className="bg-[#0078d4] rounded-[1px]"></div>
                <div className="bg-[#0078d4] rounded-[1px]"></div>
              </div>
            </button>

            {/* Search Bar Input */}
            <div className="hidden sm:flex items-center gap-2 bg-[#1b253b] border border-slate-700/60 rounded px-2.5 py-1 text-xs text-slate-300 w-48">
              <span className="text-slate-400">🔍</span>
              <span className="text-slate-400 text-[11px]">Pesquisar...</span>
            </div>

            {/* Taskbar Pinned Icons */}
            <div className="flex items-center gap-1 ml-1">
              <button
                onClick={() => onSendNotification('Explorador de Arquivos aberto no host.')}
                className="p-1.5 hover:bg-white/10 rounded transition-colors"
                title="Explorador de Arquivos"
              >
                <Folder className="w-4 h-4 text-amber-400" />
              </button>
              <button
                onClick={() => onSendNotification('Terminal PowerShell aberto no host.')}
                className="p-1.5 hover:bg-white/10 rounded transition-colors"
                title="PowerShell"
              >
                <Terminal className="w-4 h-4 text-blue-400" />
              </button>
              <button
                onClick={() => onSendNotification('Navegador aberto no host.')}
                className="p-1.5 hover:bg-white/10 rounded transition-colors"
                title="Google Chrome"
              >
                <Globe className="w-4 h-4 text-red-400" />
              </button>
              <button
                onClick={() => onSendNotification('Configurações do Windows abertas.')}
                className="p-1.5 hover:bg-white/10 rounded transition-colors"
                title="Configurações"
              >
                <SettingsIcon className="w-4 h-4 text-slate-300" />
              </button>
            </div>
          </div>

          {/* System Tray (Relógio, Idioma, Rede, Áudio) */}
          <div className="flex items-center gap-3 text-xs text-slate-300 px-2 font-mono text-[11px]">
            <span className="text-emerald-400 flex items-center gap-1" title="Serviço PrysmView Host Ativo">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
              PrysmHost
            </span>
            <span className="text-slate-500">POR</span>
            <div className="text-right leading-tight">
              <div className="font-semibold text-slate-200">
                {new Date().toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' })}
              </div>
              <div className="text-[10px] text-slate-400">
                {new Date().toLocaleDateString('pt-BR')}
              </div>
            </div>
          </div>
        </div>

        {/* Windows Start Menu Popup */}
        {isStartMenuOpen && (
          <div
            onClick={(e) => e.stopPropagation()}
            className="absolute left-2 bottom-12 w-80 bg-slate-900 border border-slate-700 rounded-xl shadow-2xl p-4 z-40 text-xs space-y-3 animate-in fade-in slide-in-from-bottom-2 text-slate-200"
          >
            <div className="flex items-center justify-between pb-2 border-b border-slate-800">
              <span className="font-bold text-white text-xs">Aplicativos Fixados</span>
              <span className="text-[10px] text-slate-400">Windows 11 Pro</span>
            </div>
            <div className="grid grid-cols-4 gap-2 text-center text-[10px]">
              <button
                onClick={() => {
                  setIsStartMenuOpen(false);
                  onSendNotification('Terminal aberto.');
                }}
                className="flex flex-col items-center p-2 rounded hover:bg-slate-800"
              >
                <Terminal className="w-6 h-6 text-blue-400 mb-1" />
                <span>Terminal</span>
              </button>
              <button
                onClick={() => {
                  setIsStartMenuOpen(false);
                  onSendNotification('Explorador de Arquivos aberto.');
                }}
                className="flex flex-col items-center p-2 rounded hover:bg-slate-800"
              >
                <Folder className="w-6 h-6 text-amber-400 mb-1" />
                <span>Arquivos</span>
              </button>
              <button
                onClick={() => {
                  setIsStartMenuOpen(false);
                  setShowSysInfoModal(true);
                }}
                className="flex flex-col items-center p-2 rounded hover:bg-slate-800"
              >
                <Laptop className="w-6 h-6 text-emerald-400 mb-1" />
                <span>Info PC</span>
              </button>
              <button
                onClick={() => {
                  setIsStartMenuOpen(false);
                  setShowRebootModal(true);
                }}
                className="flex flex-col items-center p-2 rounded hover:bg-slate-800"
              >
                <RotateCcw className="w-6 h-6 text-amber-400 mb-1" />
                <span>Reiniciar</span>
              </button>
            </div>
          </div>
        )}
      </div>

      {/* =========================================================================
          MODAIS & DIÁLOGOS (CHAT, ARQUIVOS, SAS, REBOOT, SYS INFO)
          ========================================================================= */}

      {/* 1. CHAT DRAWER */}
      {showChat && (
        <div className="absolute right-4 bottom-14 w-80 sm:w-96 h-[440px] bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col z-50 animate-in fade-in slide-in-from-bottom-4">
          <div className="h-10 bg-slate-800 border-b border-slate-700 px-3 flex items-center justify-between rounded-t-xl">
            <div className="flex items-center gap-2 text-xs font-bold text-slate-200">
              <MessageSquare className="w-3.5 h-3.5 text-sky-400" />
              <span>Chat de Suporte em Tempo Real</span>
            </div>
            <button onClick={() => setShowChat(false)} className="text-slate-400 hover:text-white p-1 rounded">
              ✕
            </button>
          </div>

          <div className="flex-1 p-3 overflow-y-auto space-y-2.5 text-xs">
            {chatMessages.map((msg) => (
              <div key={msg.id} className={`flex flex-col ${msg.sender === 'operator' ? 'items-end' : 'items-start'}`}>
                <div className="flex items-baseline gap-1 text-[10px] text-slate-400 mb-0.5">
                  <span className="font-semibold text-slate-300">{msg.senderName}</span>
                  <span>{msg.timestamp}</span>
                </div>
                <div
                  className={`max-w-[85%] px-3 py-2 rounded-xl text-xs ${
                    msg.sender === 'operator'
                      ? 'bg-[#0078d4] text-white rounded-tr-none'
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
              placeholder="Digite uma mensagem para o cliente..."
              value={newMessageText}
              onChange={(e) => setNewMessageText(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSendMessage()}
              className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-[#0078d4]"
            />
            <button
              onClick={handleSendMessage}
              className="p-2 bg-[#0078d4] hover:bg-[#006cbd] text-white rounded-lg transition-colors"
            >
              <Send className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      )}

      {/* 2. FILE TRANSFER MODAL */}
      {showFileTransfer && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-lg shadow-2xl p-5 text-xs text-slate-200">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800 mb-4">
              <div className="flex items-center gap-2">
                <FolderSync className="w-4 h-4 text-emerald-400" />
                <h3 className="font-bold text-sm text-white">Transferência de Arquivos Segura</h3>
              </div>
              <button onClick={() => setShowFileTransfer(false)} className="text-slate-400 hover:text-white">✕</button>
            </div>

            <p className="text-slate-400 mb-3 text-[11px]">
              Envio e recebimento direto via canais de dados WebRTC com hash SHA-256 verificado antes da gravação em disco.
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
                    name: 'patch_suporte_2026.zip',
                    size: 1540000,
                    direction: 'upload',
                    sha256: '4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a',
                    progress: 100,
                    status: 'verified',
                    timestamp: new Date().toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' }),
                  };
                  setTransfers((prev) => [...prev, newF]);
                  onSendNotification('Arquivo enviado e verificado com sucesso!', 'success');
                }}
                className="flex-1 py-2 bg-[#0078d4] hover:bg-[#006cbd] text-white rounded-lg font-semibold transition-colors flex items-center justify-center gap-1.5"
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

      {/* 3. CLIPBOARD MODAL */}
      {showClipboard && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-md shadow-2xl p-5 text-xs text-slate-200">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800 mb-3">
              <div className="flex items-center gap-2">
                <Copy className="w-4 h-4 text-cyan-400" />
                <h3 className="font-bold text-sm text-white">Área de Transferência (Clipboard)</h3>
              </div>
              <button onClick={() => setShowClipboard(false)} className="text-slate-400 hover:text-white">✕</button>
            </div>
            <p className="text-[11px] text-slate-400 mb-2">
              Sincronize texto entre a sua máquina e a área de transferência do Windows remoto (limite máximo de 1 MB).
            </p>
            <textarea
              rows={4}
              value={clipboardText}
              onChange={(e) => setClipboardText(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-[#0078d4] mb-3"
            />
            <div className="flex items-center gap-2">
              <button
                onClick={() => {
                  onSendNotification('Texto enviado para o clipboard remoto com sucesso!', 'success');
                  setShowClipboard(false);
                }}
                className="flex-1 py-2 bg-[#0078d4] hover:bg-[#006cbd] text-white rounded-lg font-semibold transition-colors"
              >
                Colar no Dispositivo Remoto
              </button>
              <button onClick={() => setShowClipboard(false)} className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg">
                Cancelar
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 4. SAS (CTRL+ALT+DEL) CONFIRMATION MODAL */}
      {showSasModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-sm shadow-2xl p-5 text-xs text-slate-200 text-center">
            <div className="w-10 h-10 rounded-full bg-sky-950 border border-sky-800 text-sky-400 flex items-center justify-center mx-auto mb-3">
              <KeyRound className="w-5 h-5" />
            </div>
            <h3 className="font-bold text-sm text-white mb-1">Enviar Ctrl+Alt+Del (SAS)</h3>
            <p className="text-[11px] text-slate-400 mb-4">
              Aciona a Sequência de Atenção Segura (SAS) através do serviço SYSTEM do PrysmView para desbloquear a tela ou abrir o Gerenciador de Tarefas no Windows remoto.
            </p>
            <div className="flex items-center gap-2">
              <button onClick={() => setShowSasModal(false)} className="flex-1 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg">
                Cancelar
              </button>
              <button onClick={handleSendSAS} className="flex-1 py-2 bg-[#0078d4] hover:bg-[#006cbd] text-white rounded-lg font-semibold shadow-md">
                Confirmar SAS
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 5. REBOOT CONFIRMATION MODAL */}
      {showRebootModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-sm shadow-2xl p-5 text-xs text-slate-200 text-center">
            <div className="w-10 h-10 rounded-full bg-amber-950 border border-amber-800 text-amber-400 flex items-center justify-center mx-auto mb-3">
              <AlertTriangle className="w-5 h-5" />
            </div>
            <h3 className="font-bold text-sm text-white mb-1">Reiniciar Computador Remoto</h3>
            <p className="text-[11px] text-slate-400 mb-4">
              O sistema operacional de {device.name} será reiniciado. O serviço do PrysmView Host subirá automaticamente em menos de 10s após o boot com reconexão automática.
            </p>
            <div className="flex items-center gap-2">
              <button onClick={() => setShowRebootModal(false)} className="flex-1 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg">
                Cancelar
              </button>
              <button
                onClick={() => {
                  setShowRebootModal(false);
                  onSendNotification(`Comando shutdown /r /t 5 enviado para ${device.id}. Reconexão engatilhada.`, 'warning');
                }}
                className="flex-1 py-2 bg-amber-600 hover:bg-amber-500 text-white rounded-lg font-semibold shadow-md"
              >
                Reiniciar Agora
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 6. SYSTEM INFO MODAL */}
      {showSysInfoModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-md shadow-2xl p-5 text-xs text-slate-200">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800 mb-3">
              <div className="flex items-center gap-2">
                <Laptop className="w-4 h-4 text-sky-400" />
                <h3 className="font-bold text-sm text-white">Informações da Sessão & Hardware</h3>
              </div>
              <button onClick={() => setShowSysInfoModal(false)} className="text-slate-400 hover:text-white">✕</button>
            </div>

            <div className="space-y-2 mb-4 bg-slate-950 p-3 rounded-lg border border-slate-800 font-mono text-[11px]">
              <div className="flex justify-between">
                <span className="text-slate-400">Dispositivo:</span>
                <span className="text-white">{device.name} ({device.id})</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Sistema Operacional:</span>
                <span className="text-white">{device.os}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Processador:</span>
                <span className="text-white">{device.cpu}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Memória RAM:</span>
                <span className="text-white">{device.memory}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Armazenamento:</span>
                <span className="text-white">{device.disk}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Endereço IP:</span>
                <span className="text-sky-400">{device.ip}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Latência RTT:</span>
                <span className="text-emerald-400">{rtt} ms (Excelente)</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Taxa de Quadros:</span>
                <span className="text-emerald-400">{fps} FPS</span>
              </div>
            </div>

            <button
              onClick={() => setShowSysInfoModal(false)}
              className="w-full py-2 bg-slate-800 hover:bg-slate-700 text-white font-medium rounded-lg"
            >
              Fechar
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
