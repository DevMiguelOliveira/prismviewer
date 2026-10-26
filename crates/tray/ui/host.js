// =============================================================================
// PRYSMVIEW HOST - CONTROLADOR JAVASCRIPT DO CLIENTE WEBVIEW2
// =============================================================================

let hostState = {
  isConnected: false,
  pdvId: 'PDV-CAIXA-01',
  isSessionActive: false,
  currentOperator: null,
  currentCode: null,
  sessionCodeExpiresIn: 120,
  clipboardEnabled: true,
  fileTransferEnabled: true,
  statusMessage: 'Aguardando serviço local prysmview-svc...',
  pendingConsent: null
};

let consentTimerInterval = null;
let codeTimerInterval = null;

// =============================================================================
// COMUNICAÇÃO IPC NATIVA COM O BACKEND RUST
// =============================================================================
function sendIpcMessage(action, data = {}) {
  const payload = JSON.stringify({ action, ...data });
  if (window.ipc && window.ipc.postMessage) {
    window.ipc.postMessage(payload);
  } else {
    console.log('[IPC Debug]:', payload);
  }
}

// =============================================================================
// RENDERIZAÇÃO DA INTERFACE
// =============================================================================
function renderState() {
  // 1. Atualizar ID do PDV
  const elPdvId = document.getElementById('displayPdvId');
  if (elPdvId) {
    elPdvId.textContent = hostState.pdvId || 'PDV-CAIXA-01';
  }

  // 2. Banner de Status da Conexão com o Serviço
  const banner = document.getElementById('connectionStatusBanner');
  const statusText = document.getElementById('statusText');
  if (banner && statusText) {
    if (hostState.isConnected) {
      banner.className = 'status-banner status-online';
      statusText.textContent = 'Pronto para conectar (Conexão Segura E2E)';
    } else {
      banner.className = 'status-banner status-waiting';
      statusText.textContent = hostState.statusMessage || 'Aguardando serviço local prysmview-svc...';
    }
  }

  // 3. Código de Sessão / Senha Temporária
  const codeBox = document.getElementById('displayCodeContainer');
  const codeText = document.getElementById('displaySessionCode');
  const codeExpiryNote = document.getElementById('codeExpiryNote');
  const codeExpiryCountdown = document.getElementById('codeExpiryCountdown');

  if (codeBox && codeText) {
    if (hostState.currentCode) {
      codeBox.classList.add('active-code');
      codeText.textContent = hostState.currentCode;
      if (codeExpiryNote && codeExpiryCountdown) {
        codeExpiryNote.style.display = 'flex';
        codeExpiryCountdown.textContent = `Expira em ${hostState.sessionCodeExpiresIn}s • Máximo 3 tentativas`;
      }
    } else {
      codeBox.classList.remove('active-code');
      codeText.textContent = 'Aguardando chamada...';
      if (codeExpiryNote) {
        codeExpiryNote.style.display = 'none';
      }
    }
  }

  // 4. Banner de Sessão Ativa
  const sessionCard = document.getElementById('sessionActiveCard');
  const opName = document.getElementById('activeOperatorName');
  if (sessionCard) {
    if (hostState.isSessionActive) {
      sessionCard.style.display = 'flex';
      if (opName) {
        opName.textContent = hostState.currentOperator || 'Técnico de Suporte';
      }
    } else {
      sessionCard.style.display = 'none';
    }
  }

  // 5. Modal de Consentimento de Acesso
  const consentModal = document.getElementById('consentModalOverlay');
  const consentOp = document.getElementById('consentOperatorName');
  const consentReason = document.getElementById('consentReason');
  const consentTime = document.getElementById('consentTimeRemaining');

  if (consentModal) {
    if (hostState.pendingConsent) {
      consentModal.style.display = 'flex';
      if (consentOp) consentOp.textContent = hostState.pendingConsent.operatorName || 'Central de Suporte';
      if (consentReason) consentReason.textContent = hostState.pendingConsent.reason || 'Assistência Técnica';
      if (consentTime) consentTime.textContent = `${hostState.pendingConsent.timeoutSecs || 60}s`;

      startConsentCountdown(hostState.pendingConsent.timeoutSecs || 60);
    } else {
      consentModal.style.display = 'none';
      clearInterval(consentTimerInterval);
    }
  }

  // 6. Travas de Segurança Local
  const chkClipboard = document.getElementById('chkClipboard');
  const chkFileTransfer = document.getElementById('chkFileTransfer');
  if (chkClipboard) chkClipboard.checked = hostState.clipboardEnabled;
  if (chkFileTransfer) chkFileTransfer.checked = hostState.fileTransferEnabled;
}

// Contagem regressiva do consentimento
function startConsentCountdown(seconds) {
  clearInterval(consentTimerInterval);
  let remaining = seconds;
  const timeEl = document.getElementById('consentTimeRemaining');

  consentTimerInterval = setInterval(() => {
    remaining--;
    if (timeEl) timeEl.textContent = `${remaining}s`;
    if (remaining <= 0) {
      clearInterval(consentTimerInterval);
      sendIpcMessage('consent_response', { accept: false });
      hostState.pendingConsent = null;
      renderState();
    }
  }, 1000);
}

