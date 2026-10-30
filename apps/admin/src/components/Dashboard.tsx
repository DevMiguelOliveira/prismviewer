import { useEffect } from 'react';
import { KPICard } from './KPICard';
import { DeviceCard } from './DeviceCard';
import { useAdminStore } from '../store/adminStore';
import { Device } from '@prysmview/types';

export function Dashboard() {
  const { kpis, devices, refreshKPIs, refreshDevices, setSelectedDevice } = useAdminStore();

  useEffect(() => {
    refreshKPIs();
    refreshDevices();
  }, [refreshKPIs, refreshDevices]);

  const handleConnect = (device: Device) => {
    setSelectedDevice(device);
    // Aqui seria aberto o modal de conexão com código de sessão
    alert(`Conectando a ${device.name} - ID: ${device.id}`);
  };

  const handleEdit = (device: Device) => {
    setSelectedDevice(device);
    // Aqui seria aberto o modal de edição
    alert(`Editando ${device.name}`);
  };

  return (
    <div className="space-y-6">
      {/* KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {kpis.map((kpi, index) => (
          <KPICard key={index} kpi={kpi} />
        ))}
      </div>

      {/* Devices Grid */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-white">Dispositivos</h2>
          <div className="flex gap-2">
            <span className="text-sm text-gray-400">
              {devices.filter((d) => d.status === 'online').length} online
            </span>
            <span className="text-gray-600">•</span>
            <span className="text-sm text-gray-400">
              {devices.filter((d) => d.status === 'offline').length} offline
            </span>
          </div>
        </div>

        {devices.length === 0 ? (
          <div className="bg-gray-800 border border-gray-700 rounded-lg p-12 text-center">
            <p className="text-gray-400">Nenhum dispositivo encontrado</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {devices.map((device) => (
              <DeviceCard
                key={device.id}
                device={device}
                onConnect={handleConnect}
                onEdit={handleEdit}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
