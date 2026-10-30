import { useEffect, useState } from 'react';
import { Card, CardContent, Button, Badge } from '@prysmview/ui';
import { useHostStore } from '../store/hostStore';
import { Monitor, Shield, X, Copy, RefreshCw, Lock } from 'lucide-react';

export function HostPanel() {
  const {
    deviceId,
    deviceName,
    deviceIP,
    sessionCode,
    codeExpiresAt,
    isSessionActive,
    sessionStartTime,
    operatorName,
    generateSessionCode,
    endSession,
  } = useHostStore();

  const [sessionDuration, setSessionDuration] = useState(0);

  useEffect(() => {
    // Gerar ID fixo do dispositivo (simulado)
    // Em produção, isso viria do backend
  }, []);

  useEffect(() => {
    if (isSessionActive && sessionStartTime) {
      const interval = setInterval(() => {
        setSessionDuration(Math.floor((Date.now() - sessionStartTime.getTime()) / 1000));
      }, 1000);
      return () => clearInterval(interval);
    }
  }, [isSessionActive, sessionStartTime]);

  const handleCopyId = () => {
    navigator.clipboard.writeText(deviceId || '117 029 2035');
  };

  const handleCopyCode = () => {
    if (sessionCode) {
      navigator.clipboard.writeText(sessionCode);
    }
  };

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hours > 0) return `${hours}h ${minutes}m ${secs}s`;
    if (minutes > 0) return `${minutes}m ${secs}s`;
    return `${secs}s`;
  };

  const isCodeExpired = codeExpiresAt && new Date() > codeExpiresAt;

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 p-4">
      <div className="max-w-2xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <div className="flex items-center justify-center gap-3 mb-2">
            <Monitor className="w-8 h-8 text-blue-400" />
            <h1 className="text-3xl font-bold text-white">PrysmView Host</h1>
          </div>
          <p className="text-gray-400">Acesso Remoto Seguro</p>
        </div>

        {/* Main Card */}
        <Card className="border-gray-700 bg-gray-800/80 backdrop-blur">
          <CardContent className="p-6 space-y-6">
            {/* Device ID Section */}
            <div className="bg-gray-900/50 rounded-lg p-4 border border-gray-700">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-400">ID do Dispositivo</span>
                <Badge variant="success">Online</Badge>
              </div>
              <div className="flex items-center gap-3">
                <div className="flex-1">
                  <div className="text-2xl font-mono font-bold text-white tracking-wider">
                    {deviceId || '117 029 2035'}
                  </div>
                  <div className="text-xs text-gray-500 mt-1">
                    {deviceName} • {deviceIP}
                  </div>
                </div>
                <Button variant="ghost" size="sm" onClick={handleCopyId} className="shrink-0">
                  <Copy className="w-4 h-4" />
                </Button>
              </div>
            </div>

            {/* Session Code Section */}
            {!isSessionActive && (
              <div className="bg-gray-900/50 rounded-lg p-4 border border-gray-700">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-gray-400">Código de Sessão</span>
                  {sessionCode && (
                    <Badge variant={isCodeExpired ? 'danger' : 'info'}>
                      {isCodeExpired ? 'Expirado' : 'Válido'}
                    </Badge>
                  )}
                </div>
                <div className="flex items-center gap-3">
                  <div className="flex-1">
                    <div className="text-3xl font-mono font-bold text-blue-400 tracking-widest">
                      {sessionCode || '--------'}
                    </div>
                    {codeExpiresAt && !isCodeExpired && (
                      <div className="text-xs text-gray-500 mt-1">
                        Expira em {Math.ceil((codeExpiresAt.getTime() - Date.now()) / 1000)}s
                      </div>
                    )}
                  </div>
                  {sessionCode && !isCodeExpired && (
                    <Button variant="ghost" size="sm" onClick={handleCopyCode} className="shrink-0">
                      <Copy className="w-4 h-4" />
                    </Button>
                  )}
                </div>
                <div className="mt-3 flex gap-2">
                  <Button variant="primary" onClick={generateSessionCode} className="flex-1">
                    <RefreshCw className="w-4 h-4 mr-2" />
                    Gerar Novo Código
                  </Button>
                </div>
              </div>
            )}

            {/* Active Session Banner */}
            {isSessionActive && (
              <div className="bg-green-900/20 border border-green-700 rounded-lg p-4">
                <div className="flex items-center gap-3 mb-3">
                  <div className="p-2 bg-green-900/30 rounded-full">
                    <Shield className="w-5 h-5 text-green-400" />
                  </div>
                  <div className="flex-1">
                    <div className="text-green-400 font-medium">Sessão Ativa</div>
                    <div className="text-sm text-gray-400">Conectado com {operatorName}</div>
                  </div>
                  <div className="text-right">
                    <div className="text-green-400 font-mono font-bold">
                      {formatDuration(sessionDuration)}
                    </div>
                    <div className="text-xs text-gray-500">Duração</div>
                  </div>
                </div>
                <Button variant="danger" onClick={endSession} className="w-full">
                  <X className="w-4 h-4 mr-2" />
                  Encerrar Sessão
                </Button>
              </div>
            )}

            {/* Status Info */}
            <div className="flex items-center gap-2 text-sm text-gray-500">
              <Lock className="w-4 h-4" />
              <span>Conexão criptografada de ponta a ponta</span>
            </div>
          </CardContent>
        </Card>

        {/* Footer */}
        <div className="mt-6 text-center text-sm text-gray-500">
          <p>PrysmView v2.0.0 • Ready for Secure Remote Access</p>
        </div>
      </div>
    </div>
  );
}
