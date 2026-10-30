import { Card, CardContent, CardHeader, CardTitle, Button } from '@prysmview/ui';
import { useHostStore } from '../store/hostStore';
import { Shield, X, AlertTriangle } from 'lucide-react';
import { useEffect, useState } from 'react';

export function ConsentDialog() {
  const { showConsentDialog, consentTimer, acceptConsent, rejectConsent } = useHostStore();

  const [timer, setTimer] = useState(consentTimer);

  useEffect(() => {
    if (!showConsentDialog) return;

    const interval = setInterval(() => {
      setTimer((prev) => {
        if (prev <= 1) {
          rejectConsent();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, [showConsentDialog, rejectConsent]);

  if (!showConsentDialog) return null;

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
      <Card className="w-full max-w-md border-yellow-600 bg-gray-800">
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-yellow-900/30 rounded-full">
              <AlertTriangle className="w-6 h-6 text-yellow-400" />
            </div>
            <CardTitle className="text-yellow-400">Solicitação de Acesso Remoto</CardTitle>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="bg-yellow-900/20 border border-yellow-700 rounded-lg p-4">
            <p className="text-sm text-yellow-200">
              Um técnico está solicitando acesso a este computador.
            </p>
          </div>

          <div className="space-y-2 text-sm">
            <div className="flex items-center gap-2 text-gray-300">
              <Shield className="w-4 h-4 text-blue-400" />
              <span>Todas as ações serão visíveis na tela</span>
            </div>
            <div className="flex items-center gap-2 text-gray-300">
              <Shield className="w-4 h-4 text-blue-400" />
              <span>Você pode encerrar a sessão a qualquer momento</span>
            </div>
            <div className="flex items-center gap-2 text-gray-300">
              <Shield className="w-4 h-4 text-blue-400" />
              <span>A conexão é criptografada de ponta a ponta</span>
            </div>
          </div>

          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Tempo restante:</span>
            <span
              className={`font-mono font-bold ${timer <= 10 ? 'text-red-400' : 'text-yellow-400'}`}
            >
              {timer}s
            </span>
          </div>

          <div className="flex gap-3 pt-2">
            <Button variant="secondary" onClick={rejectConsent} className="flex-1">
              <X className="w-4 h-4 mr-2" />
              Recusar
            </Button>
            <Button
              variant="primary"
              onClick={() => acceptConsent('Técnico Demo')}
              className="flex-1"
            >
              <Shield className="w-4 h-4 mr-2" />
              Aceitar
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
