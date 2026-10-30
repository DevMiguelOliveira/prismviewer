import { create } from 'zustand';
import { Device, Session, Operator, Store, Settings, KPI } from '@prysmview/types';

interface AdminState {
  // Auth
  isAuthenticated: boolean;
  operator: Operator | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;

  // Devices
  devices: Device[];
  selectedDevice: Device | null;
  setSelectedDevice: (device: Device | null) => void;
  refreshDevices: () => Promise<void>;

  // Sessions
  sessions: Session[];
  activeSession: Session | null;
  refreshSessions: () => Promise<void>;

  // Stores
  stores: Store[];
  refreshStores: () => Promise<void>;

  // KPIs
  kpis: KPI[];
  refreshKPIs: () => Promise<void>;

  // Settings
  settings: Settings;
  updateSettings: (settings: Partial<Settings>) => void;

  // Connection
  isConnecting: boolean;
  connectionStatus: 'idle' | 'connecting' | 'connected' | 'error';
  connectToDevice: (deviceId: string, sessionCode: string) => Promise<void>;
  disconnect: () => void;
}

export const useAdminStore = create<AdminState>((set, get) => ({
  // Auth
  isAuthenticated: false,
  operator: null,
  login: async (email: string, _password: string) => {
    // Simulação - conectar com API real depois
    await new Promise((resolve) => setTimeout(resolve, 1000));
    set({
      isAuthenticated: true,
      operator: {
        id: 'op-1',
        name: 'Técnico Demo',
        email,
        role: 'technician',
        createdAt: new Date(),
        lastLogin: new Date(),
      },
    });
  },
  logout: () => {
    set({
      isAuthenticated: false,
      operator: null,
      activeSession: null,
      connectionStatus: 'idle',
    });
  },

  // Devices
  devices: [],
  selectedDevice: null,
  setSelectedDevice: (device) => set({ selectedDevice: device }),
  refreshDevices: async () => {
    // Simulação - conectar com API real depois
    await new Promise((resolve) => setTimeout(resolve, 500));
    set({
      devices: [
        {
          id: 'dev-1',
          name: 'DESKTOP-01',
          store: 'Loja Centro',
          status: 'online',
          lastSeen: new Date(),
          os: 'Windows 11',
          version: '2.0.0',
          ip: '192.168.1.100',
          cpu: 'Intel Core i5-10400',
          memory: '16 GB',
        },
        {
          id: 'dev-2',
          name: 'DESKTOP-02',
          store: 'Loja Norte',
          status: 'offline',
          lastSeen: new Date(Date.now() - 3600000),
          os: 'Windows 10',
          version: '2.0.0',
          ip: '192.168.1.101',
          cpu: 'Intel Core i3-9100',
          memory: '8 GB',
        },
      ],
    });
  },

  // Sessions
  sessions: [],
  activeSession: null,
  refreshSessions: async () => {
    // Simulação
    await new Promise((resolve) => setTimeout(resolve, 500));
    set({
      sessions: [
        {
          id: 'sess-1',
          deviceId: 'dev-1',
          deviceName: 'DESKTOP-01',
          operatorId: 'op-1',
          operatorName: 'Técnico Demo',
          startTime: new Date(Date.now() - 1800000),
          status: 'ended',
          bytesTransferred: 52428800,
          duration: 1800,
        },
      ],
    });
  },

  // Stores
  stores: [],
  refreshStores: async () => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    set({
      stores: [
        {
          id: 'store-1',
          name: 'Loja Centro',
          location: 'São Paulo - Centro',
          deviceCount: 5,
          onlineCount: 4,
        },
        {
          id: 'store-2',
          name: 'Loja Norte',
          location: 'São Paulo - Norte',
          deviceCount: 3,
          onlineCount: 2,
        },
      ],
    });
  },

  // KPIs
  kpis: [],
  refreshKPIs: async () => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    set({
      kpis: [
        { label: 'Dispositivos Online', value: 6, unit: 'un', trend: 'up', change: 2 },
        { label: 'Sessões Ativas', value: 0, unit: 'un', trend: 'stable', change: 0 },
        { label: 'Banda Total', value: 1.2, unit: 'GB', trend: 'up', change: 15 },
        { label: 'Tempo Médio', value: 12, unit: 'min', trend: 'down', change: -5 },
      ],
    });
  },

  // Settings
  settings: {
    theme: 'dark',
    notifications: true,
    autoConnect: false,
    recordSessions: false,
    apiEndpoint: 'https://prysmview.duckdns.org',
    apiTimeout: 30000,
  },
  updateSettings: (newSettings) => {
    set((state) => ({
      settings: { ...state.settings, ...newSettings },
    }));
  },

  // Connection
  isConnecting: false,
  connectionStatus: 'idle',
  connectToDevice: async (deviceId: string, _sessionCode: string) => {
    set({ isConnecting: true, connectionStatus: 'connecting' });
    // Simulação
    await new Promise((resolve) => setTimeout(resolve, 2000));
    set({
      isConnecting: false,
      connectionStatus: 'connected',
      activeSession: {
        id: 'sess-new',
        deviceId,
        deviceName: get().devices.find((d) => d.id === deviceId)?.name || 'Unknown',
        operatorId: get().operator?.id || '',
        operatorName: get().operator?.name || '',
        startTime: new Date(),
        status: 'active',
        bytesTransferred: 0,
      },
    });
  },
  disconnect: () => {
    set({
      connectionStatus: 'idle',
      activeSession: null,
    });
  },
}));
