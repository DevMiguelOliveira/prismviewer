import { useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@prysmview/ui';
import { useAdminStore } from '../store/adminStore';
import { Store } from '@prysmview/types';
import { Building2, Monitor } from 'lucide-react';

export function Stores() {
  const { stores, refreshStores } = useAdminStore();

  useEffect(() => {
    refreshStores();
  }, [refreshStores]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-white">Grupos & Clientes</h2>
      </div>

      {stores.length === 0 ? (
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-12 text-center">
          <p className="text-gray-400">Nenhum grupo encontrado</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {stores.map((store) => (
            <StoreCard key={store.id} store={store} />
          ))}
        </div>
      )}
    </div>
  );
}

function StoreCard({ store }: { store: Store }) {
  const healthPercent = store.deviceCount > 0 ? (store.onlineCount / store.deviceCount) * 100 : 0;
  const healthColor = healthPercent >= 80 ? 'success' : healthPercent >= 50 ? 'warning' : 'danger';

  return (
    <Card hoverable>
      <CardHeader>
        <div className="flex items-start gap-3">
          <Building2 className="w-5 h-5 text-blue-400 mt-1" />
          <div>
            <CardTitle className="text-base">{store.name}</CardTitle>
            <p className="text-sm text-gray-400">{store.location}</p>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-gray-400">
              <Monitor className="w-4 h-4" />
              <span className="text-sm">Dispositivos</span>
            </div>
            <span className="text-white font-medium">{store.deviceCount}</span>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-400">Online</span>
            <Badge variant={healthColor}>
              {store.onlineCount}/{store.deviceCount}
            </Badge>
          </div>

          <div className="mt-4">
            <div className="w-full bg-gray-700 rounded-full h-2">
              <div
                className={`h-2 rounded-full ${
                  healthPercent >= 80
                    ? 'bg-green-500'
                    : healthPercent >= 50
                      ? 'bg-yellow-500'
                      : 'bg-red-500'
                }`}
                style={{ width: `${healthPercent}%` }}
              />
            </div>
            <p className="text-xs text-gray-500 mt-1">{healthPercent.toFixed(0)}% online</p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
