import { useEffect } from 'react';
import { Card, CardContent, Badge } from '@prysmview/ui';
import { useAdminStore } from '../store/adminStore';
import { Session } from '@prysmview/types';

export function Audit() {
  const { sessions, refreshSessions } = useAdminStore();

  useEffect(() => {
    refreshSessions();
  }, [refreshSessions]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-white">Auditoria</h2>
      </div>

      {sessions.length === 0 ? (
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-12 text-center">
          <p className="text-gray-400">Nenhuma sessão registrada</p>
        </div>
      ) : (
        <Card>
          <CardContent className="p-0">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-700">
                  <th className="text-left p-4 text-sm font-medium text-gray-400">Dispositivo</th>
                  <th className="text-left p-4 text-sm font-medium text-gray-400">Operador</th>
                  <th className="text-left p-4 text-sm font-medium text-gray-400">Início</th>
                  <th className="text-left p-4 text-sm font-medium text-gray-400">Duração</th>
                  <th className="text-left p-4 text-sm font-medium text-gray-400">Status</th>
                  <th className="text-left p-4 text-sm font-medium text-gray-400">Transferido</th>
                </tr>
              </thead>
              <tbody>
                {sessions.map((session) => (
                  <SessionRow key={session.id} session={session} />
                ))}
              </tbody>
            </table>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function SessionRow({ session }: { session: Session }) {
  const getStatusBadge = () => {
    switch (session.status) {
      case 'active':
        return <Badge variant="warning">Ativa</Badge>;
      case 'ended':
        return <Badge variant="success">Finalizada</Badge>;
      case 'rejected':
        return <Badge variant="danger">Rejeitada</Badge>;
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatDuration = (seconds?: number) => {
    if (!seconds) return '-';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m ${secs}s`;
    return `${secs}s`;
  };

  const formatDate = (date: Date) => {
    return new Date(date).toLocaleString('pt-BR');
  };

  return (
    <tr className="border-b border-gray-700 hover:bg-gray-800/50">
      <td className="p-4 text-white">{session.deviceName}</td>
      <td className="p-4 text-gray-300">{session.operatorName}</td>
      <td className="p-4 text-gray-400 text-sm">{formatDate(session.startTime)}</td>
      <td className="p-4 text-gray-400 text-sm">{formatDuration(session.duration)}</td>
      <td className="p-4">{getStatusBadge()}</td>
      <td className="p-4 text-gray-400 text-sm">{formatBytes(session.bytesTransferred)}</td>
    </tr>
  );
}
