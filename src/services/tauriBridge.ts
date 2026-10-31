/**
 * Tauri Bridge - Camada de compatibilidade segura para WebView e Desktop
 * Evita o clássico erro: "Cannot read properties of undefined (reading 'invoke')"
 * quando a aplicação roda dentro do WebView2 ou em navegador padrão.
 */

export interface SystemDiagnosticInfo {
  isTauri: boolean;
  userAgent: string;
  platform: string;
  hasWebRTC: boolean;
  hasCrypto: boolean;
  hasLocalStorage: boolean;
}

export const getDiagnosticInfo = (): SystemDiagnosticInfo => {
  const isTauri = typeof window !== 'undefined' && Boolean(
    (window as any).__TAURI__ ||
    (window as any).__TAURI_IPC__ ||
    (window as any).ipc
  );

  return {
    isTauri,
    userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : 'Node/Unknown',
    platform: typeof navigator !== 'undefined' ? navigator.platform : 'Unknown',
    hasWebRTC: typeof window !== 'undefined' && Boolean(window.RTCPeerConnection),
    hasCrypto: typeof window !== 'undefined' && Boolean(window.crypto && window.crypto.subtle),
    hasLocalStorage: (() => {
      try {
        localStorage.setItem('__test__', '1');
        localStorage.removeItem('__test__');
        return true;
      } catch {
        return false;
      }
    })(),
  };
};

/**
 * Executa um comando IPC no Rust de forma segura.
 * Se o Tauri não estiver carregado (ou em ambiente web de desenvolvimento),
 * utiliza fallback seguro sem quebrar o ciclo de vida do React.
 */
export async function invokeSafe<T>(command: string, args: Record<string, any> = {}): Promise<T | null> {
  if (typeof window !== 'undefined') {
    // Tauri 1.x e 2.x global
    const tauri = (window as any).__TAURI__;
    if (tauri && tauri.invoke) {
      try {
        return await tauri.invoke(command, args);
      } catch (err) {
        console.warn(`[Tauri IPC Error on '${command}']`, err);
        throw err;
      }
    }

    // Tauri legacy webview postMessage
    if ((window as any).ipc && typeof (window as any).ipc.postMessage === 'function') {
      try {
        (window as any).ipc.postMessage(JSON.stringify({ action: command, ...args }));
        return null;
      } catch (err) {
        console.warn(`[IPC postMessage Error on '${command}']`, err);
      }
    }
  }

  // Fallback seguro em modo navegador / simulação
  console.info(`[Tauri Bridge Mock] Comando '${command}' executado no modo simulação web:`, args);
  return null;
}
