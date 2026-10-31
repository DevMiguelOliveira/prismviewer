/**
 * Serviço de Diagnóstico de Sinalização e WebRTC
 * Testa a integridade da comunicação com a API de sinalização e
 * verifica a geração de candidatos ICE pelo STUN/TURN no navegador/WebView.
 */

export interface TestResult {
  name: string;
  status: 'pending' | 'success' | 'warning' | 'error';
  latencyMs?: number;
  message: string;
  details?: string;
  suggestedFix?: string;
}

export const STUN_SERVERS = [
  'stun:stun.l.google.com:19302',
  'stun:stun1.l.google.com:19302',
  'stun:stun.cloudflare.com:3478',
];

/**
 * Testa o endpoint de sinalização HTTP/HTTPS
 */
export async function testSignalingHealth(url: string): Promise<TestResult> {
  const start = performance.now();
  const cleanUrl = url.replace(/\/+$/, '');
  let healthEndpoint = cleanUrl.endsWith('/health') ? cleanUrl : `${cleanUrl}/health`;

  // No navegador (seja HTTP localhost ou HTTPS), se o endpoint for prysmview.duckdns.org,
  // utiliza o proxy local do Vite (/api/duckdns/health) para evitar bloqueios de CORS e Mixed Content:
  if (typeof window !== 'undefined' && !(window as any).__TAURI__ && cleanUrl.includes('prysmview.duckdns.org')) {
    healthEndpoint = '/api/duckdns/health';
  }

  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 6000);

    const res = await fetch(healthEndpoint, {
      method: 'GET',
      mode: 'cors',
      signal: controller.signal,
    });
    clearTimeout(timeoutId);

    const latency = Math.round(performance.now() - start);

    if (res.ok) {
      let dataText = '';
      try {
        const json = await res.json();
        dataText = JSON.stringify(json);
      } catch {
        dataText = await res.text();
      }

      return {
        name: 'API de Sinalização',
        status: 'success',
        latencyMs: latency,
        message: `Servidor online e respondendo (HTTP ${res.status})`,
        details: `Resposta recebida em ${latency}ms: ${dataText.substring(0, 80)}`,
      };
    } else {
      return {
        name: 'API de Sinalização',
        status: 'warning',
        latencyMs: latency,
        message: `Servidor retornou status HTTP ${res.status}`,
        details: `O endpoint ${healthEndpoint} respondeu com código diferente de 200 OK.`,
        suggestedFix: 'Verifique se o serviço `prysmview-api` está em execução e ouvindo a rota /health.',
      };
    }
  } catch (err: any) {
    const latency = Math.round(performance.now() - start);
    const isTimeout = err.name === 'AbortError';

    if (isTimeout) {
      return {
        name: 'API de Sinalização',
        status: 'error',
        latencyMs: latency,
        message: 'Timeout ao conectar com a API de sinalização (6s)',
        details: `Não foi possível obter resposta de ${healthEndpoint}. A porta pode estar bloqueada pelo firewall.`,
        suggestedFix: 'Verifique se o servidor Caddy ou a VM Azure está ligada e se a porta 443 TCP está liberada no UFW/Azure NSG.',
      };
    }

    return {
      name: 'API de Sinalização',
      status: 'error',
      latencyMs: latency,
      message: 'Falha na conexão com a sinalização',
      details: err.message || 'Erro de rede ou política de CORS / Certificado SSL.',
      suggestedFix: 'Se o domínio DuckDNS mudou de IP, atualize o IP na Azure. No ambiente local, você pode apontar para http://127.0.0.1:8080.',
    };
  }
}

/**
 * Testa o funcionamento real do WebRTC no navegador / WebView2
 * Cria um PeerConnection de teste com os servidores STUN configurados
 * e verifica se o candidato de reflexão de NAT (srflx) é obtido.
 */
export async function testWebRTCStun(): Promise<TestResult> {
  const start = performance.now();

  if (typeof window === 'undefined' || !window.RTCPeerConnection) {
    return {
      name: 'Motor WebRTC / ICE',
      status: 'error',
      message: 'RTCPeerConnection não disponível no runtime',
      details: 'O motor WebView2 precisa de suporte a WebRTC habilitado.',
      suggestedFix: 'Atualize o Microsoft Edge WebView2 Runtime no Windows.',
    };
  }

  return new Promise((resolve) => {
    let resolved = false;
    let candidatesFound: string[] = [];

    const timeout = setTimeout(() => {
      if (!resolved) {
        resolved = true;
        pc.close();
        if (candidatesFound.length > 0) {
          resolve({
            name: 'Motor WebRTC / STUN',
            status: 'success',
            latencyMs: Math.round(performance.now() - start),
            message: `Candidatos ICE gerados (${candidatesFound.length} tipo(s))`,
            details: `Tipos encontrados: ${candidatesFound.join(', ')}`,
          });
        } else {
          resolve({
            name: 'Motor WebRTC / STUN',
            status: 'warning',
            latencyMs: 5000,
            message: 'Timeout ao obter candidatos STUN públicos',
            details: 'Apenas candidatos locais (host) ou timeout. O tráfego UDP pode estar restrito pelo roteador.',
            suggestedFix: 'Habilite o servidor TURN relay (coturn) para atravessar firewalls corporativos restritivos.',
          });
        }
      }
    }, 4500);

    const pc = new RTCPeerConnection({
      iceServers: [
        { urls: STUN_SERVERS },
      ],
    });

    // Cria um canal de teste para acionar o ICE gathering
    pc.createDataChannel('prysm_test_dc');

    pc.onicecandidate = (event) => {
      if (event.candidate && event.candidate.type) {
        const candidateType = event.candidate.type;
        if (!candidatesFound.includes(candidateType)) {
          candidatesFound.push(candidateType);
        }

        // Se encontrou candidato reflexivo de NAT (srflx), STUN funcionou com sucesso!
        if (candidateType === 'srflx' && !resolved) {
          resolved = true;
          clearTimeout(timeout);
          const latency = Math.round(performance.now() - start);
          pc.close();
          resolve({
            name: 'Motor WebRTC / STUN',
            status: 'success',
            latencyMs: latency,
            message: `STUN público verificado com sucesso (${latency}ms)`,
            details: `Candidato de reflexão de NAT (srflx) descoberto via ${STUN_SERVERS[0]}.`,
          });
        }
      }
    };

    pc.createOffer()
      .then((offer) => pc.setLocalDescription(offer))
      .catch((err) => {
        if (!resolved) {
          resolved = true;
          clearTimeout(timeout);
          pc.close();
          resolve({
            name: 'Motor WebRTC / STUN',
            status: 'error',
            message: 'Erro ao criar SDP Offer',
            details: err.message,
            suggestedFix: 'Verifique se há bloqueios nas políticas de segurança do navegador ou antivírus.',
          });
        }
      });
  });
}
