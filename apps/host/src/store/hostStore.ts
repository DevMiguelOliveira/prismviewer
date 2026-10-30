import { create } from 'zustand';

interface HostState {
  // Device Identity
  deviceId: string;
  deviceName: string;
  deviceIP: string;

  // API Configuration
  apiEndpoint: string;

  // Session Code
  sessionCode: string | null;
  codeExpiresAt: Date | null;
  attempts: number;
  maxAttempts: number;

  // Connection Status
  isSessionActive: boolean;
  sessionStartTime: Date | null;
  operatorName: string | null;

  // Consent
  showConsentDialog: boolean;
  consentTimer: number;
  consentAccepted: boolean;

  // Actions
  generateSessionCode: () => void;
  acceptConsent: (operatorName: string) => void;
  rejectConsent: () => void;
  endSession: () => void;
  updateDeviceInfo: (name: string, ip: string) => void;
}

export const useHostStore = create<HostState>((set, get) => ({
  // Device Identity
  deviceId: '',
  deviceName: 'Desktop',
  deviceIP: '192.168.1.100',

  // API Configuration
  apiEndpoint: 'https://prysmview.duckdns.org',

  // Session Code
  sessionCode: null,
  codeExpiresAt: null,
  attempts: 0,
  maxAttempts: 3,

  // Connection Status
  isSessionActive: false,
  sessionStartTime: null,
  operatorName: null,

  // Consent
  showConsentDialog: false,
  consentTimer: 60,
  consentAccepted: false,

  // Actions
  generateSessionCode: () => {
    const code = Math.floor(10000000 + Math.random() * 90000000).toString();
    const expiresAt = new Date(Date.now() + 120000); // 2 minutos

    set({
      sessionCode: code,
      codeExpiresAt: expiresAt,
      attempts: 0,
    });
  },

  acceptConsent: (operatorName: string) => {
    set({
      consentAccepted: true,
      showConsentDialog: false,
      operatorName,
      isSessionActive: true,
      sessionStartTime: new Date(),
    });
    get().generateSessionCode();
  },

  rejectConsent: () => {
    set({
      consentAccepted: false,
      showConsentDialog: false,
      operatorName: null,
    });
  },

  endSession: () => {
    set({
      isSessionActive: false,
      sessionStartTime: null,
      operatorName: null,
      sessionCode: null,
      codeExpiresAt: null,
      consentAccepted: false,
    });
  },

  updateDeviceInfo: (name: string, ip: string) => {
    set({
      deviceName: name,
      deviceIP: ip,
    });
  },
}));
