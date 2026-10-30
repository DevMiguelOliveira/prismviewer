// Tipos compartilhados entre Host e Admin

export interface Device {
  id: string
  name: string
  store: string
  status: 'online' | 'offline'
  lastSeen: Date
  os: string
  version: string
  ip?: string
  cpu?: string
  memory?: string
}

export interface Session {
  id: string
  deviceId: string
  deviceName: string
  operatorId: string
  operatorName: string
  startTime: Date
  endTime?: Date
  status: 'active' | 'ended' | 'rejected'
  bytesTransferred: number
  duration?: number
}

export interface ConnectionRequest {
  deviceId: string
  operatorId: string
  operatorName: string
  reason: string
  timestamp: Date
}

export interface SessionCode {
  code: string
  expiresAt: Date
  attempts: number
}

export interface Operator {
  id: string
  name: string
  email: string
  role: 'admin' | 'technician'
  createdAt: Date
  lastLogin?: Date
}

export interface Store {
  id: string
  name: string
  location: string
  deviceCount: number
  onlineCount: number
}

export interface KPI {
  label: string
  value: number
  unit?: string
  trend?: 'up' | 'down' | 'stable'
  change?: number
}

export interface Settings {
  theme: 'dark' | 'light'
  notifications: boolean
  autoConnect: boolean
  recordSessions: boolean
  apiEndpoint: string
  apiTimeout: number
}
