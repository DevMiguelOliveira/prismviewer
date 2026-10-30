import { useState } from 'react';
import { Tabs, TabsList, TabsTrigger } from '@prysmview/ui';
import { Login } from './components/Login';
import { Dashboard } from './components/Dashboard';
import { QuickConnect } from './components/QuickConnect';
import { Stores } from './components/Stores';
import { Audit } from './components/Audit';
import { Settings } from './components/Settings';
import { useAdminStore } from './store/adminStore';
import { LogOut, User } from 'lucide-react';

function App() {
  const { isAuthenticated, operator, logout } = useAdminStore();
  const [activeTab, setActiveTab] = useState('dashboard');

  if (!isAuthenticated) {
    return <Login />;
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Header */}
      <header className="h-14 bg-gray-800 border-b border-gray-700 flex items-center justify-between px-4">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold text-blue-400">PrysmView Admin</h1>
        </div>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <User className="w-4 h-4 text-gray-400" />
            <span className="text-sm text-gray-300">{operator?.name}</span>
          </div>
          <button
            onClick={logout}
            className="p-2 hover:bg-gray-700 rounded transition-colors"
            title="Sair"
          >
            <LogOut className="w-4 h-4 text-gray-400" />
          </button>
        </div>
      </header>

      {/* Layout com Sidebar e Conteúdo */}
      <div className="flex">
        {/* Sidebar */}
        <aside className="w-64 bg-gray-800 min-h-screen p-4">
          <nav className="space-y-2">
            <Tabs defaultValue="dashboard">
              <TabsList className="flex flex-col w-full">
                <TabsTrigger
                  value="dashboard"
                  onClick={() => setActiveTab('dashboard')}
                  className={activeTab === 'dashboard' ? 'bg-blue-600' : ''}
                >
                  Dispositivos
                </TabsTrigger>
                <TabsTrigger
                  value="direct"
                  onClick={() => setActiveTab('direct')}
                  className={activeTab === 'direct' ? 'bg-blue-600' : ''}
                >
                  Conexão Rápida
                </TabsTrigger>
                <TabsTrigger
                  value="stores"
                  onClick={() => setActiveTab('stores')}
                  className={activeTab === 'stores' ? 'bg-blue-600' : ''}
                >
                  Grupos & Clientes
                </TabsTrigger>
                <TabsTrigger
                  value="audit"
                  onClick={() => setActiveTab('audit')}
                  className={activeTab === 'audit' ? 'bg-blue-600' : ''}
                >
                  Auditoria
                </TabsTrigger>
                <TabsTrigger
                  value="settings"
                  onClick={() => setActiveTab('settings')}
                  className={activeTab === 'settings' ? 'bg-blue-600' : ''}
                >
                  Configurações
                </TabsTrigger>
              </TabsList>
            </Tabs>
          </nav>
        </aside>

        {/* Conteúdo Principal */}
        <main className="flex-1 p-6 overflow-auto">
          {activeTab === 'dashboard' && <Dashboard />}
          {activeTab === 'direct' && <QuickConnect />}
          {activeTab === 'stores' && <Stores />}
          {activeTab === 'audit' && <Audit />}
          {activeTab === 'settings' && <Settings />}
        </main>
      </div>
    </div>
  );
}

export default App;
