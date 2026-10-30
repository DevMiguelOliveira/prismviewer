// WebRTC com STUN público (Google)
// Custo zero, sem TURN server inicial

export interface WebRTCConfig {
  stunServers: string[];
  iceServers: RTCIceServer[];
}

export const DEFAULT_WEBRTC_CONFIG: WebRTCConfig = {
  stunServers: ['stun:stun.l.google.com:19302', 'stun:stun1.l.google.com:19302'],
  iceServers: [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:stun1.l.google.com:19302' }],
};

export class WebRTCConnection {
  private peerConnection: RTCPeerConnection | null = null;
  private dataChannel: RTCDataChannel | null = null;
  private config: WebRTCConfig;

  constructor(config: WebRTCConfig = DEFAULT_WEBRTC_CONFIG) {
    this.config = config;
  }

  async createPeerConnection(): Promise<RTCPeerConnection> {
    this.peerConnection = new RTCPeerConnection({
      iceServers: this.config.iceServers,
    });

    this.peerConnection.onicecandidate = (event) => {
      if (event.candidate) {
        // Enviar ICE candidate para o peer via sinalização
        console.log('ICE candidate:', event.candidate);
      }
    };

    this.peerConnection.onconnectionstatechange = () => {
      console.log('Connection state:', this.peerConnection?.connectionState);
    };

    return this.peerConnection;
  }

  async createOffer(): Promise<RTCSessionDescriptionInit> {
    if (!this.peerConnection) {
      throw new Error('Peer connection not created');
    }

    const offer = await this.peerConnection.createOffer();
    await this.peerConnection.setLocalDescription(offer);
    return offer;
  }

  async createAnswer(offer: RTCSessionDescriptionInit): Promise<RTCSessionDescriptionInit> {
    if (!this.peerConnection) {
      throw new Error('Peer connection not created');
    }

    await this.peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
    const answer = await this.peerConnection.createAnswer();
    await this.peerConnection.setLocalDescription(answer);
    return answer;
  }

  async setRemoteDescription(description: RTCSessionDescriptionInit): Promise<void> {
    if (!this.peerConnection) {
      throw new Error('Peer connection not created');
    }

    await this.peerConnection.setRemoteDescription(new RTCSessionDescription(description));
  }

  createDataChannel(label: string): RTCDataChannel {
    if (!this.peerConnection) {
      throw new Error('Peer connection not created');
    }

    this.dataChannel = this.peerConnection.createDataChannel(label);
    return this.dataChannel;
  }

  onTrack(callback: (event: RTCTrackEvent) => void): void {
    if (!this.peerConnection) {
      throw new Error('Peer connection not created');
    }

    this.peerConnection.ontrack = callback;
  }

  addTrack(track: MediaStreamTrack, stream: MediaStream): void {
    if (!this.peerConnection) {
      throw new Error('Peer connection not created');
    }

    this.peerConnection.addTrack(track, stream);
  }

  close(): void {
    if (this.dataChannel) {
      this.dataChannel.close();
    }
    if (this.peerConnection) {
      this.peerConnection.close();
    }
  }
}
