import { useState } from 'react';
import { Login } from './components/Login';
import { Dashboard } from './components/Dashboard';
import { QuickConnect } from './components/QuickConnect';
import { Stores } from './components/Stores';
import { Audit } from './components/Audit';
import { Settings } from './components/Settings';
import { useAdminStore } from './store/adminStore';
import { LogOut, User, Monitor, Zap, Users, FileText, Settings as SettingsIcon } from 'lucide-react';

type Tab = 'dashboard' | 'direct' | 'stores' | 'audit' | 'settings';

function App() {
  const { isAuthenticated, operator, logout } = useAdminStore();
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');

  if (!isAuthenticated) {
    return <Login />;
  }

  const tabs = [
    { id: 'dashboard' as Tab, label: 'Dispositivos', icon: Monitor },
    { id: 'direct' as Tab, label: 'Conexão Rápida', icon: Zap },
    { id: 'stores' as Tab, label: 'Grupos & Clientes', icon: Users },
    { id: 'audit' as Tab, label: 'Auditoria', icon: FileText },
    { id: 'settings' as Tab, label: 'Configurações', icon: SettingsIcon },
  ];

  const renderContent = () => {
    switch (activeTab) {
      case 'dashboard':
        return <Dashboard />;
      case 'direct':
        return <QuickConnect />;
      case 'stores':
        return <Stores />;
      case 'audit':
        return <Audit />;
      case 'settings':
        return <Settings />;
      default:
        return <Dashboard />;
    }
  };

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
        <aside className="w-64 bg-gray-800 min-h-screen p-4 border-r border-gray-700">
          <nav className="space-y-2">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                    activeTab === tab.id
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-300 hover:bg-gray-700'
                  }`}
                >
                  <Icon className="w-5 h-5" />
                  <span className="font-medium">{tab.label}</span>
                </button>
              );
            })}
          </nav>
        </aside>

        {/* Conteúdo Principal */}
        <main className="flex-1 p-6 overflow-auto">
          {renderContent()}
        </main>
      </div>
    </div>
  );
}

export default App;
