import React, { useState, useEffect } from 'react';
import {
  Terminal,
  Shield,
  Copy,
  Check,
  RefreshCw,
  Lock,
  Eye,
  EyeOff,
  AlertOctagon,
  Clock,
  CheckCircle2,
  XCircle,
  HelpCircle,
  Sliders,
} from 'lucide-react';

interface HostTerminalProps {
  onSendNotification: (msg: string, type?: 'success' | 'warning' | 'error') => void;
}

export const HostTerminal: React.FC<HostTerminalProps> = ({ onSendNotification }) => {
  // Device Identity
  const [hostDeviceId, setHostDeviceId] = useState('DESKTOP-CORP-01');
  const [localIp, setLocalIp] = useState('192.168.1.101');
  const [copiedId, setCopiedId] = useState(false);

  // 8-digit temporary session code
  const [sessionCode, setSessionCode] = useState('84920153');
  const [codeCountdown, setCodeCountdown] = useState(115);
  const [copiedCode, setCopiedCode] = useState(false);

  // Unattended password
  const [unattendedPassword, setUnattendedPassword] = useState('••••••••');
  const [showPassword, setShowPassword] = useState(false);
  const [showEditPassModal, setShowEditPassModal] = useState(false);
  const [newPasswordInput, setNewPasswordInput] = useState('');

  // Security Toggles
  const [allowClipboard, setAllowClipboard] = useState(true);
  const [allowFiles, setAllowFiles] = useState(true);

  // Active Session simulation on host
  const [isSessionActive, setIsSessionActive] = useState(false);
  const [connectedOperator, setConnectedOperator] = useState('Administrador Geral (TECH-01)');

  // Incoming Consent Dialog simulation
  const [showConsentDialog, setShowConsentDialog] = useState(false);
  const [consentCountdown, setConsentCountdown] = useState(60);
  const [requestingOperator, setRequestingOperator] = useState('Carlos Eduardo Silva (N2 - Suporte)');
  const [requestReason, setRequestReason] = useState('Diagnóstico de performance e manutenção preventiva de sistema');

  // Timer loop for 8-digit code expiry (120s)
  useEffect(() => {
    const timer = setInterval(() => {
      setCodeCountdown((prev) => {
        if (prev <= 1) {
          // Auto generate new code on expiry
          const next = Math.floor(10000000 + Math.random() * 90000000).toString();
          setSessionCode(next);
          return 120;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  // Timer loop for consent dialog (60s)
  useEffect(() => {
    if (!showConsentDialog) return;
    const timer = setInterval(() => {
      setConsentCountdown((prev) => {
        if (prev <= 1) {
          setShowConsentDialog(false);
          onSendNotification('Solicitação de acesso recusada automaticamente por tempo limite (60s).', 'warning');
          return 60;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(timer);
  }, [showConsentDialog, onSendNotification]);

  const handleCopyId = () => {
    navigator.clipboard?.writeText(hostDeviceId);
    setCopiedId(true);
    setTimeout(() => setCopiedId(false), 2000);
    onSendNotification(`ID ${hostDeviceId} copiada para a área de transferência!`, 'success');
  };

  const handleCopyCode = () => {
    navigator.clipboard?.writeText(sessionCode);
    setCopiedCode(true);
    setTimeout(() => setCopiedCode(false), 2000);
    onSendNotification(`Código ${sessionCode} copiado!`, 'success');
  };

  const handleGenerateNewCode = () => {
    const next = Math.floor(10000000 + Math.random() * 90000000).toString();
    setSessionCode(next);
    setCodeCountdown(120);
    onSendNotification('Novo código de sessão de 8 dígitos gerado com sucesso.', 'success');
  };

  const handleAcceptConsent = () => {
    setShowConsentDialog(false);
    setIsSessionActive(true);
    setConnectedOperator(requestingOperator);
    onSendNotification(`Acesso remoto aceito para ${requestingOperator}. Sessão segura iniciada.`, 'success');
  };

  const handleRejectConsent = () => {
    setShowConsentDialog(false);
    onSendNotification('Acesso remoto recusado pelo operador local.', 'warning');
  };

  const handleTerminateSession = () => {
    setIsSessionActive(false);
    onSendNotification('Sessão remota encerrada imediatamente pelo usuário local.', 'warning');
  };

  return (
    <div className="max-w-xl mx-auto py-4 px-2 animate-in fade-in duration-200">
      {/* Simulation Trigger Bar for Testing */}
      <div className="mb-4 p-3 bg-slate-900 border border-slate-800 rounded-xl flex items-center justify-between text-xs">
        <span className="text-slate-400">
          Simular chamada remota de um técnico para ver a janela de consentimento:
        </span>
        <button
          onClick={() => {
            setShowConsentDialog(true);
            setConsentCountdown(60);
          }}
          className="px-2.5 py-1 bg-emerald-600 hover:bg-emerald-500 text-white font-semibold rounded-lg text-xs transition-colors shrink-0"
        >
          Simular Chamada
        </button>
      </div>

      {/* Main Window (TeamViewer Host style) */}
      <div className="bg-slate-900 border border-slate-700/80 rounded-2xl shadow-2xl overflow-hidden text-xs">
        {/* Banner Header */}
        <header className="bg-gradient-to-r from-sky-900 via-blue-900 to-slate-900 px-5 py-4 border-b border-sky-800/60 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-white/10 border border-white/20 flex items-center justify-center text-white shadow-inner">
              <svg className="w-6 h-6" viewBox="0 0 24 24" fill="none">
                <polygon points="12,2 22,8.5 22,15.5 12,22 2,15.5 2,8.5" stroke="#ffffff" strokeWidth="2" fill="rgba(255, 255, 255, 0.2)" />
                <polyline points="12,2 12,22" stroke="#ffffff" strokeWidth="1.5" opacity="0.8" />
                <polyline points="2,8.5 12,14 22,8.5" stroke="#ffffff" strokeWidth="1.5" opacity="0.8" />
              </svg>
            </div>
            <div>
              <h1 className="text-base font-extrabold text-white tracking-wide">PrysmView Host</h1>
              <p className="text-[11px] text-sky-200">Agente de Suporte Remoto Desktop (Windows 10/11)</p>
            </div>
          </div>

          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-slate-950/60 border border-sky-600/40 text-[10px] text-sky-300 font-mono">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            <span>Serviço SYSTEM Ativo</span>
          </div>
        </header>

        {/* Content Body */}
        <main className="p-5 space-y-4">
          {/* Active Session Warning Banner (when connected) */}
          {isSessionActive && (
            <div className="p-4 rounded-xl bg-red-950/80 border border-red-700/80 text-red-200 space-y-2.5 animate-in slide-in-from-top-2">
              <div className="flex items-center gap-2 text-xs font-bold text-red-300">
                <span className="pulse-dot-red"></span>
                <span>SESSÃO REMOTA EM ANDAMENTO</span>
              </div>
              <div className="text-[11px] flex justify-between text-slate-300">
                <span>Técnico Conectado:</span>
                <span className="font-semibold text-white">{connectedOperator}</span>
              </div>
              <div className="text-[10px] text-red-400 flex items-center gap-1">
                <Shield className="w-3 h-3 text-red-400" />
                <span>Conexão Segura Criptografada (QUIC / TLS 1.3)</span>
              </div>
              <button
                onClick={handleTerminateSession}
                className="w-full py-2 bg-red-600 hover:bg-red-500 text-white font-bold rounded-lg text-xs transition-colors flex items-center justify-center gap-1.5 shadow-md shadow-red-600/30"
              >
                <AlertOctagon className="w-4 h-4" />
                <span>Encerrar Acesso Imediatamente</span>
              </button>
            </div>
          )}

          {/* Permitir Suporte Remoto Card */}
          <div className="bg-slate-950/70 border border-slate-800 rounded-xl p-4 space-y-4">
            <div className="flex items-center justify-between pb-2 border-b border-slate-800">
              <h2 className="font-bold text-sm text-slate-100">Permitir Suporte Remoto</h2>
              <span className="text-[11px] text-slate-400 flex items-center gap-1 font-mono">
                <span className="w-2 h-2 rounded-full bg-emerald-400"></span>
                {localIp}
              </span>
            </div>

            <p className="text-slate-400 text-[11px]">
              Informe a <strong>Sua ID</strong> e o <strong>Código Temporário</strong> ao técnico de suporte da central:
            </p>

            {/* Sua ID Row */}
            <div className="p-3 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-between">
              <div>
                <span className="px-1.5 py-0.5 rounded bg-sky-950 border border-sky-800 text-sky-400 font-bold text-[9px] uppercase tracking-wider">
                  SUA ID
                </span>
                <div className="text-base font-extrabold font-mono text-white mt-0.5">
                  {hostDeviceId}
                </div>
              </div>
              <button
                onClick={handleCopyId}
                className="flex items-center gap-1 px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg text-xs font-semibold border border-slate-700 transition-colors"
              >
                {copiedId ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copiedId ? 'Copiado!' : 'Copiar'}</span>
              </button>
            </div>

            {/* Código de 8 Dígitos Box */}
            <div className="p-4 rounded-xl bg-slate-900/90 border border-slate-700/80 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-[11px] font-bold text-slate-300 uppercase tracking-wide">
                  Código de Acesso Rápido (8 Dígitos)
                </span>
                <span className="text-[10px] text-emerald-400 bg-emerald-950/80 border border-emerald-800/80 px-2 py-0.5 rounded-full font-mono">
                  Uso Único
                </span>
              </div>

              <div className="py-2.5 px-4 rounded-lg bg-slate-950 border border-slate-800 text-center">
                <span className="text-2xl font-black font-mono tracking-[0.25em] text-emerald-400 selection:bg-emerald-500">
                  {sessionCode}
                </span>
              </div>

              <div className="flex items-center gap-2">
                <button
                  onClick={handleGenerateNewCode}
                  className="flex-1 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg text-xs font-semibold border border-slate-700 transition-colors flex items-center justify-center gap-1"
                >
                  <RefreshCw className="w-3 h-3 text-sky-400" />
                  <span>Gerar Novo Código</span>
                </button>
                <button
                  onClick={handleCopyCode}
                  className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg text-xs font-semibold border border-slate-700 transition-colors flex items-center justify-center gap-1"
                >
                  {copiedCode ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                  <span>{copiedCode ? 'Copiado' : 'Copiar'}</span>
                </button>
              </div>

              <div className="text-[10px] text-slate-500 flex items-center justify-between pt-1">
                <span className="flex items-center gap-1 text-slate-400 font-mono">
                  <Clock className="w-3 h-3 text-amber-400" />
                  Expira em {codeCountdown}s
                </span>
                <span>Máximo 3 tentativas seguras</span>
              </div>
            </div>

            {/* Acesso Desassistido Box */}
            <div className="p-3 rounded-lg bg-slate-900 border border-slate-800 space-y-1.5">
              <div className="flex items-center justify-between">
                <span className="text-[11px] font-bold text-slate-300">Acesso Desassistido (Senha Fixa)</span>
                <div className="flex items-center gap-1.5">
                  <button
                    onClick={() => setShowPassword(!showPassword)}
                    className="p-1 hover:bg-slate-800 rounded text-slate-400 hover:text-white"
                    title={showPassword ? 'Ocultar senha' : 'Ver senha'}
                  >
                    {showPassword ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5 text-sky-400" />}
                  </button>
                  <button
                    onClick={() => {
                      setNewPasswordInput(unattendedPassword);
                      setShowEditPassModal(true);
                    }}
                    className="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-[10px] border border-slate-700 font-semibold"
                  >
                    Configurar
                  </button>
                </div>
              </div>

              <div className="font-mono text-xs text-amber-400">
                {showPassword ? unattendedPassword : '••••••••••••••••'}
              </div>
            </div>

            {/* Security Toggles */}
            <div className="grid grid-cols-2 gap-2 pt-1">
              <label className="flex items-center gap-2 p-2 rounded-lg bg-slate-900 border border-slate-800 cursor-pointer">
                <input
                  type="checkbox"
                  checked={allowClipboard}
                  onChange={(e) => setAllowClipboard(e.target.checked)}
                  className="rounded text-sky-600 focus:ring-0"
                />
                <span className="text-[11px] text-slate-300">Clipboard (Ctrl+C/V)</span>
              </label>

              <label className="flex items-center gap-2 p-2 rounded-lg bg-slate-900 border border-slate-800 cursor-pointer">
                <input
                  type="checkbox"
                  checked={allowFiles}
                  onChange={(e) => setAllowFiles(e.target.checked)}
                  className="rounded text-sky-600 focus:ring-0"
                />
                <span className="text-[11px] text-slate-300">Arquivos (SHA-256)</span>
              </label>
            </div>
          </div>
        </main>

        {/* Footer */}
        <footer className="bg-slate-950 px-5 py-3 border-t border-slate-800 flex items-center justify-between text-[11px] text-slate-400">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-400"></span>
            <span>Pronto para conectar (conexão segura P2P)</span>
          </div>
          <span className="font-mono text-[10px] text-slate-500">PrysmView v2.0.0</span>
        </footer>
      </div>

      {/* =========================================================================
          CONSENT DIALOG MODAL (60 SECONDS TIMEOUT)
          ========================================================================= */}
      {showConsentDialog && (
        <div className="fixed inset-0 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border-2 border-sky-500 rounded-2xl w-full max-w-sm shadow-2xl p-5 text-xs text-slate-200 text-center animate-in zoom-in-95">
            <div className="w-12 h-12 rounded-full bg-sky-950 border border-sky-800 text-sky-400 flex items-center justify-center mx-auto mb-3">
              <Shield className="w-6 h-6" />
            </div>

            <h3 className="font-extrabold text-base text-white mb-1">
              Solicitação de Suporte Remoto
            </h3>
            <p className="text-[11px] text-slate-400 mb-4">
              Um operador técnico solicitou acesso à tela deste computador:
            </p>

            <div className="bg-slate-950 p-3 rounded-xl border border-slate-800 text-left space-y-1.5 mb-4 text-[11px]">
              <div>
                <span className="text-slate-500">Técnico:</span>{' '}
                <strong className="text-slate-200">{requestingOperator}</strong>
              </div>
              <div>
                <span className="text-slate-500">Motivo:</span>{' '}
                <span className="text-sky-300">{requestReason}</span>
              </div>
              <div>
                <span className="text-slate-500">Tempo para resposta:</span>{' '}
                <strong className="text-amber-400 font-mono">{consentCountdown} segundos</strong>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <button
                onClick={handleRejectConsent}
                className="flex-1 py-2.5 bg-slate-800 hover:bg-slate-700 text-slate-300 font-semibold rounded-xl text-xs transition-colors flex items-center justify-center gap-1.5"
              >
                <XCircle className="w-4 h-4 text-red-400" />
                <span>Recusar</span>
              </button>
              <button
                onClick={handleAcceptConsent}
                className="flex-1 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-xl text-xs transition-colors shadow-lg shadow-emerald-600/30 flex items-center justify-center gap-1.5"
              >
                <CheckCircle2 className="w-4 h-4 text-white" />
                <span>Aceitar Acesso</span>
              </button>
            </div>
          </div>
        </div>
      )}

      {/* EDIT UNATTENDED PASSWORD MODAL */}
      {showEditPassModal && (
        <div className="fixed inset-0 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-sm shadow-2xl p-5 text-xs text-slate-200">
            <h3 className="font-bold text-sm text-white mb-2">Configurar Senha Desassistida</h3>
            <p className="text-[11px] text-slate-400 mb-3">
              Defina a senha fixa para acessos remotos sem necessidade de intervenção do usuário local:
            </p>
            <input
              type="text"
              value={newPasswordInput}
              onChange={(e) => setNewPasswordInput(e.target.value)}
              className="w-full bg-slate-950 border border-slate-700 rounded-lg p-2 text-xs font-mono text-amber-400 mb-4 focus:outline-none focus:border-amber-500"
            />
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowEditPassModal(false)}
                className="flex-1 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg"
              >
                Cancelar
              </button>
              <button
                onClick={() => {
                  setUnattendedPassword(newPasswordInput);
                  setShowEditPassModal(false);
                  onSendNotification('Senha de acesso desassistido atualizada no DPAPI com sucesso.', 'success');
                }}
                className="flex-1 py-2 bg-amber-600 hover:bg-amber-500 text-white font-bold rounded-lg"
              >
                Salvar Senha
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
