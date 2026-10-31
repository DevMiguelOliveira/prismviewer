import React, { useState } from 'react';
import {
  Monitor,
  Zap,
  ShieldCheck,
  Server,
  Activity,
  HardDrive,
  Cpu,
  RefreshCw,
  Plus,
  KeyRound,
  ChevronRight,
  ChevronDown,
  ChevronUp,
  LayoutGrid,
  List,
  CheckCircle2,
  XCircle,
  ExternalLink,
  Lock,
  ArrowUpRight,
  Sparkles,
} from 'lucide-react';
import { Device, Store } from '../types';

interface DashboardProps {
  devices: Device[];
  stores: Store[];
  onConnectDevice: (device: Device, method: 'code' | 'unattended', codeOrPassword?: string) => void;
  onSendNotification: (msg: string, type?: 'success' | 'warning' | 'error') => void;
  onAddDevice: (newDev: Device) => void;
}

export const Dashboard: React.FC<DashboardProps> = ({
  devices,
  stores,
  onConnectDevice,
  onSendNotification,
  onAddDevice,
}) => {
  // Accordion Hero State
  const [accordionOpen, setAccordionOpen] = useState(false);
  const [quickTargetId, setQuickTargetId] = useState('');
  const [quickCode, setQuickCode] = useState('');
  const [quickPassword, setQuickPassword] = useState('');
  const [quickMethod, setQuickMethod] = useState<'code' | 'unattended'>('code');

  // Filters & View
  const [statusFilter, setStatusFilter] = useState<'all' | 'online' | 'offline'>('all');
  const [selectedStore, setSelectedStore] = useState<string>('all');
  const [viewMode, setViewMode] = useState<'cards' | 'table'>('cards');

  // Selected device for side drawer
  const [drawerDevice, setDrawerDevice] = useState<Device | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);

  // New device form state
  const [newDevId, setNewDevId] = useState('');
  const [newDevName, setNewDevName] = useState('');
  const [newDevStore, setNewDevStore] = useState('Loja Centro');
  const [newDevIp, setNewDevIp] = useState('192.168.1.');

  // Filtered devices
  const filteredDevices = devices.filter((dev) => {
    if (statusFilter === 'online' && dev.status !== 'online') return false;
    if (statusFilter === 'offline' && dev.status !== 'offline') return false;
    if (selectedStore !== 'all' && dev.store !== selectedStore) return false;
    return true;
  });

  const onlineCount = devices.filter((d) => d.status === 'online').length;
  const offlineCount = devices.length - onlineCount;

  const handleQuickConnect = (e: React.FormEvent) => {
    e.preventDefault();
    if (!quickTargetId.trim()) {
      onSendNotification('Informe a ID ou IP do dispositivo alvo.', 'warning');
      return;
    }
    const matched = devices.find(
      (d) => d.id.toLowerCase() === quickTargetId.trim().toLowerCase() || d.ip === quickTargetId.trim()
    );
    if (!matched) {
      onSendNotification(`Dispositivo '${quickTargetId}' não encontrado no catálogo.`, 'error');
      return;
    }
    if (quickMethod === 'code' && (!quickCode || quickCode.length !== 8)) {
      onSendNotification('Informe o código temporário de 8 dígitos gerado no PDV.', 'warning');
      return;
    }
    onConnectDevice(matched, quickMethod, quickMethod === 'code' ? quickCode : quickPassword);
  };

  const handleCreateDevice = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newDevId.trim() || !newDevName.trim()) {
      onSendNotification('Preencha os campos obrigatórios da ID e Nome.', 'warning');
      return;
    }
    const created: Device = {
      id: newDevId.trim().toUpperCase(),
      name: newDevName.trim(),
      store: newDevStore,
      status: 'online',
      lastSeen: 'Agora mesmo',
      os: 'Windows 11 Pro 23H2',
      version: '2.0.0',
      ip: newDevIp.trim(),
      cpu: 'Intel Core i5-10400 @ 2.90GHz (6C/12T)',
      memory: '16.0 GB DDR4 (42% em uso)',
      disk: 'SSD 240GB (95GB Livres)',
      unattendedPassword: 'admin@prysm2024',
      resolution: '1920x1080 @ 60Hz',
      monitors: 1,
      uptime: '1 dia, 4 horas',
    };
    onAddDevice(created);
    setShowAddModal(false);
    setNewDevId('');
    setNewDevName('');
    onSendNotification(`Dispositivo ${created.id} cadastrado com sucesso!`, 'success');
  };

  return (
    <div className="space-y-5 animate-in fade-in duration-200">
      {/* =========================================================================
          HERO: TEAMVIEWER QUICK SUPPORT ACCORDION
          ========================================================================= */}
      <div className="bg-slate-900/90 border border-slate-800 rounded-xl overflow-hidden shadow-lg">
        <div
          onClick={() => setAccordionOpen(!accordionOpen)}
          className="px-4 py-3 bg-gradient-to-r from-slate-900 via-slate-850 to-slate-900 flex items-center justify-between cursor-pointer hover:bg-slate-800/60 transition-colors"
        >
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-sky-950 border border-sky-800/80 text-sky-400 flex items-center justify-center">
              <Zap className="w-4 h-4 fill-sky-400/20" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-sm text-slate-100">Suporte & Conexão Direta</span>
                <span className="px-2 py-0.5 rounded-full bg-slate-800 text-[11px] font-mono text-sky-400 border border-slate-700">
                  Sua ID: 127.0.0.1
                </span>
                <span className="px-2 py-0.5 rounded-full bg-emerald-950/80 text-[10px] text-emerald-400 border border-emerald-800/80 flex items-center gap-1">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                  Porta 19533 Ativa
                </span>
              </div>
              <p className="text-xs text-slate-400">
                Acesso imediato com código de 8 dígitos ou senha desassistida
              </p>
            </div>
          </div>

          <button
            type="button"
            className="flex items-center gap-1 text-xs text-sky-400 hover:text-sky-300 font-semibold px-2.5 py-1 rounded bg-slate-800/80 border border-slate-700"
          >
            <span>{accordionOpen ? 'Recolher' : 'Expandir Conexão'}</span>
            {accordionOpen ? <ChevronUp className="w-3.5 h-3.5" /> : <ChevronDown className="w-3.5 h-3.5" />}
          </button>
        </div>

        {accordionOpen && (
          <form
            onSubmit={handleQuickConnect}
            className="p-4 border-t border-slate-800 bg-slate-950/60 flex flex-col md:flex-row items-center gap-3 animate-in slide-in-from-top-2"
          >
            <div className="flex-1 w-full grid grid-cols-1 sm:grid-cols-3 gap-3">
              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">
                  ID do Dispositivo ou IP:
                </label>
                <input
                  type="text"
                  placeholder="Ex: PDV-CAIXA-01 ou 192.168.1.101"
                  value={quickTargetId}
                  onChange={(e) => setQuickTargetId(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <label className="text-[11px] font-medium text-slate-400">Método de Autenticação:</label>
                  <div className="flex items-center gap-2 text-[10px]">
                    <button
                      type="button"
                      onClick={() => setQuickMethod('code')}
                      className={`font-semibold ${quickMethod === 'code' ? 'text-sky-400 underline' : 'text-slate-500'}`}
                    >
                      Código 8d
                    </button>
                    <button
                      type="button"
                      onClick={() => setQuickMethod('unattended')}
                      className={`font-semibold ${quickMethod === 'unattended' ? 'text-sky-400 underline' : 'text-slate-500'}`}
                    >
                      Senha Fixa
                    </button>
                  </div>
                </div>

                {quickMethod === 'code' ? (
                  <input
                    type="text"
                    maxLength={8}
                    placeholder="Código de 8 dígitos (ex: 84920153)"
                    value={quickCode}
                    onChange={(e) => setQuickCode(e.target.value.replace(/\D/g, ''))}
                    className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs font-mono text-emerald-400 tracking-wider placeholder-slate-500 focus:outline-none focus:border-sky-500"
                  />
                ) : (
                  <input
                    type="password"
                    placeholder="Senha de acesso desassistido..."
                    value={quickPassword}
                    onChange={(e) => setQuickPassword(e.target.value)}
                    className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-sky-500"
                  />
                )}
              </div>

              <div className="flex items-end">
                <button
                  type="submit"
                  className="w-full h-8 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-bold transition-all shadow-md shadow-sky-600/30 flex items-center justify-center gap-1.5"
                >
                  <Zap className="w-3.5 h-3.5" />
                  <span>Conectar Agora</span>
                </button>
              </div>
            </div>
          </form>
        )}
      </div>

      {/* =========================================================================
          METRICS KPIS CARDS
          ========================================================================= */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <div className="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800/80">
          <div className="flex items-center justify-between text-slate-400 mb-1">
            <span className="text-xs font-medium">Total de PDVs</span>
            <Monitor className="w-4 h-4 text-sky-400" />
          </div>
          <div className="text-2xl font-black text-white">{devices.length}</div>
          <div className="text-[10px] text-slate-400 mt-0.5">Cadastrados no tenant</div>
        </div>

        <div className="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800/80">
          <div className="flex items-center justify-between text-slate-400 mb-1">
            <span className="text-xs font-medium">PDVs Online</span>
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
          </div>
          <div className="text-2xl font-black text-emerald-400">{onlineCount}</div>
          <div className="text-[10px] text-emerald-400/80 mt-0.5">Prontos para conexão</div>
        </div>

        <div className="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800/80">
          <div className="flex items-center justify-between text-slate-400 mb-1">
            <span className="text-xs font-medium">PDVs Offline</span>
            <XCircle className="w-4 h-4 text-slate-500" />
          </div>
          <div className="text-2xl font-black text-slate-400">{offlineCount}</div>
          <div className="text-[10px] text-slate-500 mt-0.5">Caixas fechados</div>
        </div>

        <div className="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800/80">
          <div className="flex items-center justify-between text-slate-400 mb-1">
            <span className="text-xs font-medium">Latência Média</span>
            <Activity className="w-4 h-4 text-sky-400" />
          </div>
          <div className="text-2xl font-black text-sky-400 font-mono">18 ms</div>
          <div className="text-[10px] text-sky-400/80 mt-0.5">Rede interna de lojas</div>
        </div>
      </div>

      {/* =========================================================================
          TOOLBAR: FILTERS, STORE SELECTOR, VIEW MODE & NEW DEVICE
          ========================================================================= */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 pt-1">
        <div className="flex items-center gap-2 flex-wrap">
          {/* Status Pills */}
          <div className="flex items-center bg-slate-900 p-0.5 rounded-lg border border-slate-800 text-xs">
            <button
              onClick={() => setStatusFilter('all')}
              className={`px-3 py-1 rounded-md font-medium transition-colors ${
                statusFilter === 'all' ? 'bg-sky-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              Todos ({devices.length})
            </button>
            <button
              onClick={() => setStatusFilter('online')}
              className={`px-3 py-1 rounded-md font-medium transition-colors ${
                statusFilter === 'online' ? 'bg-emerald-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              Online ({onlineCount})
            </button>
            <button
              onClick={() => setStatusFilter('offline')}
              className={`px-3 py-1 rounded-md font-medium transition-colors ${
                statusFilter === 'offline' ? 'bg-slate-700 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              Offline ({offlineCount})
            </button>
          </div>

          {/* Store Selector */}
          <select
            value={selectedStore}
            onChange={(e) => setSelectedStore(e.target.value)}
            className="bg-slate-900 border border-slate-800 rounded-lg px-3 py-1 text-xs text-slate-300 focus:outline-none focus:border-sky-500"
          >
            <option value="all">Todas as Lojas</option>
            {stores.map((s) => (
              <option key={s.id} value={s.name}>
                {s.name} ({s.code})
              </option>
            ))}
          </select>
        </div>

        <div className="flex items-center gap-2">
          {/* View mode toggle */}
          <div className="flex items-center bg-slate-900 p-0.5 rounded-lg border border-slate-800">
            <button
              onClick={() => setViewMode('cards')}
              className={`p-1 rounded ${viewMode === 'cards' ? 'bg-slate-800 text-sky-400' : 'text-slate-400'}`}
              title="Visualização em Cards"
            >
              <LayoutGrid className="w-3.5 h-3.5" />
            </button>
            <button
              onClick={() => setViewMode('table')}
              className={`p-1 rounded ${viewMode === 'table' ? 'bg-slate-800 text-sky-400' : 'text-slate-400'}`}
              title="Visualização em Lista/Tabela"
            >
              <List className="w-3.5 h-3.5" />
            </button>
          </div>

          {/* New Device Button */}
          <button
            onClick={() => setShowAddModal(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm shadow-sky-600/30"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Cadastrar PDV</span>
          </button>
        </div>
      </div>

      {/* =========================================================================
          DEVICES GRID OR TABLE VIEW
          ========================================================================= */}
      {viewMode === 'cards' ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3.5">
          {filteredDevices.map((dev) => (
            <div
              key={dev.id}
              className="bg-slate-900/90 border border-slate-800/90 hover:border-slate-700 rounded-xl p-4 flex flex-col justify-between transition-all hover:shadow-xl hover:shadow-slate-950/60 group"
            >
              <div>
                {/* Header card */}
                <div className="flex items-start justify-between gap-2 mb-2">
                  <div>
                    <div className="flex items-center gap-2">
                      <span
                        className={`w-2.5 h-2.5 rounded-full ${
                          dev.status === 'online' ? 'pulse-dot-green' : 'bg-slate-600'
                        }`}
                      ></span>
                      <h4 className="font-bold text-sm text-slate-100 group-hover:text-sky-300 transition-colors font-mono">
                        {dev.id}
                      </h4>
                    </div>
                    <p className="text-xs text-slate-400 mt-0.5">{dev.name}</p>
                  </div>
                  <span className="px-2 py-0.5 rounded text-[10px] font-semibold bg-slate-800 border border-slate-700 text-slate-300">
                    {dev.store}
                  </span>
                </div>

                {/* Specs */}
                <div className="space-y-1.5 my-3 text-[11px] text-slate-400 bg-slate-950/60 p-2.5 rounded-lg border border-slate-800/60">
                  <div className="flex justify-between">
                    <span>IP Local:</span>
                    <span className="font-mono text-slate-300">{dev.ip}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Sistema Operacional:</span>
                    <span className="text-slate-300 truncate max-w-[170px]" title={dev.os}>
                      {dev.os}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>CPU / RAM:</span>
                    <span className="text-slate-300 truncate max-w-[170px]">
                      {dev.cpu.split('@')[0]}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>Último Contato:</span>
                    <span className="text-emerald-400/90">{dev.lastSeen}</span>
                  </div>
                </div>
              </div>

              {/* Action Buttons */}
              <div className="flex items-center gap-2 pt-2 border-t border-slate-800/80">
                <button
                  onClick={() => onConnectDevice(dev, 'code')}
                  disabled={dev.status === 'offline'}
                  className="flex-1 py-1.5 bg-sky-600 hover:bg-sky-500 disabled:bg-slate-800 disabled:text-slate-600 text-white rounded-lg text-xs font-semibold transition-colors flex items-center justify-center gap-1.5 shadow-sm shadow-sky-600/30"
                >
                  <Zap className="w-3.5 h-3.5" />
                  <span>Conectar</span>
                </button>

                {dev.unattendedPassword && (
                  <button
                    onClick={() => onConnectDevice(dev, 'unattended', dev.unattendedPassword)}
                    disabled={dev.status === 'offline'}
                    className="p-1.5 bg-slate-800 hover:bg-slate-700 disabled:opacity-40 text-slate-300 hover:text-white rounded-lg border border-slate-700 transition-colors"
                    title="Acesso Desassistido com Senha Fixa"
                  >
                    <Lock className="w-3.5 h-3.5 text-amber-400" />
                  </button>
                )}

                <button
                  onClick={() => setDrawerDevice(dev)}
                  className="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white rounded-lg border border-slate-700 transition-colors"
                  title="Ver especificações e diagnósticos do PDV"
                >
                  <ChevronRight className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : (
        /* Table View */
        <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-lg">
          <table className="w-full text-left text-xs text-slate-300">
            <thead className="bg-slate-950/80 text-[11px] text-slate-400 uppercase tracking-wider border-b border-slate-800">
              <tr>
                <th className="py-2.5 px-4">Status</th>
                <th className="py-2.5 px-4">ID Dispositivo</th>
                <th className="py-2.5 px-4">Nome & Loja</th>
                <th className="py-2.5 px-4">IP & SO</th>
                <th className="py-2.5 px-4">Hardware</th>
                <th className="py-2.5 px-4 text-right">Ações</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800">
              {filteredDevices.map((dev) => (
                <tr key={dev.id} className="hover:bg-slate-800/40 transition-colors">
                  <td className="py-3 px-4">
                    <span
                      className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-semibold ${
                        dev.status === 'online'
                          ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                          : 'bg-slate-800 text-slate-400 border border-slate-700'
                      }`}
                    >
                      <span
                        className={`w-1.5 h-1.5 rounded-full ${dev.status === 'online' ? 'bg-emerald-400' : 'bg-slate-500'}`}
                      ></span>
                      {dev.status === 'online' ? 'Online' : 'Offline'}
                    </span>
                  </td>
                  <td className="py-3 px-4 font-mono font-bold text-slate-100">{dev.id}</td>
                  <td className="py-3 px-4">
                    <div className="font-medium text-slate-200">{dev.name}</div>
                    <div className="text-[10px] text-slate-400">{dev.store}</div>
                  </td>
                  <td className="py-3 px-4">
                    <div className="font-mono text-slate-300">{dev.ip}</div>
                    <div className="text-[10px] text-slate-400">{dev.os}</div>
                  </td>
                  <td className="py-3 px-4 text-[11px] text-slate-400">
                    <div>{dev.cpu.split('@')[0]}</div>
                    <div>{dev.memory.split('(')[0]}</div>
                  </td>
                  <td className="py-3 px-4 text-right">
                    <div className="flex items-center justify-end gap-1.5">
                      <button
                        onClick={() => onConnectDevice(dev, 'code')}
                        disabled={dev.status === 'offline'}
                        className="px-2.5 py-1 bg-sky-600 hover:bg-sky-500 disabled:opacity-40 text-white rounded text-[11px] font-semibold transition-colors flex items-center gap-1"
                      >
                        <Zap className="w-3 h-3" />
                        <span>Conectar</span>
                      </button>
                      <button
                        onClick={() => setDrawerDevice(dev)}
                        className="p-1 text-slate-400 hover:text-white"
                        title="Detalhes"
                      >
                        <ChevronRight className="w-4 h-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* =========================================================================
          DEVICE DETAIL DRAWER (SLIDE-OVER)
          ========================================================================= */}
      {drawerDevice && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex justify-end z-50 animate-in fade-in">
          <div className="w-full max-w-md bg-slate-900 border-l border-slate-800 h-full p-5 overflow-y-auto flex flex-col justify-between text-xs text-slate-300 animate-in slide-in-from-right duration-200">
            <div>
              <div className="flex items-center justify-between pb-3 border-b border-slate-800 mb-4">
                <div className="flex items-center gap-2">
                  <Monitor className="w-4 h-4 text-sky-400" />
                  <span className="font-bold text-sm text-white font-mono">{drawerDevice.id}</span>
                </div>
                <button
                  onClick={() => setDrawerDevice(null)}
                  className="p-1 hover:bg-slate-800 rounded text-slate-400 hover:text-white"
                >
                  ✕
                </button>
              </div>

              {/* Status Banner */}
              <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 mb-4 flex items-center justify-between">
                <div>
                  <div className="font-bold text-slate-200">{drawerDevice.name}</div>
                  <div className="text-[11px] text-slate-400">{drawerDevice.store}</div>
                </div>
                <span
                  className={`px-2 py-0.5 rounded text-[10px] font-semibold ${
                    drawerDevice.status === 'online'
                      ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                      : 'bg-slate-800 text-slate-400'
                  }`}
                >
                  {drawerDevice.status === 'online' ? '● Online' : 'Offline'}
                </span>
              </div>

              {/* Hardware & Diagnostics */}
              <div className="space-y-3">
                <div>
                  <h5 className="font-bold text-slate-200 text-xs mb-1.5 flex items-center gap-1.5">
                    <Server className="w-3.5 h-3.5 text-sky-400" />
                    <span>Informações do Sistema</span>
                  </h5>
                  <div className="bg-slate-950 p-2.5 rounded-lg border border-slate-800 space-y-1.5 text-[11px]">
                    <div className="flex justify-between">
                      <span className="text-slate-400">Sistema:</span>
                      <span className="text-slate-200 text-right">{drawerDevice.os}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">IP na Rede:</span>
                      <span className="font-mono text-slate-200">{drawerDevice.ip}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">Versão PrysmHost:</span>
                      <span className="font-mono text-sky-400">v{drawerDevice.version}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">Uptime:</span>
                      <span className="text-emerald-400">{drawerDevice.uptime}</span>
                    </div>
                  </div>
                </div>

                <div>
                  <h5 className="font-bold text-slate-200 text-xs mb-1.5 flex items-center gap-1.5">
                    <Cpu className="w-3.5 h-3.5 text-emerald-400" />
                    <span>Hardware do PDV</span>
                  </h5>
                  <div className="bg-slate-950 p-2.5 rounded-lg border border-slate-800 space-y-1.5 text-[11px]">
                    <div className="flex justify-between">
                      <span className="text-slate-400">Processador:</span>
                      <span className="text-slate-200 text-right">{drawerDevice.cpu}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">Memória RAM:</span>
                      <span className="text-slate-200">{drawerDevice.memory}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">Armazenamento:</span>
                      <span className="text-slate-200">{drawerDevice.disk}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-400">Monitores / Resolução:</span>
                      <span className="text-slate-200">{drawerDevice.resolution}</span>
                    </div>
                  </div>
                </div>

                {drawerDevice.unattendedPassword && (
                  <div>
                    <h5 className="font-bold text-slate-200 text-xs mb-1.5 flex items-center gap-1.5">
                      <Lock className="w-3.5 h-3.5 text-amber-400" />
                      <span>Acesso Desassistido</span>
                    </h5>
                    <div className="bg-slate-950 p-2.5 rounded-lg border border-slate-800 text-[11px] flex items-center justify-between">
                      <span className="text-slate-400">Senha Fixa Configurada:</span>
                      <span className="font-mono text-amber-400">••••••••</span>
                    </div>
                  </div>
                )}
              </div>
            </div>

            <div className="pt-4 border-t border-slate-800 flex items-center gap-2">
              <button
                onClick={() => {
                  setDrawerDevice(null);
                  onConnectDevice(drawerDevice, 'code');
                }}
                disabled={drawerDevice.status === 'offline'}
                className="flex-1 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-40 text-white rounded-lg font-bold text-xs transition-colors flex items-center justify-center gap-1.5"
              >
                <Zap className="w-3.5 h-3.5" />
                <span>Iniciar Sessão Remota</span>
              </button>
              <button
                onClick={() => setDrawerDevice(null)}
                className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg text-xs"
              >
                Fechar
              </button>
            </div>
          </div>
        </div>
      )}

      {/* =========================================================================
          MODAL: CADASTRO DE NOVO PDV
          ========================================================================= */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-md shadow-2xl p-5 text-xs text-slate-200">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800 mb-4">
              <div className="flex items-center gap-2">
                <Plus className="w-4 h-4 text-sky-400" />
                <h3 className="font-bold text-sm text-white">Cadastrar Novo PDV / Caixa</h3>
              </div>
              <button onClick={() => setShowAddModal(false)} className="text-slate-400 hover:text-white">✕</button>
            </div>

            <form onSubmit={handleCreateDevice} className="space-y-3">
              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">
                  ID do Dispositivo (Único):
                </label>
                <input
                  type="text"
                  required
                  placeholder="Ex: PDV-CAIXA-05"
                  value={newDevId}
                  onChange={(e) => setNewDevId(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-white uppercase font-mono focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">
                  Nome Amigável / Descrição:
                </label>
                <input
                  type="text"
                  required
                  placeholder="Ex: Caixa 05 - Saída Estacionamento"
                  value={newDevName}
                  onChange={(e) => setNewDevName(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-white focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">
                  Loja Pertencente:
                </label>
                <select
                  value={newDevStore}
                  onChange={(e) => setNewDevStore(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-white focus:outline-none focus:border-sky-500"
                >
                  {stores.map((s) => (
                    <option key={s.id} value={s.name}>
                      {s.name} ({s.code})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">
                  Endereço IP Local na Loja:
                </label>
                <input
                  type="text"
                  required
                  placeholder="Ex: 192.168.1.105"
                  value={newDevIp}
                  onChange={(e) => setNewDevIp(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-white font-mono focus:outline-none focus:border-sky-500"
                />
              </div>

              <div className="pt-3 border-t border-slate-800 flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setShowAddModal(false)}
                  className="flex-1 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg"
                >
                  Cancelar
                </button>
                <button
                  type="submit"
                  className="flex-1 py-2 bg-sky-600 hover:bg-sky-500 text-white font-bold rounded-lg shadow-md"
                >
                  Salvar Dispositivo
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
