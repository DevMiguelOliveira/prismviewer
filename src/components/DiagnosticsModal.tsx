import React, { useState, useEffect, useCallback } from 'react';
import {
  Activity,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  RefreshCw,
  Server,
  Radio,
  Monitor,
  Shield,
  HelpCircle,
  Copy,
  Terminal,
  ExternalLink,
  Check,
} from 'lucide-react';
import { getDiagnosticInfo } from '../services/tauriBridge';
import { testSignalingHealth, testWebRTCStun, TestResult } from '../services/signalingService';

interface DiagnosticsModalProps {
  isOpen: boolean;
  onClose: () => void;
  currentEndpoint: string;
  onUpdateEndpoint: (newUrl: string) => void;
}

export const DiagnosticsModal: React.FC<DiagnosticsModalProps> = ({
  isOpen,
  onClose,
  currentEndpoint,
  onUpdateEndpoint,
}) => {
  const [isRunning, setIsRunning] = useState(false);
  const [results, setResults] = useState<TestResult[]>([]);
  const [customUrl, setCustomUrl] = useState(currentEndpoint);
  const [copiedCode, setCopiedCode] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'tests' | 'frontendFix' | 'signalingFix'>('tests');

  const runAllTests = useCallback(async () => {
    setIsRunning(true);
    setResults([]);

    // 1. Teste de Ambiente Webview / Tauri
    const sysInfo = getDiagnosticInfo();
    const tauriTest: TestResult = {
      name: 'Ambiente Webview & IPC',
      status: 'success',
      message: sysInfo.isTauri
        ? 'Executando dentro do container desktop nativo (Tauri)'
        : 'Executando em modo web com camada segura de compatibilidade',
      details: `Motor: ${sysInfo.hasWebRTC ? 'WebRTC OK' : 'Sem WebRTC'}, Crypto: ${sysInfo.hasCrypto ? 'SubtleCrypto OK' : 'Não'}, LocalStorage: ${sysInfo.hasLocalStorage ? 'Persistente' : 'Bloqueado'}`,
    };

    // 2. Teste de Armazenamento
    const storageTest: TestResult = {
      name: 'Persistência de Dados (Local)',
      status: sysInfo.hasLocalStorage ? 'success' : 'error',
      message: sysInfo.hasLocalStorage ? 'Armazenamento local acessível sem restrições' : 'Falha no acesso ao LocalStorage (permissões restritas)',
      details: sysInfo.hasLocalStorage
        ? 'Tokens, credenciais e configurações salvam com sucesso'
        : 'Em serviços Windows (LocalSystem), o WebView2 pode necessitar de diretório de dados explícito.',
      suggestedFix: 'Configure userDataFolder explícito no Tauri para o serviço Windows.',
    };

    setResults([tauriTest, storageTest]);

    // 3. Teste de Sinalização HTTP
    const signalingResult = await testSignalingHealth(customUrl);
    setResults((prev) => [...prev, signalingResult]);

    // 4. Teste de WebRTC e STUN
    const webrtcResult = await testWebRTCStun();
    setResults((prev) => [...prev, webrtcResult]);

    setIsRunning(false);
  }, [customUrl]);

  useEffect(() => {
    if (isOpen) {
      runAllTests();
    }
  }, [isOpen, runAllTests]);

  if (!isOpen) return null;

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard?.writeText(text);
    setCopiedCode(id);
    setTimeout(() => setCopiedCode(null), 2500);
  };

  return (
    <div className="fixed inset-0 bg-black/75 backdrop-blur-sm flex items-center justify-center p-3 sm:p-5 z-50 animate-in fade-in">
      <div className="bg-slate-900 border border-slate-700/80 rounded-2xl w-full max-w-2xl max-h-[90vh] shadow-2xl flex flex-col text-xs text-slate-200 overflow-hidden">
        {/* Header */}
        <div className="h-14 px-5 bg-slate-950 border-b border-slate-800 flex items-center justify-between shrink-0">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-sky-950 border border-sky-800 flex items-center justify-center text-sky-400">
              <Activity className="w-4 h-4" />
            </div>
            <div>
              <h3 className="font-bold text-sm text-white">Central de Diagnóstico & Correção de Erros</h3>
              <p className="text-[11px] text-slate-400">Resolução de falhas de Webview, WebRTC e Sinalização</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 hover:bg-slate-800 rounded-lg text-slate-400 hover:text-white transition-colors"
          >
            ✕
          </button>
        </div>

        {/* Navigation Tabs */}
        <div className="flex items-center bg-slate-950/60 px-5 pt-2 border-b border-slate-800 gap-2 shrink-0">
          <button
            onClick={() => setActiveTab('tests')}
            className={`pb-2 px-3 font-semibold transition-colors border-b-2 text-xs flex items-center gap-1.5 ${
              activeTab === 'tests'
                ? 'border-sky-500 text-sky-400'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Activity className="w-3.5 h-3.5" />
            <span>Diagnóstico ao Vivo</span>
          </button>
          <button
            onClick={() => setActiveTab('signalingFix')}
            className={`pb-2 px-3 font-semibold transition-colors border-b-2 text-xs flex items-center gap-1.5 ${
              activeTab === 'signalingFix'
                ? 'border-sky-500 text-sky-400'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Radio className="w-3.5 h-3.5" />
            <span>Correção: Sinalização & Rede</span>
          </button>
          <button
            onClick={() => setActiveTab('frontendFix')}
            className={`pb-2 px-3 font-semibold transition-colors border-b-2 text-xs flex items-center gap-1.5 ${
              activeTab === 'frontendFix'
                ? 'border-sky-500 text-sky-400'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Monitor className="w-3.5 h-3.5" />
            <span>Correção: Frontend & Webview</span>
          </button>
        </div>

        {/* Content Body */}
        <div className="flex-1 overflow-y-auto p-5 space-y-4">
          {activeTab === 'tests' && (
            <div className="space-y-4">
              {/* Endpoint Selector Bar */}
              <div className="p-3 bg-slate-950 border border-slate-800 rounded-xl space-y-2">
                <label className="text-[11px] font-semibold text-slate-300 block">
                  URL da API de Sinalização para teste:
                </label>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={customUrl}
                    onChange={(e) => setCustomUrl(e.target.value)}
                    className="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white font-mono focus:outline-none focus:border-sky-500"
                    placeholder="https://prysmview.duckdns.org ou http://127.0.0.1:8080"
                  />
                  <button
                    onClick={() => {
                      onUpdateEndpoint(customUrl);
                      runAllTests();
                    }}
                    disabled={isRunning}
                    className="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 disabled:bg-slate-800 text-white font-semibold rounded-lg text-xs flex items-center gap-1.5 shadow-sm transition-colors shrink-0"
                  >
                    <RefreshCw className={`w-3.5 h-3.5 ${isRunning ? 'animate-spin' : ''}`} />
                    <span>{isRunning ? 'Testando...' : 'Reexecutar'}</span>
                  </button>
                </div>
                <div className="flex items-center gap-2 text-[10px] text-slate-400">
                  <span>Atalhos rápidos:</span>
                  <button
                    type="button"
                    onClick={() => {
                      setCustomUrl('https://prysmview.duckdns.org');
                      onUpdateEndpoint('https://prysmview.duckdns.org');
                    }}
                    className="text-emerald-400 underline font-mono font-semibold"
                  >
                    https://prysmview.duckdns.org (Produção Azure)
                  </button>
                  <span>•</span>
                  <button
                    type="button"
                    onClick={() => {
                      setCustomUrl('http://prysmview.duckdns.org');
                      onUpdateEndpoint('http://prysmview.duckdns.org');
                    }}
                    className="text-sky-400 underline font-mono"
                  >
                    http://prysmview.duckdns.org
                  </button>
                  <span>•</span>
                  <button
                    type="button"
                    onClick={() => {
                      setCustomUrl('http://127.0.0.1:8080');
                      onUpdateEndpoint('http://127.0.0.1:8080');
                    }}
                    className="text-sky-400 underline font-mono"
                  >
                    Localhost:8080
                  </button>
                </div>
              </div>

              {/* Test Results List */}
              <div className="space-y-2.5">
                {results.map((r, idx) => (
                  <div
                    key={idx}
                    className={`p-3.5 rounded-xl border transition-all ${
                      r.status === 'success'
                        ? 'bg-slate-950/80 border-emerald-900/60'
                        : r.status === 'warning'
                        ? 'bg-slate-950/80 border-amber-900/60'
                        : 'bg-slate-950/80 border-red-900/60'
                    }`}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex items-center gap-2">
                        {r.status === 'success' ? (
                          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                        ) : r.status === 'warning' ? (
                          <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />
                        ) : (
                          <XCircle className="w-4 h-4 text-red-400 shrink-0" />
                        )}
                        <div>
                          <span className="font-bold text-slate-100 text-xs">{r.name}</span>
                          <p className="text-[11px] text-slate-300 font-medium">{r.message}</p>
                        </div>
                      </div>

                      {r.latencyMs !== undefined && (
                        <span className="font-mono text-[10px] px-2 py-0.5 rounded bg-slate-900 border border-slate-800 text-slate-300">
                          {r.latencyMs} ms
                        </span>
                      )}
                    </div>

                    {r.details && (
                      <p className="mt-2 text-[10px] font-mono text-slate-400 bg-slate-900/90 p-2 rounded-lg border border-slate-800/80">
                        {r.details}
                      </p>
                    )}

                    {r.suggestedFix && (
                      <div className="mt-2 text-[11px] text-amber-300/90 bg-amber-950/40 p-2 rounded-lg border border-amber-800/40 flex items-start gap-1.5">
                        <HelpCircle className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
                        <div>
                          <strong>Como corrigir:</strong> {r.suggestedFix}
                        </div>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'signalingFix' && (
            <div className="space-y-4">
              <div className="p-3.5 bg-slate-950 border border-slate-800 rounded-xl space-y-2">
                <h4 className="font-bold text-slate-100 text-xs flex items-center gap-1.5 text-sky-400">
                  <Radio className="w-4 h-4" />
                  <span>Causas Comuns de Erro de Sinalização & Conexão</span>
                </h4>
                <p className="text-slate-400 text-[11px] leading-relaxed">
                  O erro de sinalização ocorre quando o app desktop não consegue se comunicar com o servidor para trocar os pacotes SDP e candidatos ICE da conexão P2P.
                </p>
              </div>

              {/* Fix 1: VM Azure Firewall & Portas */}
              <div className="p-3.5 bg-slate-950 border border-slate-800 rounded-xl space-y-2">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-slate-200">1. Verificar Serviços e Portas na Azure VM</span>
                  <button
                    onClick={() =>
                      copyToClipboard(
                        'sudo systemctl status prysmview-api\nsudo systemctl status caddy\nsudo ufw allow 443/tcp\nsudo ufw allow 80/tcp\nsudo ufw allow 3340/udp',
                        'azure-cmd'
                      )
                    }
                    className="flex items-center gap-1 text-[10px] text-sky-400 hover:text-sky-300 font-semibold"
                  >
                    {copiedCode === 'azure-cmd' ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                    <span>Copiar Comandos</span>
                  </button>
                </div>
                <pre className="p-2.5 rounded bg-slate-900 border border-slate-800 font-mono text-[10px] text-emerald-400 overflow-x-auto">
{`# 1. Verificar se a API Go e o Caddy estão ativos na VM:
sudo systemctl status prysmview-api
sudo systemctl status caddy

# 2. Liberar portas no Firewall UFW:
sudo ufw allow 443/tcp
sudo ufw allow 80/tcp
sudo ufw allow 3340/udp`}
                </pre>
              </div>

              {/* Fix 2: DuckDNS IP Sync */}
              <div className="p-3.5 bg-slate-950 border border-slate-800 rounded-xl space-y-2">
                <span className="font-bold text-slate-200">2. Atualizar IP do Domínio DuckDNS</span>
                <p className="text-slate-400 text-[11px]">
                  Se a máquina virtual na Azure foi reiniciada, o IP público (20.121.137.44) pode ter mudado.
                  Acesse <a href="https://www.duckdns.org" target="_blank" rel="noreferrer" className="text-sky-400 underline">duckdns.org</a> e aponte o domínio <code>prysmview</code> para o IP atual da sua VM.
                </p>
              </div>
            </div>
          )}

          {activeTab === 'frontendFix' && (
            <div className="space-y-4">
              <div className="p-3.5 bg-slate-950 border border-slate-800 rounded-xl space-y-2">
                <h4 className="font-bold text-slate-100 text-xs flex items-center gap-1.5 text-sky-400">
                  <Monitor className="w-4 h-4" />
                  <span>Causas Comuns de Erro no Frontend / WebView2</span>
                </h4>
                <p className="text-slate-400 text-[11px] leading-relaxed">
                  No Tauri, o WebView2 do Windows pode exibir tela branca ou acusar erro de JavaScript por dois motivos principais:
                </p>
              </div>

              {/* Fix 1: CSP no tauri.conf.json */}
              <div className="p-3.5 bg-slate-950 border border-slate-800 rounded-xl space-y-2">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-slate-200">1. Política de Segurança (CSP) no tauri.conf.json</span>
                  <button
                    onClick={() =>
                      copyToClipboard(
                        `"security": {\n  "csp": "default-src 'self'; connect-src 'self' https://*.duckdns.org wss://*.duckdns.org https://prysmview.duckdns.org http://localhost:* ws://localhost:*;"\n}`,
                        'csp-cmd'
                      )
                    }
                    className="flex items-center gap-1 text-[10px] text-sky-400 hover:text-sky-300 font-semibold"
                  >
                    {copiedCode === 'csp-cmd' ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                    <span>Copiar CSP</span>
                  </button>
                </div>
                <p className="text-[11px] text-slate-400">
                  O WebView2 bloqueia requisições <code>fetch()</code> e <code>WebSocket</code> para a sua API se a URL não estiver declarada no Content Security Policy do Tauri:
                </p>
                <pre className="p-2.5 rounded bg-slate-900 border border-slate-800 font-mono text-[10px] text-sky-300 overflow-x-auto">
{`"security": {
  "csp": "default-src 'self'; connect-src 'self' https://*.duckdns.org wss://*.duckdns.org http://localhost:* ws://localhost:*;"
}`}
                </pre>
              </div>

              {/* Fix 2: Global Tauri Invoke */}
              <div className="p-3.5 bg-slate-950 border border-slate-800 rounded-xl space-y-2">
                <span className="font-bold text-slate-200">2. Erro "Cannot read properties of undefined (reading 'invoke')"</span>
                <p className="text-[11px] text-slate-400">
                  Se a interface tentar chamar <code>window.__TAURI__.invoke</code> antes do Tauri injetar os scripts nativos, a aplicação trava com tela branca.
                  Já integramos a biblioteca de <strong>tauriBridge seguro</strong> nesta versão, garantindo fallback gracioso automático caso o runtime esteja inicializando.
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="h-12 px-5 bg-slate-950 border-t border-slate-800 flex items-center justify-between shrink-0 text-[11px]">
          <span className="text-slate-400">Status atual: {currentEndpoint}</span>
          <button
            onClick={onClose}
            className="px-4 py-1.5 bg-sky-600 hover:bg-sky-500 text-white font-bold rounded-lg transition-colors"
          >
            Concluído
          </button>
        </div>
      </div>
    </div>
  );
};
