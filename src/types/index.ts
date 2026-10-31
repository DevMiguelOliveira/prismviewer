export interface Device {
  id: string;
  name: string;
  store: string;
  status: 'online' | 'offline';
  lastSeen: string;
  os: string;
  version: string;
  ip: string;
  cpu: string;
  memory: string;
  disk: string;
  unattendedPassword?: string;
  activeOperator?: string;
  resolution: string;
  monitors: number;
  uptime: string;
}

export interface Session {
  id: string;
  deviceId: string;
  deviceName: string;
  operatorId: string;
  operatorName: string;
  store: string;
  startTime: string;
  endTime?: string;
  durationSeconds?: number;
  status: 'active' | 'ended' | 'rejected';
  bytesTransferred: number;
  consentType: 'Código de 8 Dígitos' | 'Acesso Desassistido' | 'Consentimento Manual';
  reason: string;
  auditHash: string;
}

export interface Operator {
  id: string;
  name: string;
  username: string;
  email: string;
  role: 'Administrador TI' | 'N1 - Suporte Balcão' | 'N2 - Especialista Remoto' | 'N3 - Infraestrutura';
  permissions: string[];
  status: 'Ativo' | 'Inativo';
  lastActive: string;
}

export interface Store {
  id: string;
  name: string;
  code: string;
  location: string;
  manager: string;
  phone: string;
  deviceCount: number;
  onlineCount: number;
}

export interface KPI {
  label: string;
  value: string | number;
  unit?: string;
  trend?: 'up' | 'down' | 'stable';
  change?: string;
  subtext?: string;
}

export interface Settings {
  theme: 'dark' | 'light';
  notifications: boolean;
  autoConnect: boolean;
  clipboardSync: boolean;
  fileTransferEnabled: boolean;
  apiEndpoint: string;
  apiTimeout: number;
  stunServer: string;
  turnServer: string;
  installedVersion: string;
  channel: 'stable' | 'beta';
}

export interface FileTransferItem {
  id: string;
  name: string;
  size: number;
  direction: 'upload' | 'download';
  sha256: string;
  progress: number;
  status: 'pending' | 'transferring' | 'completed' | 'verified' | 'failed';
  timestamp: string;
}

export interface ChatMessage {
  id: string;
  sender: 'operator' | 'host';
  senderName: string;
  text: string;
  timestamp: string;
}
