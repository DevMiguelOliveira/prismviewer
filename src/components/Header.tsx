import React, { useState } from 'react';
import {
  Search,
  Bell,
  Sun,
  Moon,
  RefreshCw,
  Monitor,
  ShieldCheck,
  UserCheck,
  ChevronDown,
  Terminal,
  LogOut,
  Sliders,
  CheckCircle2,
  Activity,
} from 'lucide-react';
import { Operator } from '../types';

interface HeaderProps {
  currentTab: string;
  activeOperator: Operator;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onRefresh: () => void;
  theme: 'dark' | 'light';
  onToggleTheme: () => void;
  appMode: 'admin' | 'host';
  onToggleAppMode: () => void;
  onSelectTab: (tab: string) => void;
  onOpenDiagnostics: () => void;
  onLogout?: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  currentTab,
  activeOperator,
  searchQuery,
  onSearchChange,
  onRefresh,
  theme,
  onToggleTheme,
  appMode,
  onToggleAppMode,
  onSelectTab,
  onOpenDiagnostics,
  onLogout,
}) => {
  const [showProfileMenu, setShowProfileMenu] = useState(false);
  const [showNotifications, setShowNotifications] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleRefreshClick = () => {
    setIsRefreshing(true);
    onRefresh();
    setTimeout(() => setIsRefreshing(false), 600);
  };

  const getTabLabel = (id: string) => {
    switch (id) {
      case 'dashboard': return 'Painel de Dispositivos';
      case 'direct': return 'Conexão Rápida';
      case 'stores': return 'Grupos & Lojas';
      case 'audit': return 'Auditoria E2E';
      case 'technicians': return 'Técnicos & Operadores';
      case 'settings': return 'Configurações Globais';
      default: return 'Visão Geral';
    }
  };

  return (
    <header className="h-14 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 sticky top-0 z-40 select-none shadow-md">
      {/* Brand & Breadcrumb */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-sky-600 via-blue-600 to-indigo-600 p-[1.5px] flex items-center justify-center shadow-lg shadow-sky-500/20">
            <div className="w-full h-full bg-slate-950 rounded-[7px] flex items-center justify-center">
              <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none">
                <polygon points="12,2 22,8.5 22,15.5 12,22 2,15.5 2,8.5" stroke="#38bdf8" strokeWidth="2" fill="rgba(56, 189, 248, 0.2)" />
                <polyline points="12,2 12,22" stroke="#38bdf8" strokeWidth="1.5" opacity="0.8" />
                <polyline points="2,8.5 12,14 22,8.5" stroke="#38bdf8" strokeWidth="1.5" opacity="0.8" />
              </svg>
            </div>
          </div>

          <div className="flex items-baseline gap-1.5">
            <span className="font-extrabold tracking-wider text-base text-white">
              PRYSM<span className="text-sky-400 font-black">VIEW</span>
            </span>
            <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-sky-950/80 border border-sky-800/60 text-sky-400">
              v2.0.0
            </span>
          </div>
        </div>

        <span className="text-slate-600 text-sm hidden sm:inline">/</span>
        <span className="text-xs text-slate-300 font-medium hidden sm:inline-flex items-center gap-1.5">
          {appMode === 'admin' ? (
            <>
              <Monitor className="w-3.5 h-3.5 text-sky-400" />
              <span>{getTabLabel(currentTab)}</span>
            </>
          ) : (
            <>
              <Terminal className="w-3.5 h-3.5 text-emerald-400" />
              <span className="text-emerald-400 font-semibold">Simulador do PDV (Host Caixa)</span>
            </>
          )}
        </span>
      </div>

      {/* Center Search (Admin mode) */}
      {appMode === 'admin' && (
        <div className="flex-1 max-w-md mx-4 hidden md:block">
          <div className="relative">
            <Search className="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder="Buscar por caixa, loja, IP ou SO (ex: CAIXA-01, 192.168...)"
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              className="w-full bg-slate-950/80 border border-slate-800 rounded-lg pl-9 pr-4 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-sky-500 transition-colors"
            />
            {searchQuery && (
              <button
                onClick={() => onSearchChange('')}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 text-xs px-1"
              >
                ✕
              </button>
            )}
          </div>
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-2">
        {/* Switch App Mode: Operator Console vs Host PDV */}
        <button
          onClick={onToggleAppMode}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border transition-all ${
            appMode === 'admin'
              ? 'bg-slate-800/80 hover:bg-slate-800 text-slate-200 border-slate-700 hover:border-sky-500/50'
              : 'bg-emerald-950/80 hover:bg-emerald-900/80 text-emerald-300 border-emerald-700/80'
          }`}
          title="Alternar entre Painel do Operador (Admin) e Terminal do Caixa (Host)"
        >
          {appMode === 'admin' ? (
            <>
              <Terminal className="w-3.5 h-3.5 text-emerald-400" />
              <span className="hidden lg:inline">Testar Visão do</span>
              <span className="text-emerald-400">PDV Host</span>
            </>
          ) : (
            <>
              <Monitor className="w-3.5 h-3.5 text-sky-400" />
              <span className="hidden lg:inline">Voltar ao</span>
              <span className="text-sky-400">Painel Operador</span>
            </>
          )}
        </button>

        {/* Live Network Health Chip / Diagnóstico */}
        <button
          onClick={onOpenDiagnostics}
          className="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-slate-950/80 hover:bg-slate-800/80 border border-slate-800 hover:border-sky-500/50 text-[11px] text-slate-300 transition-colors cursor-pointer"
          title="Clique para abrir Diagnóstico de Conexão, WebRTC e WebView"
        >
          <span className="pulse-dot-green"></span>
          <span className="font-mono text-emerald-400 font-semibold">18 ms</span>
          <span className="text-slate-500">•</span>
          <span className="text-slate-400">DTLS 1.3</span>
          <Activity className="w-3 h-3 text-sky-400 ml-0.5" />
        </button>

        {/* Refresh Devices */}
        {appMode === 'admin' && (
          <button
            onClick={handleRefreshClick}
            className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition-colors"
            title="Sincronizar Dispositivos"
          >
            <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin text-sky-400' : ''}`} />
          </button>
        )}

        {/* Theme Toggle */}
        <button
          onClick={onToggleTheme}
          className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition-colors"
          title={`Alternar para tema ${theme === 'dark' ? 'claro' : 'escuro'}`}
        >
          {theme === 'dark' ? <Sun className="w-4 h-4 text-amber-400" /> : <Moon className="w-4 h-4 text-sky-400" />}
        </button>

        {/* Notifications */}
        <div className="relative">
          <button
            onClick={() => setShowNotifications(!showNotifications)}
            className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition-colors relative"
            title="Notificações do Sistema"
          >
            <Bell className="w-4 h-4" />
            <span className="absolute top-1.5 right-1.5 w-2 h-2 bg-sky-500 rounded-full"></span>
          </button>

          {showNotifications && (
            <div className="absolute right-0 mt-2 w-80 bg-slate-900 border border-slate-700/80 rounded-xl shadow-2xl p-3 z-50 text-xs animate-in fade-in slide-in-from-top-2">
              <div className="flex items-center justify-between pb-2 mb-2 border-b border-slate-800">
                <span className="font-semibold text-slate-200">Notificações</span>
                <span className="text-[10px] text-sky-400 font-mono">2 não lidas</span>
              </div>
              <div className="space-y-2">
                <div className="p-2 rounded bg-slate-800/60 border border-slate-700/50 flex items-start gap-2">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                  <div>
                    <p className="font-medium text-slate-200">Sessão Segura Finalizada</p>
                    <p className="text-[10px] text-slate-400">Caixa 01 (Loja Centro) - Auditoria gravada com SHA-256.</p>
                  </div>
                </div>
                <div className="p-2 rounded bg-slate-800/60 border border-slate-700/50 flex items-start gap-2">
                  <ShieldCheck className="w-4 h-4 text-sky-400 shrink-0 mt-0.5" />
                  <div>
                    <p className="font-medium text-slate-200">Handshake P2P Concluído</p>
                    <p className="text-[10px] text-slate-400">Canal de controle WebRTC estabelecido sem relay TURN.</p>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Operator Profile Menu */}
        <div className="relative">
          <button
            onClick={() => setShowProfileMenu(!showProfileMenu)}
            className="flex items-center gap-2 pl-2 pr-2.5 py-1 rounded-lg hover:bg-slate-800 transition-colors border border-transparent hover:border-slate-700"
          >
            <div className="w-6 h-6 rounded-full bg-gradient-to-tr from-sky-500 to-indigo-600 flex items-center justify-center text-white text-[11px] font-bold">
              {activeOperator.name.charAt(0)}
            </div>
            <div className="text-left hidden md:block">
              <div className="text-xs font-semibold text-slate-200 leading-none">{activeOperator.name}</div>
              <div className="text-[10px] text-sky-400 font-medium leading-tight">{activeOperator.role}</div>
            </div>
            <ChevronDown className="w-3.5 h-3.5 text-slate-400" />
          </button>

          {showProfileMenu && (
            <div className="absolute right-0 mt-2 w-64 bg-slate-900 border border-slate-700 rounded-xl shadow-2xl p-3 z-50 text-xs animate-in fade-in slide-in-from-top-2">
              <div className="flex items-center gap-2.5 pb-2.5 mb-2 border-b border-slate-800">
                <div className="w-9 h-9 rounded-full bg-gradient-to-tr from-sky-500 to-indigo-600 flex items-center justify-center text-white text-sm font-bold">
                  {activeOperator.name.charAt(0)}
                </div>
                <div>
                  <div className="font-bold text-slate-200">{activeOperator.name}</div>
                  <div className="text-slate-400 text-[11px]">{activeOperator.email}</div>
                  <div className="text-emerald-400 text-[10px] flex items-center gap-1 font-medium mt-0.5">
                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                    Sessão Ativa (Operador Master)
                  </div>
                </div>
              </div>

              <div className="space-y-1">
                <button
                  onClick={() => {
                    onSelectTab('settings');
                    setShowProfileMenu(false);
                  }}
                  className="w-full flex items-center gap-2 px-2.5 py-1.5 text-slate-300 hover:text-white hover:bg-slate-800 rounded-md transition-colors"
                >
                  <Sliders className="w-3.5 h-3.5 text-slate-400" />
                  <span>Configurações do Sistema</span>
                </button>
                <button
                  onClick={() => {
                    onSelectTab('technicians');
                    setShowProfileMenu(false);
                  }}
                  className="w-full flex items-center gap-2 px-2.5 py-1.5 text-slate-300 hover:text-white hover:bg-slate-800 rounded-md transition-colors"
                >
                  <UserCheck className="w-3.5 h-3.5 text-slate-400" />
                  <span>Gerenciar Técnicos & Permissões</span>
                </button>
                {onLogout && (
                  <button
                    onClick={() => {
                      setShowProfileMenu(false);
                      onLogout();
                    }}
                    className="w-full flex items-center gap-2 px-2.5 py-1.5 text-red-400 hover:text-red-300 hover:bg-red-950/40 rounded-md transition-colors mt-1"
                  >
                    <LogOut className="w-3.5 h-3.5" />
                    <span>Sair / Trocar Usuário</span>
                  </button>
                )}
              </div>

              <div className="mt-2 pt-2 border-t border-slate-800">
                <div className="px-2 py-1 text-[10px] text-slate-500 flex items-center justify-between">
                  <span>Host ID Local:</span>
                  <span className="font-mono text-slate-400">127.0.0.1:19533</span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </header>
  );
};
