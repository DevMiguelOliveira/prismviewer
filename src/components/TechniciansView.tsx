import React, { useState } from 'react';
import { Operator } from '../types';
import {
  UserCheck,
  Shield,
  Plus,
  Mail,
  KeyRound,
  CheckCircle2,
  XCircle,
} from 'lucide-react';

interface TechniciansViewProps {
  technicians: Operator[];
  onAddTechnician: (tech: Operator) => void;
  onSendNotification: (msg: string, type?: 'success' | 'warning' | 'error') => void;
}

export const TechniciansView: React.FC<TechniciansViewProps> = ({
  technicians,
  onAddTechnician,
  onSendNotification,
}) => {
  const [showAddModal, setShowAddModal] = useState(false);
  const [name, setName] = useState('');
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [role, setRole] = useState<Operator['role']>('N1 - Suporte Balcão');

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !username.trim()) {
      onSendNotification('Preencha os campos obrigatórios do técnico.', 'warning');
      return;
    }

    const created: Operator = {
      id: `TECH-0${technicians.length + 1}`,
      name: name.trim(),
      username: username.trim(),
      email: email.trim() || `${username.trim()}@prysmview.com`,
      role,
      permissions: ['Acesso Remoto E2E', 'Controle Total', 'Chat Suporte'],
      status: 'Ativo',
      lastActive: 'Agora mesmo',
    };

    onAddTechnician(created);
    setShowAddModal(false);
    setName('');
    setUsername('');
    setEmail('');
    onSendNotification(`Técnico ${created.name} adicionado com perfil ${created.role}!`, 'success');
  };

  return (
    <div className="space-y-5 animate-in fade-in duration-200">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-2 border-b border-slate-800">
        <div>
          <h2 className="text-base font-bold text-white flex items-center gap-2">
            <UserCheck className="w-5 h-5 text-sky-400" />
            <span>Equipe Técnica & Controle de Acesso (RBAC)</span>
          </h2>
          <p className="text-xs text-slate-400">
            Gerenciamento de operadores autorizados a abrir sessões remotas nos caixas de loja
          </p>
        </div>

        <button
          onClick={() => setShowAddModal(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-semibold transition-colors shadow-sm shadow-sky-600/30"
        >
          <Plus className="w-4 h-4" />
          <span>Novo Técnico</span>
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {technicians.map((tech) => (
          <div
            key={tech.id}
            className="bg-slate-900 border border-slate-800 rounded-xl p-4 text-xs space-y-3 hover:border-slate-700 transition-all shadow-md"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-sky-600 to-indigo-600 flex items-center justify-center text-white font-bold text-sm">
                  {tech.name.charAt(0)}
                </div>
                <div>
                  <div className="flex items-center gap-2">
                    <h3 className="font-bold text-sm text-slate-100">{tech.name}</h3>
                    <span className="font-mono text-[10px] text-slate-500">{tech.id}</span>
                  </div>
                  <p className="text-[11px] text-sky-400 font-medium">{tech.role}</p>
                </div>
              </div>

              <span
                className={`px-2 py-0.5 rounded text-[10px] font-semibold ${
                  tech.status === 'Ativo'
                    ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                    : 'bg-slate-800 text-slate-500'
                }`}
              >
                {tech.status}
              </span>
            </div>

            <div className="p-2.5 rounded-lg bg-slate-950 border border-slate-800/80 space-y-1 text-[11px] text-slate-400">
              <div className="flex justify-between">
                <span>Usuário Login:</span>
                <span className="font-mono text-slate-200">{tech.username}</span>
              </div>
              <div className="flex justify-between">
                <span>E-mail Corporativo:</span>
                <span className="text-slate-200">{tech.email}</span>
              </div>
              <div className="flex justify-between">
                <span>Último Acesso:</span>
                <span className="text-emerald-400">{tech.lastActive}</span>
              </div>
            </div>

            <div>
              <span className="text-[10px] uppercase font-bold text-slate-400 tracking-wider block mb-1">
                Permissões no PDV:
              </span>
              <div className="flex flex-wrap gap-1">
                {tech.permissions.map((perm, idx) => (
                  <span
                    key={idx}
                    className="px-2 py-0.5 rounded bg-slate-800 border border-slate-700 text-slate-300 text-[10px]"
                  >
                    ✓ {perm}
                  </span>
                ))}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* MODAL: NOVO TÉCNICO */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-md shadow-2xl p-5 text-xs text-slate-200">
            <h3 className="font-bold text-sm text-white mb-3">Cadastrar Novo Técnico</h3>
            <form onSubmit={handleCreate} className="space-y-3">
              <div>
                <label className="block text-slate-400 font-medium mb-1">Nome Completo:</label>
                <input
                  type="text"
                  required
                  placeholder="Ex: Lucas Henrique Oliveira"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs text-white focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-slate-400 font-medium mb-1">Nome de Usuário (Login):</label>
                <input
                  type="text"
                  required
                  placeholder="Ex: lucas.oliveira"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs font-mono text-white focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-slate-400 font-medium mb-1">E-mail:</label>
                <input
                  type="email"
                  placeholder="lucas@empresa.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs text-white focus:outline-none focus:border-sky-500"
                />
              </div>

              <div>
                <label className="block text-slate-400 font-medium mb-1">Perfil / Cargo:</label>
                <select
                  value={role}
                  onChange={(e) => setRole(e.target.value as Operator['role'])}
                  className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2 text-xs text-white focus:outline-none focus:border-sky-500"
                >
                  <option value="N1 - Suporte Balcão">N1 - Suporte Balcão (Apenas Visualizar e Chat)</option>
                  <option value="N2 - Especialista Remoto">N2 - Especialista Remoto (Controle e Arquivos)</option>
                  <option value="N3 - Infraestrutura">N3 - Infraestrutura (SAS Ctrl+Alt+Del e Reboot)</option>
                  <option value="Administrador TI">Administrador TI (Acesso Geral e Auditoria)</option>
                </select>
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
                  Cadastrar Técnico
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
