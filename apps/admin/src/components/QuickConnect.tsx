import { useState } from 'react';
import { Input, Button, Card, CardContent, CardHeader, CardTitle } from '@prysmview/ui';
import { useAdminStore } from '../store/adminStore';

export function QuickConnect() {
  const [partnerId, setPartnerId] = useState('');
  const [sessionCode, setSessionCode] = useState('');
  const { connectToDevice, isConnecting, connectionStatus } = useAdminStore();

  const handleConnect = async () => {
    if (!partnerId || !sessionCode) {
      alert('Preencha o ID do dispositivo e o código de sessão');
      return;
    }

    try {
      await connectToDevice(partnerId, sessionCode);
      alert('Conexão estabelecida com sucesso!');
    } catch (error) {
      alert('Erro ao conectar: ' + error);
    }
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Conexão Rápida</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <Input
              label="ID do Dispositivo"
              placeholder="Ex: DESKTOP-01 ou 192.168.1.100"
              value={partnerId}
              onChange={(e) => setPartnerId(e.target.value)}
            />

            <Input
              label="Código de Sessão (8 dígitos)"
              placeholder="00000000"
              value={sessionCode}
              onChange={(e) => setSessionCode(e.target.value)}
              maxLength={8}
            />

            <Button
              variant="primary"
              loading={isConnecting}
              disabled={!partnerId || !sessionCode || isConnecting}
              onClick={handleConnect}
              className="w-full"
            >
              {isConnecting ? 'Conectando...' : 'Conectar'}
            </Button>

            {connectionStatus === 'connected' && (
              <div className="mt-4 p-4 bg-green-900/30 border border-green-700 rounded-lg">
                <p className="text-green-400 text-center font-medium">✓ Conectado com sucesso</p>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Instruções</CardTitle>
        </CardHeader>
        <CardContent>
          <ol className="list-decimal list-inside space-y-2 text-gray-400 text-sm">
            <li>Solicite ao usuário do dispositivo remoto o ID do dispositivo</li>
            <li>O usuário irá gerar um código de sessão de 8 dígitos</li>
            <li>Digite o ID e o código nos campos acima</li>
            <li>Clique em "Conectar" para iniciar a sessão</li>
          </ol>
        </CardContent>
      </Card>
    </div>
  );
}
