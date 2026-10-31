import React, { useState } from 'react';
import { Store, Device } from '../types';
import {
  Users,
  MapPin,
  Phone,
  Monitor,
  Plus,
  Edit,
  Trash2,
  CheckCircle2,
  XCircle,
  Building,
} from 'lucide-react';

interface StoresViewProps {
  stores: Store[];
  devices: Device[];
  onAddStore: (store: Store) => void;
  onSendNotification: (msg: string, type?: 'success' | 'warning' | 'error') => void;
}

export const StoresView: React.FC<StoresViewProps> = ({
  stores,
  devices,
  onAddStore,
  onSendNotification,
}) => {
  const [showAddModal, setShowAddModal] = useState(false);
  const [newStoreName, setNewStoreName] = useState('');
  const [newStoreCode, setNewStoreCode] = useState('');
  const [newStoreLocation, setNewStoreLocation] = useState('');
  const [newStoreManager, setNewStoreManager] = useState('');
  const [newStorePhone, setNewStorePhone] = useState('');

  const handleCreateStore = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newStoreName.trim() || !newStoreCode.trim()) {
      onSendNotification('Preencha os campos obrigatórios da loja.', 'warning');
      return;
    }
    const created: Store = {
      id: `store-${Date.now()}`,
      name: newStoreName.trim(),
      code: newStoreCode.trim().toUpperCase(),
      location: newStoreLocation.trim() || 'São Paulo, SP',
      manager: newStoreManager.trim() || 'Gerente Geral',
      phone: newStorePhone.trim() || '(11) 3000-0000',
      deviceCount: 0,
      onlineCount: 0,
    };
    onAddStore(created);
    setShowAddModal(false);
    setNewStoreName('');
    setNewStoreCode('');
    onSendNotification(`Loja ${created.name} (${created.code}) cadastrada com sucesso!`, 'success');
  };

  return (
    <div className="space-y-5 animate-in fade-in duration-200">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-2 border-b border-slate-800">
        <div>
          <h2 className="text-base font-bold text-white flex items-center gap-2">
            <Building className="w-5 h-5 text-sky-400" />
            <span>Grupos de Lojas & Filiais</span>
          </h2>
          <p className="text-xs text-slate-400">
            Organização hierárquica dos caixas e servidores locais por unidade varejista
          </p>
        </div>

        <button
          onClick={() => setShowAddModal(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-semibold transition-colors shadow-sm shadow-sky-600/30"
        >
          <Plus className="w-4 h-4" />
          <span>Cadastrar Nova Loja</span>
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {stores.map((store) => {
          const storeDevs = devices.filter((d) => d.store === store.name);
          const onlineDevs = storeDevs.filter((d) => d.status === 'online').length;

          return (
            <div
              key={store.id}
              className="bg-slate-900 border border-slate-800 rounded-xl p-4 text-xs space-y-3 hover:border-slate-700 transition-all shadow-md"
            >
              <div className="flex items-start justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="px-1.5 py-0.5 rounded bg-sky-950 border border-sky-800 text-sky-400 font-mono font-bold text-[10px]">
                      {store.code}
                    </span>
                    <h3 className="font-bold text-sm text-slate-100">{store.name}</h3>
                  </div>
                  <p className="text-[11px] text-slate-400 flex items-center gap-1 mt-1">
                    <MapPin className="w-3 h-3 text-slate-500" />
                    <span>{store.location}</span>
                  </p>
                </div>

                <div className="text-right">
                  <div className="text-xs font-bold text-slate-200">
                    <span className="text-emerald-400 font-mono">{onlineDevs}</span> /{' '}
                    <span className="font-mono text-slate-400">{storeDevs.length} PDVs</span>
                  </div>
                  <span className="text-[10px] text-slate-500">online agora</span>
                </div>
              </div>

              <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800/80 space-y-1 text-[11px] text-slate-400">
                <div className="flex justify-between">
                  <span>Gerente da Unidade:</span>
                  <span className="text-slate-200 font-medium">{store.manager}</span>
                </div>
                <div className="flex justify-between">
                  <span>Telefone Contato:</span>
                  <span className="text-slate-200 font-mono">{store.phone}</span>
                </div>
              </div>

              <div>
                <span className="text-[10px] uppercase font-bold text-slate-400 tracking-wider block mb-1.5">
                  Caixas Vinculados:
                </span>
                <div className="flex flex-wrap gap-1.5">
                  {storeDevs.length > 0 ? (
                    storeDevs.map((d) => (
                      <span
                        key={d.id}
                        className={`px-2 py-0.5 rounded text-[10px] font-mono flex items-center gap-1 border ${
                          d.status === 'online'
                            ? 'bg-emerald-950/60 border-emerald-800 text-emerald-300'
                            : 'bg-slate-800/60 border-slate-700 text-slate-400'
                        }`}
                      >
                        <span
                          className={`w-1.5 h-1.5 rounded-full ${
                            d.status === 'online' ? 'bg-emerald-400' : 'bg-slate-500'
                          }`}
                        ></span>
                        {d.id}
                      </span>
                    ))
                  ) : (
                    <span className="text-[11px] text-slate-500 italic">Nenhum PDV cadastrado nesta loja</span>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* MODAL: NOVA LOJA */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-md shadow-2xl p-5 text-xs text-slate-200">
            <h3 className="font-bold text-sm text-white mb-3">Cadastrar Nova Loja</h3>
            <form onSubmit={handleCreateStore} className="space-y-3">
              <div>
                <label className="block text-slate-400 font-medium mb-1">Nome da Loja:</label>
                <input
                  type="text"
                  required
                  placeholder="Ex: Loja Morumbi"
                  value={newStoreName}
                  onChange={(e) => setNewStoreName(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs text-white focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-slate-400 font-medium mb-1">Código da Unidade:</label>
                <input
                  type="text"
                  required
                  placeholder="Ex: LJ-05"
                  value={newStoreCode}
                  onChange={(e) => setNewStoreCode(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs font-mono text-white focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-slate-400 font-medium mb-1">Endereço / Localização:</label>
                <input
                  type="text"
                  placeholder="Av. Giovanni Gronchi, 3000"
                  value={newStoreLocation}
                  onChange={(e) => setNewStoreLocation(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs text-white focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-slate-400 font-medium mb-1">Gerente Responsável:</label>
                <input
                  type="text"
                  placeholder="Nome do gerente"
                  value={newStoreManager}
                  onChange={(e) => setNewStoreManager(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs text-white focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-slate-400 font-medium mb-1">Telefone da Loja:</label>
                <input
                  type="text"
                  placeholder="(11) 3740-0000"
                  value={newStorePhone}
                  onChange={(e) => setNewStorePhone(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs text-white focus:outline-none focus:border-sky-500"
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
                  Salvar Loja
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
