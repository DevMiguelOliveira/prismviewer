import { useState } from 'react';
import { WebRTCConnection, DEFAULT_WEBRTC_CONFIG } from '../lib/webrtc';

export function Connection() {
  const [status, setStatus] = useState<'idle' | 'connecting' | 'connected' | 'error'>('idle');
  const [connection, setConnection] = useState<WebRTCConnection | null>(null);

  const handleConnect = async () => {
    try {
      setStatus('connecting');
      const webrtc = new WebRTCConnection(DEFAULT_WEBRTC_CONFIG);
      await webrtc.createPeerConnection();
      setConnection(webrtc);
      setStatus('connected');
    } catch (error) {
      console.error('Connection error:', error);
      setStatus('error');
    }
  };

  const handleDisconnect = () => {
    connection?.close();
    setConnection(null);
    setStatus('idle');
  };

  return (
    <div className="p-4 border border-gray-700 rounded-lg bg-gray-800">
      <h3 className="text-lg font-bold mb-4">Conexão WebRTC</h3>
      <div className="flex items-center gap-4">
        <div
          className={`w-3 h-3 rounded-full ${
            status === 'connected'
              ? 'bg-green-500'
              : status === 'connecting'
                ? 'bg-yellow-500'
                : status === 'error'
                  ? 'bg-red-500'
                  : 'bg-gray-500'
          }`}
        />
        <span className="capitalize">{status}</span>
      </div>
      {status === 'idle' && (
        <button
          onClick={handleConnect}
          className="mt-4 px-4 py-2 bg-blue-600 rounded hover:bg-blue-700 transition"
        >
          Conectar
        </button>
      )}
      {status === 'connected' && (
        <button
          onClick={handleDisconnect}
          className="mt-4 px-4 py-2 bg-red-600 rounded hover:bg-red-700 transition"
        >
          Desconectar
        </button>
      )}
      <p className="mt-2 text-sm text-gray-400">STUN: stun.l.google.com:19302 (gratuito)</p>
    </div>
  );
}