// =============================================================================
// FUNÇÃO EXPOSTA PARA O BACKEND RUST ATUALIZAR O ESTADO
// =============================================================================
window.updateHostState = function(stateObj) {
  if (typeof stateObj === 'string') {
    try {
      stateObj = JSON.parse(stateObj);
    } catch (e) {
      console.error('Falha ao interpretar stateObj do Rust:', e);
      return;
    }
  }

  hostState = { ...hostState, ...stateObj };
  renderState();
};

// =============================================================================
// INICIALIZAÇÃO E EVENTOS DOM
// =============================================================================
document.addEventListener('DOMContentLoaded', () => {
  // Carrega estado inicial caso injetado pelo WebView
  if (window.__INITIAL_HOST_STATE__) {
    window.updateHostState(window.__INITIAL_HOST_STATE__);
  } else {
    renderState();
  }

  // Botão Copiar ID
  const btnCopy = document.getElementById('btnCopyPdvId');
  if (btnCopy) {
    btnCopy.addEventListener('click', () => {
      const textToCopy = hostState.pdvId || 'PDV-CAIXA-01';
      navigator.clipboard.writeText(textToCopy).then(() => {
        btnCopy.classList.add('copied');
        const feedback = document.getElementById('copyFeedbackText');
        if (feedback) feedback.textContent = 'Copiado!';
        setTimeout(() => {
          btnCopy.classList.remove('copied');
          if (feedback) feedback.textContent = 'Copiar';
        }, 2000);
      });
    });
  }

  // Botão Gerar Código Temporário
  const btnGenCode = document.getElementById('btnGenerateCode');
  if (btnGenCode) {
    btnGenCode.addEventListener('click', () => {
      const randomDigits = Math.floor(10000000 + Math.random() * 90000000).toString();
      hostState.currentCode = randomDigits;
      hostState.sessionCodeExpiresIn = 120;
      
      if (codeTimerInterval) clearInterval(codeTimerInterval);
      codeTimerInterval = setInterval(() => {
        if (hostState.sessionCodeExpiresIn > 0) {
          hostState.sessionCodeExpiresIn--;
          const el = document.getElementById('codeExpiryCountdown');
          if (el) el.textContent = `Expira em ${hostState.sessionCodeExpiresIn}s • Máximo 3 tentativas`;
        } else {
          clearInterval(codeTimerInterval);
          hostState.currentCode = null;
          renderState();
        }
      }, 1000);

      renderState();
      sendIpcMessage('session_code_generated', { code: randomDigits, expiresIn: 120 });
    });
  }

  // Botão Copiar Código de Sessão
  const btnCopyCode = document.getElementById('btnCopySessionCode');
  if (btnCopyCode) {
    btnCopyCode.addEventListener('click', () => {
      if (!hostState.currentCode && btnGenCode) {
        btnGenCode.click();
      }
      setTimeout(() => {
        const codeToCopy = hostState.currentCode || '';
        if (codeToCopy) {
          navigator.clipboard.writeText(codeToCopy).then(() => {
            btnCopyCode.classList.add('copied');
            const feedback = document.getElementById('copyCodeFeedbackText');
            if (feedback) feedback.textContent = 'Copiado!';
            setTimeout(() => {
              btnCopyCode.classList.remove('copied');
              if (feedback) feedback.textContent = 'Copiar';
            }, 2000);
          });
        }
      }, 50);
    });
  }

  // Botão Encerrar Sessão Imediatamente
  const btnTerminate = document.getElementById('btnTerminateSession');
  if (btnTerminate) {
    btnTerminate.addEventListener('click', () => {
      sendIpcMessage('terminate_session');
      hostState.isSessionActive = false;
      hostState.currentOperator = null;
      renderState();
    });
  }

  // Botões de Consentimento
  const btnAccept = document.getElementById('btnConsentAccept');
  if (btnAccept) {
    btnAccept.addEventListener('click', () => {
      sendIpcMessage('consent_response', { accept: true });
      hostState.pendingConsent = null;
      renderState();
    });
  }

  const btnReject = document.getElementById('btnConsentReject');
  if (btnReject) {
    btnReject.addEventListener('click', () => {
      sendIpcMessage('consent_response', { accept: false });
      hostState.pendingConsent = null;
      renderState();
    });
  }

  // Travas de Segurança
  const chkClipboard = document.getElementById('chkClipboard');
  if (chkClipboard) {
    chkClipboard.addEventListener('change', (e) => {
      hostState.clipboardEnabled = e.target.checked;
      sendIpcMessage('toggle_clipboard', { enabled: e.target.checked });
    });
  }

  const chkFileTransfer = document.getElementById('chkFileTransfer');
  if (chkFileTransfer) {
    chkFileTransfer.addEventListener('change', (e) => {
      hostState.fileTransferEnabled = e.target.checked;
      sendIpcMessage('toggle_file_transfer', { enabled: e.target.checked });
    });
  }

  // Controles de Janela
  const btnMin = document.getElementById('btnWinMin');
  if (btnMin) {
    btnMin.addEventListener('click', () => {
      sendIpcMessage('window_minimize');
    });
  }

  const btnClose = document.getElementById('btnWinClose');
  if (btnClose) {
    btnClose.addEventListener('click', () => {
      sendIpcMessage('window_close');
    });
  }
});
