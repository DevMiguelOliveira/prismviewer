import { Card, CardContent, CardHeader, CardTitle, Badge, Button } from '@prysmview/ui';
import { Device } from '@prysmview/types';
import { Monitor, Cpu, HardDrive, Wifi } from 'lucide-react';

interface DeviceCardProps {
  device: Device;
  onConnect: (device: Device) => void;
  onEdit: (device: Device) => void;
}

export function DeviceCard({ device, onConnect, onEdit }: DeviceCardProps) {
  const isOnline = device.status === 'online';

  return (
    <Card hoverable className="relative">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <Monitor className="w-5 h-5 text-blue-400" />
            <div>
              <CardTitle className="text-base">{device.name}</CardTitle>
              <p className="text-sm text-gray-400">{device.store}</p>
            </div>
          </div>
          <Badge variant={isOnline ? 'success' : 'danger'}>{isOnline ? 'Online' : 'Offline'}</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2 text-sm">
          <div className="flex items-center gap-2 text-gray-400">
            <Cpu className="w-4 h-4" />
            <span>{device.cpu || 'N/A'}</span>
          </div>
          <div className="flex items-center gap-2 text-gray-400">
            <HardDrive className="w-4 h-4" />
            <span>{device.memory || 'N/A'}</span>
          </div>
          <div className="flex items-center gap-2 text-gray-400">
            <Wifi className="w-4 h-4" />
            <span>{device.ip || 'N/A'}</span>
          </div>
          <div className="flex items-center gap-2 text-gray-400">
            <span className="text-xs">{device.os}</span>
            <span className="text-xs">•</span>
            <span className="text-xs">v{device.version}</span>
          </div>
        </div>

        <div className="mt-4 flex gap-2">
          <Button
            variant="primary"
            size="sm"
            disabled={!isOnline}
            onClick={() => onConnect(device)}
            className="flex-1"
          >
            Conectar
          </Button>
          <Button variant="ghost" size="sm" onClick={() => onEdit(device)}>
            Editar
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
