import React, { useState } from 'react';
import {
  Shield,
  Lock,
  Mail,
  Eye,
  EyeOff,
  ArrowRight,
  CheckCircle2,
  Server,
  Zap,
  Terminal,
  KeyRound,
  Sparkles,
  UserCheck,
} from 'lucide-react';
import { Operator } from '../types';
import { INITIAL_OPERATORS } from '../data/mockData';

interface LoginProps {
  onLoginSuccess: (operator: Operator) => void;
  serverUrl?: string;
}

export const Login: React.FC<LoginProps> = ({
  onLoginSuccess,
  serverUrl = 'https://prysmview.duckdns.org',
}) => {
  const [emailOrUser, setEmailOrUser] = useState('admin@prysmview.com');
  const [password, setPassword] = useState('admin123');
  const [showPassword, setShowPassword] = useState(false);
  const [rememberMe, setRememberMe] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMessage(null);

    if (!emailOrUser.trim() || !password.trim()) {
      setErrorMessage('Por favor, preencha o e-mail/usuário e a senha.');
      return;
    }

    setIsLoading(true);

    // Simula autenticação com token de sessão
    setTimeout(() => {
      setIsLoading(false);
      // Tenta achar operador correspondente ou usa o Admin como padrão
      const matched = INITIAL_OPERATORS.find(
        (op) =>
          op.email.toLowerCase() === emailOrUser.trim().toLowerCase() ||
          op.username.toLowerCase() === emailOrUser.trim().toLowerCase()
      ) || INITIAL_OPERATORS[0];

      onLoginSuccess(matched);
    }, 700);
  };

  return (
    <div className="min-h-screen w-full bg-[#070b13] text-slate-100 flex flex-col justify-between relative overflow-hidden select-none font-sans">
      {/* Background Ambient Glows & Grid */}
      <div className="absolute inset-0 bg-[radial-gradient(#1e293b_1px,transparent_1px)] [background-size:24px_24px] opacity-25 pointer-events-none"></div>
      <div className="absolute top-[-15%] left-1/2 -translate-x-1/2 w-[650px] h-[450px] bg-sky-500/10 rounded-full blur-[140px] pointer-events-none"></div>
      <div className="absolute bottom-[-10%] right-[-5%] w-[450px] h-[350px] bg-indigo-500/10 rounded-full blur-[120px] pointer-events-none"></div>

      {/* Top Bar with Server Status */}
      <header className="w-full px-6 py-4 flex items-center justify-between z-10">
        <div className="flex items-center gap-2.5">
          <div className="w-7 h-7 rounded-lg bg-gradient-to-tr from-sky-500 via-blue-600 to-indigo-600 p-[1.5px] shadow-lg shadow-sky-500/20">
            <div className="w-full h-full bg-slate-950 rounded-[6px] flex items-center justify-center">
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none">
                <polygon points="12,2 22,8.5 22,15.5 12,22 2,15.5 2,8.5" stroke="#38bdf8" strokeWidth="2" fill="rgba(56, 189, 248, 0.2)" />
                <polyline points="12,2 12,22" stroke="#38bdf8" strokeWidth="1.5" opacity="0.8" />
              </svg>
            </div>
          </div>
          <span className="font-extrabold tracking-wider text-sm text-white">
            PRYSM<span className="text-sky-400 font-black">VIEW</span>
          </span>
        </div>
      </header>

      {/* Center Auth Card */}
      <main className="w-full max-w-md mx-auto px-4 z-10 my-auto py-6">
        <div className="bg-slate-900/85 backdrop-blur-xl border border-slate-700/70 rounded-2xl p-6 sm:p-8 shadow-2xl shadow-black/80 relative transition-all">
          {/* Subtle Top Border Glow */}
          <div className="absolute top-0 left-8 right-8 h-[1.5px] bg-gradient-to-r from-transparent via-sky-500 to-transparent opacity-75"></div>

          {/* Logo & Headline */}
          <div className="text-center mb-6">
            <div className="w-14 h-14 rounded-2xl bg-gradient-to-tr from-sky-500 via-blue-600 to-indigo-600 p-[2px] mx-auto mb-3 shadow-xl shadow-sky-500/25 flex items-center justify-center">
              <div className="w-full h-full bg-[#0a0f1d] rounded-[14px] flex items-center justify-center">
                <svg className="w-7 h-7" viewBox="0 0 24 24" fill="none">
                  <polygon points="12,2 22,8.5 22,15.5 12,22 2,15.5 2,8.5" stroke="#38bdf8" strokeWidth="2" fill="rgba(56, 189, 248, 0.25)" />
                  <polyline points="12,2 12,22" stroke="#38bdf8" strokeWidth="1.5" opacity="0.9" />
                  <polyline points="2,8.5 12,14 22,8.5" stroke="#38bdf8" strokeWidth="1.5" opacity="0.9" />
                </svg>
              </div>
            </div>

            <h1 className="text-xl font-extrabold text-white tracking-tight flex items-center justify-center gap-1.5">
              <span>PrysmView</span>
              <span className="text-xs px-2 py-0.5 rounded-full bg-sky-950 border border-sky-800 text-sky-400 font-mono font-bold">
                ADMIN
              </span>
            </h1>
            <p className="text-xs text-slate-400 mt-1">
              Console Corporativo de Acesso e Controle Remoto Desktop
            </p>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            {errorMessage && (
              <div className="p-2.5 rounded-xl bg-red-950/80 border border-red-800 text-red-200 text-xs flex items-center gap-2 animate-in fade-in">
                <span className="w-2 h-2 rounded-full bg-red-400 shrink-0"></span>
                <span>{errorMessage}</span>
              </div>
            )}

            <div>
              <label className="block text-xs font-semibold text-slate-300 mb-1.5">
                E-mail ou Usuário
              </label>
              <div className="relative">
                <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none text-slate-400">
                  <Mail className="w-4 h-4" />
                </div>
                <input
                  type="text"
                  required
                  placeholder="admin@prysmview.com ou usuario"
                  value={emailOrUser}
                  onChange={(e) => setEmailOrUser(e.target.value)}
                  className="w-full bg-[#0b101d] border border-slate-700/80 rounded-xl pl-10 pr-4 py-2.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 transition-all font-medium"
                />
              </div>
            </div>

            <div>
              <div className="flex items-center justify-between mb-1.5">
                <label className="text-xs font-semibold text-slate-300">
                  Senha de Acesso
                </label>
                <span className="text-[11px] text-sky-400 hover:text-sky-300 cursor-pointer">
                  Recuperar senha?
                </span>
              </div>
              <div className="relative">
                <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none text-slate-400">
                  <Lock className="w-4 h-4" />
                </div>
                <input
                  type={showPassword ? 'text' : 'password'}
                  required
                  placeholder="••••••••••••"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full bg-[#0b101d] border border-slate-700/80 rounded-xl pl-10 pr-10 py-2.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 transition-all font-mono"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute inset-y-0 right-0 pr-3.5 flex items-center text-slate-400 hover:text-slate-200 transition-colors"
                  tabIndex={-1}
                >
                  {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
            </div>

            <div className="flex items-center justify-between pt-1">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={rememberMe}
                  onChange={(e) => setRememberMe(e.target.checked)}
                  className="w-4 h-4 rounded bg-[#0b101d] border-slate-700 text-sky-600 focus:ring-0"
                />
                <span className="text-xs text-slate-300">Lembrar credenciais</span>
              </label>
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="w-full py-3 px-4 bg-gradient-to-r from-sky-600 via-blue-600 to-indigo-600 hover:from-sky-500 hover:to-indigo-500 disabled:opacity-50 text-white font-bold rounded-xl text-xs transition-all shadow-lg shadow-sky-600/30 flex items-center justify-center gap-2 group mt-2"
            >
              <span>{isLoading ? 'Autenticando na Nuvem...' : 'Entrar no Console'}</span>
              {!isLoading && <ArrowRight className="w-4 h-4 group-hover:translate-x-0.5 transition-transform" />}
            </button>
          </form>
        </div>
      </main>

      {/* Footer Security Badges */}
      <footer className="w-full px-6 py-4 flex flex-col sm:flex-row items-center justify-between text-[11px] text-slate-500 z-10 gap-2 border-t border-slate-800/60 bg-slate-950/40 backdrop-blur-sm">
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-1 text-slate-400">
            <Shield className="w-3.5 h-3.5 text-emerald-400" />
            <span>Criptografia E2E (DTLS 1.3 / DPAPI)</span>
          </span>
          <span className="text-slate-700">•</span>
          <span className="text-slate-400">Zero Logs de Tela ou Entrada</span>
        </div>

        <div className="font-mono text-[10px] text-slate-500">
          PrysmView v2.0.0 • Windows 10 & 11 Certified
        </div>
      </footer>
    </div>
  );
};
