import React, { useState } from 'react';
import { Session } from '../types';
import {
  FileText,
  Search,
  Download,
  ShieldCheck,
  Clock,
  User,
  HardDrive,
  Hash,
  ExternalLink,
  CheckCircle,
} from 'lucide-react';

interface AuditViewProps {
  sessions: Session[];
  onSendNotification: (msg: string, type?: 'success' | 'warning' | 'error') => void;
}

export const AuditView: React.FC<AuditViewProps> = ({ sessions, onSendNotification }) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedSession, setSelectedSession] = useState<Session | null>(null);

  const filteredSessions = sessions.filter(
    (s) =>
      s.deviceId.toLowerCase().includes(searchTerm.toLowerCase()) ||
      s.operatorName.toLowerCase().includes(searchTerm.toLowerCase()) ||
      s.store.toLowerCase().includes(searchTerm.toLowerCase()) ||
      s.reason.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleExportCSV = () => {
    const headers = ['ID', 'Dispositivo', 'Grupo_Departamento', 'Operador', 'Inicio', 'Fim', 'Duracao_s', 'Bytes', 'Consentimento', 'Motivo', 'SHA256_Auditoria'];
    const rows = filteredSessions.map((s) => [
      s.id,
      s.deviceId,
      `"${s.store}"`,
      `"${s.operatorName}"`,
      `"${s.startTime}"`,
      `"${s.endTime || ''}"`,
      s.durationSeconds || 0,
      s.bytesTransferred,
      `"${s.consentType}"`,
      `"${s.reason}"`,
      s.auditHash,
    ]);

    const csvContent = 'data:text/csv;charset=utf-8,' + [headers.join(','), ...rows.map((e) => e.join(','))].join('\n');
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement('a');
    link.setAttribute('href', encodedUri);
    link.setAttribute('download', `auditoria_prysmview_${new Date().toISOString().slice(0, 10)}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    onSendNotification('Relatório de auditoria exportado em formato CSV!', 'success');
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  return (
    <div className="space-y-5 animate-in fade-in duration-200">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-2 border-b border-slate-800">
        <div>
          <h2 className="text-base font-bold text-white flex items-center gap-2">
            <ShieldCheck className="w-5 h-5 text-emerald-400" />
            <span>Auditoria E2E & Registros de Conformidade</span>
          </h2>
          <p className="text-xs text-slate-400">
            Registro imutável de todas as sessões remotas conforme a regra 9 do AGENTS.md (sem telas ou teclas)
          </p>
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto">
          <div className="relative flex-1 sm:w-64">
            <Search className="w-3.5 h-3.5 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder="Filtrar por técnico, dispositivo ou motivo..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full bg-slate-900 border border-slate-800 rounded-lg pl-8 pr-3 py-1.5 text-xs text-white focus:outline-none focus:border-sky-500"
            />
          </div>

          <button
            onClick={handleExportCSV}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg text-xs font-semibold border border-slate-700 transition-colors shrink-0"
          >
            <Download className="w-3.5 h-3.5 text-sky-400" />
            <span>Exportar CSV</span>
          </button>
        </div>
      </div>

      {/* Audit Table */}
      <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-lg">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs text-slate-300">
            <thead className="bg-slate-950/80 text-[11px] text-slate-400 uppercase tracking-wider border-b border-slate-800">
              <tr>
                <th className="py-2.5 px-4">Sessão / Horário</th>
                <th className="py-2.5 px-4">Dispositivo Alvo</th>
                <th className="py-2.5 px-4">Operador / Grupo</th>
                <th className="py-2.5 px-4">Consentimento & Motivo</th>
                <th className="py-2.5 px-4">Volume Dados</th>
                <th className="py-2.5 px-4">Integridade SHA-256</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800">
              {filteredSessions.map((sess) => (
                <tr
                  key={sess.id}
                  onClick={() => setSelectedSession(sess)}
                  className="hover:bg-slate-800/40 cursor-pointer transition-colors"
                >
                  <td className="py-3 px-4">
                    <div className="font-mono font-bold text-sky-400">{sess.id}</div>
                    <div className="text-[10px] text-slate-400 flex items-center gap-1 mt-0.5">
                      <Clock className="w-3 h-3 text-slate-500" />
                      <span>{sess.startTime}</span>
                      {sess.durationSeconds && (
                        <span className="text-slate-500">
                          ({Math.round(sess.durationSeconds / 60)} min)
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="py-3 px-4">
                    <div className="font-mono font-bold text-slate-200">{sess.deviceId}</div>
                    <div className="text-[10px] text-slate-400">{sess.deviceName}</div>
                  </td>
                  <td className="py-3 px-4">
                    <div className="font-medium text-slate-200">{sess.operatorName}</div>
                    <div className="text-[10px] text-slate-400">{sess.store}</div>
                  </td>
                  <td className="py-3 px-4 max-w-xs">
                    <span className="inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold bg-sky-950 text-sky-400 border border-sky-800 mb-1">
                      {sess.consentType}
                    </span>
                    <div className="text-[11px] text-slate-300 truncate" title={sess.reason}>
                      {sess.reason}
                    </div>
                  </td>
                  <td className="py-3 px-4 font-mono text-[11px] text-slate-300">
                    {formatBytes(sess.bytesTransferred)}
                  </td>
                  <td className="py-3 px-4">
                    <div className="flex items-center gap-1.5 text-emerald-400 font-mono text-[10px]">
                      <CheckCircle className="w-3 h-3 text-emerald-400" />
                      <span>{sess.auditHash.substring(0, 16)}...</span>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* SESSION AUDIT DETAIL MODAL */}
      {selectedSession && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl w-full max-w-lg shadow-2xl p-5 text-xs text-slate-200">
            <div className="flex items-center justify-between pb-3 border-b border-slate-800 mb-4">
              <div className="flex items-center gap-2">
                <FileText className="w-4 h-4 text-sky-400" />
                <h3 className="font-bold text-sm text-white">Certificado de Auditoria da Sessão</h3>
              </div>
              <button onClick={() => setSelectedSession(null)} className="text-slate-400 hover:text-white">✕</button>
            </div>

            <div className="space-y-3">
              <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 space-y-1.5 font-mono text-[11px]">
                <div className="flex justify-between">
                  <span className="text-slate-500">ID da Sessão:</span>
                  <span className="text-sky-400 font-bold">{selectedSession.id}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Horário:</span>
                  <span className="text-slate-200">{selectedSession.startTime} até {selectedSession.endTime}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Operador:</span>
                  <span className="text-slate-200">{selectedSession.operatorName} ({selectedSession.operatorId})</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Dispositivo Remoto:</span>
                  <span className="text-slate-200">{selectedSession.deviceId} [{selectedSession.store}]</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Consentimento:</span>
                  <span className="text-emerald-400">{selectedSession.consentType}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Volume Trafegado:</span>
                  <span className="text-slate-200">{formatBytes(selectedSession.bytesTransferred)}</span>
                </div>
              </div>

              <div>
                <span className="text-slate-400 font-semibold block mb-1">Motivo Registrado:</span>
                <p className="p-2.5 rounded-lg bg-slate-950 border border-slate-800 text-slate-300">
                  {selectedSession.reason}
                </p>
              </div>

              <div>
                <span className="text-slate-400 font-semibold block mb-1 flex items-center gap-1">
                  <Hash className="w-3.5 h-3.5 text-emerald-400" />
                  <span>Assinatura Imutável SHA-256:</span>
                </span>
                <p className="p-2.5 rounded-lg bg-slate-950 border border-slate-800 text-emerald-400 font-mono text-[10px] break-all">
                  {selectedSession.auditHash}
                </p>
              </div>
            </div>

            <div className="pt-4 mt-4 border-t border-slate-800 flex justify-end">
              <button
                onClick={() => setSelectedSession(null)}
                className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg text-xs"
              >
                Fechar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
