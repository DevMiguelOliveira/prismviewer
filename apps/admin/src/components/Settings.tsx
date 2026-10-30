import { Card, CardContent, CardHeader, CardTitle, Button, Input } from '@prysmview/ui';
import { useAdminStore } from '../store/adminStore';
import { User, Shield, Palette, Server } from 'lucide-react';

export function Settings() {
  const { settings, updateSettings, operator, logout } = useAdminStore();

  const handleSave = () => {
    // Aqui seria salvo no localStorage ou na API
    localStorage.setItem('prysmview-settings', JSON.stringify(settings));
    alert('Configurações salvas com sucesso!');
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-white">Configurações</h2>
        <Button variant="primary" onClick={handleSave}>
          Salvar Alterações
        </Button>
      </div>

      {/* Profile Section */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <User className="w-5 h-5 text-blue-400" />
            <CardTitle>Perfil</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-2">Nome</label>
              <Input value={operator?.name || ''} disabled />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-2">Email</label>
              <Input value={operator?.email || ''} disabled />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-2">Função</label>
              <Input value={operator?.role === 'admin' ? 'Administrador' : 'Técnico'} disabled />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Security Section */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Shield className="w-5 h-5 text-blue-400" />
            <CardTitle>Segurança</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-white font-medium">Notificações</p>
                <p className="text-sm text-gray-400">Receber alertas de sessões</p>
              </div>
              <button
                onClick={() => updateSettings({ notifications: !settings.notifications })}
                className={`w-12 h-6 rounded-full transition-colors ${
                  settings.notifications ? 'bg-blue-600' : 'bg-gray-600'
                }`}
              >
                <div
                  className={`w-4 h-4 bg-white rounded-full transition-transform ${
                    settings.notifications ? 'translate-x-7' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="text-white font-medium">Auto Conectar</p>
                <p className="text-sm text-gray-400">
                  Conectar automaticamente a dispositivos confiáveis
                </p>
              </div>
              <button
                onClick={() => updateSettings({ autoConnect: !settings.autoConnect })}
                className={`w-12 h-6 rounded-full transition-colors ${
                  settings.autoConnect ? 'bg-blue-600' : 'bg-gray-600'
                }`}
              >
                <div
                  className={`w-4 h-4 bg-white rounded-full transition-transform ${
                    settings.autoConnect ? 'translate-x-7' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="text-white font-medium">Gravar Sessões</p>
                <p className="text-sm text-gray-400">Registrar sessões para auditoria</p>
              </div>
              <button
                onClick={() => updateSettings({ recordSessions: !settings.recordSessions })}
                className={`w-12 h-6 rounded-full transition-colors ${
                  settings.recordSessions ? 'bg-blue-600' : 'bg-gray-600'
                }`}
              >
                <div
                  className={`w-4 h-4 bg-white rounded-full transition-transform ${
                    settings.recordSessions ? 'translate-x-7' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Appearance Section */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Palette className="w-5 h-5 text-blue-400" />
            <CardTitle>Aparência</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-2">Tema</label>
              <div className="flex gap-2">
                <button
                  onClick={() => updateSettings({ theme: 'dark' })}
                  className={`flex-1 p-4 rounded-lg border transition-colors ${
                    settings.theme === 'dark'
                      ? 'border-blue-600 bg-blue-900/20'
                      : 'border-gray-700 hover:border-gray-600'
                  }`}
                >
                  <div className="w-full h-8 bg-gray-900 rounded mb-2" />
                  <p className="text-sm text-white">Escuro</p>
                </button>
                <button
                  onClick={() => updateSettings({ theme: 'light' })}
                  className={`flex-1 p-4 rounded-lg border transition-colors ${
                    settings.theme === 'light'
                      ? 'border-blue-600 bg-blue-900/20'
                      : 'border-gray-700 hover:border-gray-600'
                  }`}
                >
                  <div className="w-full h-8 bg-gray-200 rounded mb-2" />
                  <p className="text-sm text-white">Claro</p>
                </button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* API Section */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Server className="w-5 h-5 text-blue-400" />
            <CardTitle>Configurações da API</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-2">
                Endpoint da API
              </label>
              <Input
                value={settings.apiEndpoint}
                onChange={(e) => updateSettings({ apiEndpoint: e.target.value })}
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-2">Timeout (ms)</label>
              <Input
                type="number"
                value={settings.apiTimeout}
                onChange={(e) => updateSettings({ apiTimeout: parseInt(e.target.value) })}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Logout */}
      <Card>
        <CardContent className="pt-6">
          <Button variant="danger" onClick={logout} className="w-full">
            Sair da Conta
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
