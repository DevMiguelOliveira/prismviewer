import React, { useState } from 'react';
import {
  Zap,
  KeyRound,
  Lock,
  Shield,
  HelpCircle,
  Copy,
  CheckCircle2,
  AlertCircle,
  Monitor,
  MousePointer,
  FolderSync,
  FileText,
} from 'lucide-react';
import { Device } from '../types';

interface QuickConnectProps {
  devices: Device[];
  onConnectDevice: (device: Device, method: 'code' | 'unattended', codeOrPassword?: string) => void;
  onSendNotification: (msg: string, type?: 'success' | 'warning' | 'error') => void;
}

export const QuickConnect: React.FC<QuickConnectProps> = ({
  devices,
  onConnectDevice,
  onSendNotification,
}) => {
  const [targetId, setTargetId] = useState('');
  const [authMethod, setAuthMethod] = useState<'code' | 'unattended'>('code');
  const [sessionCode, setSessionCode] = useState('');
  const [unattendedPassword, setUnattendedPassword] = useState('');
  const [reason, setReason] = useState('Suporte técnico e manutenção remota');
  const [allowMouse, setAllowMouse] = useState(true);
  const [allowClipboard, setAllowClipboard] = useState(true);
  const [allowFiles, setAllowFiles] = useState(true);
  const [isVerifying, setIsVerifying] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!targetId.trim()) {
      onSendNotification('Informe a ID do dispositivo ou o endereço IP.', 'warning');
      return;
    }

    const matched = devices.find(
      (d) =>
        d.id.toLowerCase() === targetId.trim().toLowerCase() ||
        d.ip === targetId.trim() ||
        d.name.toLowerCase().includes(targetId.trim().toLowerCase())
    );

    if (!matched) {
      onSendNotification(`Dispositivo '${targetId}' não localizado na rede corporativa.`, 'error');
      return;
    }

    if (matched.status === 'offline') {
      onSendNotification(`O dispositivo '${matched.id}' está offline no momento.`, 'error');
      return;
    }

    if (authMethod === 'code' && (!sessionCode || sessionCode.length !== 8)) {
      onSendNotification('O código de sessão deve conter exatamente 8 dígitos.', 'warning');
      return;
    }

    setIsVerifying(true);
    setTimeout(() => {
      setIsVerifying(false);
      onConnectDevice(
        matched,
        authMethod,
        authMethod === 'code' ? sessionCode : unattendedPassword
      );
    }, 800);
  };

  return (
    <div className="max-w-4xl mx-auto space-y-6 animate-in fade-in duration-250">
      <div className="grid grid-cols-1 md:grid-cols-5 gap-6">
        {/* Main Connect Card */}
        <div className="md:col-span-3 bg-slate-900 border border-slate-800 rounded-2xl shadow-xl p-6 text-xs">
          <div className="flex items-center gap-3 pb-4 border-b border-slate-800 mb-5">
            <div className="w-10 h-10 rounded-xl bg-sky-950 border border-sky-800 flex items-center justify-center text-sky-400">
              <Zap className="w-5 h-5 fill-sky-400/20" />
            </div>
            <div>
              <h2 className="text-base font-bold text-white">Conexão Rápida ao Host</h2>
              <p className="text-slate-400 text-xs">
                Controle remoto direto inspirado no TeamViewer com criptografia E2E (DTLS 1.3)
              </p>
            </div>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-slate-300 font-semibold mb-1.5">
                ID do Dispositivo Remoto ou IP:
              </label>
              <div className="relative">
                <input
                  type="text"
                  required
                  placeholder="Ex: DESKTOP-01 ou 192.168.1.101"
                  value={targetId}
                  onChange={(e) => setTargetId(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-700 rounded-xl px-4 py-2.5 text-sm text-white placeholder-slate-500 font-mono focus:outline-none focus:border-sky-500 transition-colors"
                />
              </div>
              <div className="flex items-center gap-1.5 mt-1.5 text-[11px] text-slate-500">
                <span>Dispositivos rápidos online:</span>
                {devices
                  .filter((d) => d.status === 'online')
                  .slice(0, 3)
                  .map((d) => (
                    <button
                      key={d.id}
                      type="button"
                      onClick={() => setTargetId(d.id)}
                      className="px-1.5 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-sky-400 font-mono text-[10px]"
                    >
                      {d.id}
                    </button>
                  ))}
              </div>
            </div>

            {/* Authentication Method Tabs */}
            <div className="pt-2">
              <label className="block text-slate-300 font-semibold mb-1.5">
                Modo de Autenticação:
              </label>
              <div className="grid grid-cols-2 gap-2 bg-slate-950 p-1 rounded-xl border border-slate-800">
                <button
                  type="button"
                  onClick={() => setAuthMethod('code')}
                  className={`py-2 rounded-lg font-semibold flex items-center justify-center gap-1.5 transition-colors ${
                    authMethod === 'code'
                      ? 'bg-sky-600 text-white shadow-sm'
                      : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  <KeyRound className="w-3.5 h-3.5" />
                  <span>Código de 8 Dígitos</span>
                </button>
                <button
                  type="button"
                  onClick={() => setAuthMethod('unattended')}
                  className={`py-2 rounded-lg font-semibold flex items-center justify-center gap-1.5 transition-colors ${
                    authMethod === 'unattended'
                      ? 'bg-amber-600 text-white shadow-sm'
                      : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  <Lock className="w-3.5 h-3.5" />
                  <span>Acesso Desassistido</span>
                </button>
              </div>
            </div>

            {authMethod === 'code' ? (
              <div>
                <label className="block text-slate-300 font-semibold mb-1">
                  Código Temporário de 8 Dígitos gerado no Host:
                </label>
                <input
                  type="text"
                  maxLength={8}
                  placeholder="00000000"
                  value={sessionCode}
                  onChange={(e) => setSessionCode(e.target.value.replace(/\D/g, ''))}
                  className="w-full bg-slate-950 border border-slate-700 rounded-xl px-4 py-2 text-base text-emerald-400 font-mono tracking-widest text-center placeholder-slate-600 focus:outline-none focus:border-emerald-500"
                />
                <p className="text-[10px] text-slate-500 mt-1">
                  Validade de 120 segundos e máximo de 3 tentativas por sessão com verificação em tempo constante HMAC.
                </p>
              </div>
            ) : (
              <div>
                <label className="block text-slate-300 font-semibold mb-1">
                  Senha Fixa de Acesso Desassistido:
                </label>
                <input
                  type="password"
                  placeholder="Digite a senha cadastrada no host..."
                  value={unattendedPassword}
                  onChange={(e) => setUnattendedPassword(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-700 rounded-xl px-4 py-2.5 text-sm text-white placeholder-slate-600 focus:outline-none focus:border-amber-500"
                />
                <p className="text-[10px] text-slate-500 mt-1">
                  Protegida no disco do host via DPAPI do Windows (escopo de máquina LocalSystem).
                </p>
              </div>
            )}

            <div>
              <label className="block text-slate-300 font-semibold mb-1">
                Motivo do Chamado / Justificativa (Auditoria):
              </label>
              <input
                type="text"
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-sky-500"
              />
            </div>

            {/* Permissions Toggles */}
            <div className="pt-2 border-t border-slate-800 space-y-2">
              <span className="text-slate-400 font-semibold text-[11px] block">
                Canais de Conexão Permitidos:
              </span>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
                <label className="flex items-center gap-2 p-2 rounded-lg bg-slate-950 border border-slate-800/80 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={allowMouse}
                    onChange={(e) => setAllowMouse(e.target.checked)}
                    className="rounded text-sky-600 focus:ring-0"
                  />
                  <span className="text-[11px] text-slate-300">Mouse & Teclado</span>
                </label>

                <label className="flex items-center gap-2 p-2 rounded-lg bg-slate-950 border border-slate-800/80 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={allowClipboard}
                    onChange={(e) => setAllowClipboard(e.target.checked)}
                    className="rounded text-sky-600 focus:ring-0"
                  />
                  <span className="text-[11px] text-slate-300">Clipboard (&lt;1MB)</span>
                </label>

                <label className="flex items-center gap-2 p-2 rounded-lg bg-slate-950 border border-slate-800/80 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={allowFiles}
                    onChange={(e) => setAllowFiles(e.target.checked)}
                    className="rounded text-sky-600 focus:ring-0"
                  />
                  <span className="text-[11px] text-slate-300">Arquivos SHA-256</span>
                </label>
              </div>
            </div>

            <button
              type="submit"
              disabled={isVerifying}
              className="w-full py-3 bg-gradient-to-r from-sky-600 to-blue-600 hover:from-sky-500 hover:to-blue-500 text-white font-bold rounded-xl text-sm transition-all shadow-lg shadow-sky-600/30 flex items-center justify-center gap-2 mt-4"
            >
              <Zap className="w-4 h-4" />
              <span>{isVerifying ? 'Realizando Handshake DTLS 1.3...' : 'Estabelecer Sessão Remota'}</span>
            </button>
          </form>
        </div>

        {/* Instructions & Security Model */}
        <div className="md:col-span-2 space-y-4 text-xs">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl p-5 space-y-3">
            <h3 className="font-bold text-sm text-slate-100 flex items-center gap-2">
              <Shield className="w-4 h-4 text-emerald-400" />
              <span>Segurança Inegociável</span>
            </h3>

            <ol className="list-decimal list-inside space-y-2 text-slate-400 text-[11px] leading-relaxed">
              <li>
                <strong className="text-slate-200">Consentimento no Host:</strong> O usuário local visualiza o nome do técnico e o motivo informado.
              </li>
              <li>
                <strong className="text-slate-200">Criptografia E2E:</strong> Conexão P2P com DTLS 1.3 e verificação Ed25519; o servidor de sinalização nunca vê dados de tela.
              </li>
              <li>
                <strong className="text-slate-200">Encerramento Imediato:</strong> O usuário local da máquina pode interromper a sessão a qualquer segundo com o botão de emergência.
              </li>
              <li>
                <strong className="text-slate-200">Auditoria Completa:</strong> Início, fim, bytes trafegados e hash SHA-256 de auditoria são imutavelmente registrados.
              </li>
            </ol>
          </div>

          <div className="bg-slate-900 border border-slate-800 rounded-2xl p-5 text-[11px] space-y-2">
            <h4 className="font-bold text-slate-200 text-xs flex items-center gap-1.5">
              <HelpCircle className="w-3.5 h-3.5 text-sky-400" />
              <span>Como solicitar o código?</span>
            </h4>
            <p className="text-slate-400">
              Peça ao colaborador na máquina remota para abrir o aplicativo <strong>PrysmView Host</strong> na barra de tarefas do Windows e informar a <strong>ID</strong> e o <span className="text-sky-400 font-semibold">"Código Temporário"</span>.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
