// =============================================================================
// PRYSMVIEW HOST - CONTROLADOR JAVASCRIPT DO CLIENTE WEBVIEW2
// =============================================================================

let hostState = {
  isConnected: false,
  pdvId: '100000000',
  formattedId: '100 000 000',
  localUser: null,
  alias: null,
  group: 'Dispositivos Matriz',
  description: null,
  accessPassword: null,
  isPasswordRevealed: false,
  isSessionActive: false,
  currentOperator: null,
  currentCode: null,
  sessionCodeExpiresIn: 120,
  clipboardEnabled: true,
  fileTransferEnabled: true,
  inputBlocked: false,
  statusMessage: 'Aguardando servico local prysmview-svc...',
  pendingConsent: null
};

let consentTimerInterval = null;
let codeTimerInterval = null;

// =============================================================================
// COMUNICACAO IPC NATIVA COM O BACKEND RUST
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
// RENDERIZACAO DA INTERFACE
// =============================================================================
function renderState() {
  // 0. Atualizar Apelido / Operador Local do Dispositivo
  const elLocalUser = document.getElementById('displayLocalUser');
  const elUserAvatar = document.getElementById('displayUserAvatar');
  if (elLocalUser) {
    const displayName = hostState.alias || hostState.localUser || 'Dispositivo Local';
    elLocalUser.textContent = displayName;
    if (elUserAvatar) {
      const initials = displayName
        .split(' ')
        .filter(Boolean)
        .map(n => n[0])
        .slice(0, 2)
        .join('')
        .toUpperCase() || 'OP';
      elUserAvatar.textContent = initials;
    }
  }

  // 1. Atualizar ID Fixo Permanente do PDV (Estilo TeamViewer)
  const elPdvId = document.getElementById('displayPdvId');
  if (elPdvId) {
    elPdvId.textContent = hostState.formattedId || hostState.pdvId || '100 000 000';
  }

  // 2. Banner de Status da Conexao com o Servico
  const banner = document.getElementById('connectionStatusBanner');
  const statusText = document.getElementById('statusText');
  if (banner && statusText) {
    if (hostState.isConnected) {
      banner.className = 'tv-status-indicator status-online';
      statusText.textContent = 'Pronto para conectar (Conexao Segura E2E)';
    } else {
      banner.className = 'tv-status-indicator status-waiting';
      statusText.textContent = hostState.statusMessage || 'Aguardando servico local prysmview-svc...';
    }
  }

  // 3. Codigo de Sessao / Senha Temporaria (8 digitos)
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
        codeExpiryCountdown.textContent = `Expira em ${hostState.sessionCodeExpiresIn}s • Maximo 3 tentativas`;
      }
    } else {
      codeBox.classList.remove('active-code');
      codeBox.style.borderColor = '';
      codeText.textContent = 'Aguardando chamada...';
      if (codeExpiryNote) {
        codeExpiryNote.style.display = 'none';
      }
    }
  }

  // 3.1 Senha Fixa Desassistida (Host)
  const elPassword = document.getElementById('displayAccessPassword');
  const labelToggle = document.getElementById('labelTogglePassword');
  const btnToggle = document.getElementById('btnToggleShowPassword');
  if (elPassword) {
    if (hostState.accessPassword) {
      if (hostState.isPasswordRevealed) {
        elPassword.textContent = hostState.accessPassword;
        if (labelToggle) labelToggle.textContent = '🙈 Ocultar';
      } else {
        elPassword.textContent = '•••••••• (Definida)';
        if (labelToggle) labelToggle.textContent = '👁 Revelar';
      }
      if (btnToggle) btnToggle.style.display = 'inline-flex';
    } else {
      elPassword.textContent = 'Nenhuma senha definida';
      if (btnToggle) btnToggle.style.display = 'none';
    }
  }

  // 4. Banner e Docking Lateral de Sessao Ativa
  const sessionCard = document.getElementById('sessionActiveCard');
  const opName = document.getElementById('activeOperatorName');
  if (sessionCard) {
    if (hostState.isSessionActive) {
      if (sessionCard.style.display !== 'flex') {
        sessionCard.style.display = 'flex';
        if (window.gsap) {
          gsap.fromTo(sessionCard,
            { opacity: 0, y: -10, scale: 0.98 },
            { opacity: 1, y: 0, scale: 1, duration: 0.35, ease: 'power2.out', clearProps: 'transform,opacity' }
          );
        }
      }
      document.body.classList.add('session-docked');
      if (opName) {
        opName.textContent = hostState.currentOperator || 'Tecnico de Suporte';
      }
    } else {
      sessionCard.style.display = 'none';
      document.body.classList.remove('session-docked');
    }
  }

  // 5. Modal de Consentimento de Acesso
  const consentModal = document.getElementById('consentModalOverlay');
  const consentOp = document.getElementById('consentOperatorName');
  const consentReason = document.getElementById('consentReason');
  const consentTime = document.getElementById('consentTimeRemaining');

  if (consentModal) {
    if (hostState.pendingConsent) {
      if (consentModal.style.display !== 'flex') {
        consentModal.style.display = 'flex';
        if (window.gsap) {
          const dialog = consentModal.querySelector('.consent-dialog');
          if (dialog) {
            gsap.fromTo(dialog,
              { opacity: 0, scale: 0.88, y: -15 },
              { opacity: 1, scale: 1, y: 0, duration: 0.35, ease: 'back.out(1.5)', clearProps: 'transform,opacity' }
            );
          }
        }
      }
      if (consentOp) consentOp.textContent = hostState.pendingConsent.operatorName || 'Central de Suporte';
      if (consentReason) consentReason.textContent = hostState.pendingConsent.reason || 'Assistencia Tecnica';
      if (consentTime) consentTime.textContent = `${hostState.pendingConsent.timeoutSecs || 60}s`;

      startConsentCountdown(hostState.pendingConsent.timeoutSecs || 60);
    } else {
      consentModal.style.display = 'none';
      clearInterval(consentTimerInterval);
    }
  }

  // 6. Travas de Seguranca Local
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
// FUNCAO EXPOSTA PARA O BACKEND RUST ATUALIZAR O ESTADO
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
// CONTROLE DE AUTENTICACAO DE TECNICO E PROPRIEDADES DO HOST
// =============================================================================
function openTechAuthModal() {
  const modal = document.getElementById('techAuthModal');
  if (!modal) return;
  const errBox = document.getElementById('techAuthError');
  if (errBox) errBox.style.display = 'none';
  const inputPwd = document.getElementById('inputTechPassword');
  if (inputPwd) inputPwd.value = '';
  modal.style.display = 'flex';
  if (window.gsap) {
    const dialog = modal.querySelector('.consent-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.fromTo(dialog,
        { opacity: 0, scale: 0.92, y: -10 },
        { opacity: 1, scale: 1, y: 0, duration: 0.25, ease: 'back.out(1.5)', clearProps: 'transform,opacity' }
      );
    }
  }
  const inputUser = document.getElementById('inputTechUsername');
  if (inputUser) {
    if (!inputUser.value) inputUser.value = 'admin';
    setTimeout(() => {
      if (inputPwd) inputPwd.focus();
    }, 50);
  }
}

function closeTechAuthModal() {
  const modal = document.getElementById('techAuthModal');
  if (!modal) return;
  if (window.gsap) {
    const dialog = modal.querySelector('.consent-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.to(dialog, {
        opacity: 0,
        scale: 0.95,
        duration: 0.15,
        ease: 'power2.in',
        onComplete: () => {
          modal.style.display = 'none';
          dialog.style.opacity = '';
          dialog.style.transform = '';
        }
      });
      return;
    }
  }
  modal.style.display = 'none';
}

function submitTechAuth(e) {
  if (e) e.preventDefault();
  const inputUser = document.getElementById('inputTechUsername');
  const inputPwd = document.getElementById('inputTechPassword');
  const username = inputUser ? inputUser.value.trim() : '';
  const password = inputPwd ? inputPwd.value.trim() : '';

  if (!username || !password) return;

  const btnSubmit = document.getElementById('btnSubmitTechAuth');
  if (btnSubmit) btnSubmit.disabled = true;

  sendIpcMessage('verify_technician_login', { username, password });
}

window.onTechAuthResult = function(res) {
  const btnSubmit = document.getElementById('btnSubmitTechAuth');
  if (btnSubmit) btnSubmit.disabled = false;

  if (res && res.success) {
    closeTechAuthModal();
    openHostPropsModal();
  } else {
    const errBox = document.getElementById('techAuthError');
    if (errBox) {
      errBox.textContent = (res && res.message) ? res.message : 'Credenciais de tecnico invalidas.';
      errBox.style.display = 'block';
    }
    const inputPwd = document.getElementById('inputTechPassword');
    if (inputPwd) {
      inputPwd.value = '';
      inputPwd.focus();
    }
  }
};

function openHostPropsModal() {
  const modal = document.getElementById('hostPropsModal');
  if (!modal) return;

  const inputId = document.getElementById('inputPropFixedId');
  const inputPwd = document.getElementById('inputPropAccessPassword');
  const inputAlias = document.getElementById('inputPropAlias');
  const inputGroup = document.getElementById('inputPropGroup');
  const inputDesc = document.getElementById('inputPropDescription');

  if (inputId) inputId.value = hostState.formattedId || hostState.pdvId || '100 000 000';
  if (inputPwd) inputPwd.value = hostState.accessPassword || '';
  if (inputAlias) inputAlias.value = hostState.alias || hostState.localUser || '';
  if (inputGroup) inputGroup.value = hostState.group || 'Dispositivos Matriz';
  if (inputDesc) inputDesc.value = hostState.description || '';

  modal.style.display = 'flex';
  if (window.gsap) {
    const dialog = modal.querySelector('.consent-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.fromTo(dialog,
        { opacity: 0, scale: 0.92, y: -10 },
        { opacity: 1, scale: 1, y: 0, duration: 0.25, ease: 'back.out(1.5)', clearProps: 'transform,opacity' }
      );
    }
  }
  if (inputPwd) setTimeout(() => inputPwd.focus(), 50);
}

function closeHostPropsModal() {
  const modal = document.getElementById('hostPropsModal');
  if (!modal) return;
  if (window.gsap) {
    const dialog = modal.querySelector('.consent-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.to(dialog, {
        opacity: 0,
        scale: 0.95,
        duration: 0.15,
        ease: 'power2.in',
        onComplete: () => {
          modal.style.display = 'none';
          dialog.style.opacity = '';
          dialog.style.transform = '';
        }
      });
      return;
    }
  }
  modal.style.display = 'none';
}

function submitHostProps(e) {
  if (e) e.preventDefault();
  const inputPwd = document.getElementById('inputPropAccessPassword');
  const inputAlias = document.getElementById('inputPropAlias');
  const inputGroup = document.getElementById('inputPropGroup');
  const inputDesc = document.getElementById('inputPropDescription');

  const accessPassword = inputPwd ? inputPwd.value.trim() : '';
  const alias = inputAlias ? inputAlias.value.trim() : '';
  const group = inputGroup ? inputGroup.value.trim() : '';
  const description = inputDesc ? inputDesc.value.trim() : '';

  hostState.accessPassword = accessPassword || null;
  if (alias) hostState.alias = alias;
  if (group) hostState.group = group;
  if (description) hostState.description = description;

  sendIpcMessage('save_host_props', {
    accessPassword,
    alias,
    group,
    description
  });

  closeHostPropsModal();
  renderState();
}

window.openTechAuthModal = openTechAuthModal;
window.closeTechAuthModal = closeTechAuthModal;
window.openHostPropsModal = openHostPropsModal;
window.closeHostPropsModal = closeHostPropsModal;

// =============================================================================
// INICIALIZACAO E EVENTOS DOM
// =============================================================================
document.addEventListener('DOMContentLoaded', () => {
  // Carrega estado inicial caso injetado pelo WebView
  if (window.__INITIAL_HOST_STATE__) {
    window.updateHostState(window.__INITIAL_HOST_STATE__);
  }

  renderState();

  // Animacao de entrada fluida com GSAP
  if (window.gsap) {
    gsap.killTweensOf('.tv-banner, .tv-control-card, .tv-footer');
    gsap.fromTo('.tv-banner', { opacity: 0, y: -12 }, { opacity: 1, y: 0, duration: 0.4, ease: 'power2.out', clearProps: 'transform,opacity' });
    gsap.fromTo('.tv-control-card', { opacity: 0, y: 14 }, { opacity: 1, y: 0, duration: 0.45, delay: 0.08, ease: 'power2.out', clearProps: 'transform,opacity' });
    gsap.fromTo('.tv-footer', { opacity: 0, y: 10 }, { opacity: 1, y: 0, duration: 0.4, delay: 0.15, ease: 'power2.out', clearProps: 'transform,opacity' });

    // Micro-interacoes nos botoes
    document.querySelectorAll('.tv-btn-action, .tv-btn-secondary, .btn-consent-accept, .btn-consent-reject, .tv-mini-link-btn, .btn-emergency, .tv-gear-btn, .tv-cancel-btn').forEach(btn => {
      if (btn.dataset.gsapBound) return;
      btn.dataset.gsapBound = 'true';
      btn.addEventListener('mousedown', () => gsap.to(btn, { scale: 0.95, duration: 0.1, ease: 'power1.out' }));
      btn.addEventListener('mouseup', () => gsap.to(btn, { scale: 1, duration: 0.15, ease: 'power1.out' }));
      btn.addEventListener('mouseleave', () => gsap.to(btn, { scale: 1, duration: 0.15, ease: 'power1.out' }));
    });
  }

  // Botao Copiar ID Permanente
  const btnCopy = document.getElementById('btnCopyPdvId');
  if (btnCopy) {
    btnCopy.addEventListener('click', () => {
      const textToCopy = hostState.formattedId || hostState.pdvId || '100000000';
      if (window.gsap) {
        gsap.killTweensOf(btnCopy);
        gsap.fromTo(btnCopy,
          { scale: 0.92 },
          { scale: 1.05, duration: 0.15, yoyo: true, repeat: 1, ease: 'power1.inOut', overwrite: 'auto', clearProps: 'transform' }
        );
      }
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

  // Botao Gerar Codigo Temporario (8 digitos)
  const btnGenCode = document.getElementById('btnGenerateCode');
  if (btnGenCode) {
    btnGenCode.addEventListener('click', () => {
      const randomDigits = Math.floor(10000000 + Math.random() * 90000000).toString();
      hostState.currentCode = randomDigits;
      hostState.sessionCodeExpiresIn = 120;

      if (window.gsap) {
        gsap.killTweensOf('#displayCodeContainer');
        gsap.killTweensOf('#displaySessionCode');
        gsap.fromTo('#displayCodeContainer',
          { scale: 0.92, borderColor: '#00f0ff' },
          { scale: 1, duration: 0.35, ease: 'back.out(2)', clearProps: 'borderColor' }
        );
        gsap.fromTo('#displaySessionCode',
          { opacity: 0, y: -6 },
          { opacity: 1, y: 0, duration: 0.25, ease: 'power2.out', clearProps: 'transform,opacity' }
        );
      }

      if (codeTimerInterval) clearInterval(codeTimerInterval);
      codeTimerInterval = setInterval(() => {
        if (hostState.sessionCodeExpiresIn > 0) {
          hostState.sessionCodeExpiresIn--;
          const el = document.getElementById('codeExpiryCountdown');
          if (el) el.textContent = `Expira em ${hostState.sessionCodeExpiresIn}s • Maximo 3 tentativas`;
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

  // Botao Copiar Codigo de Sessao
  const btnCopyCode = document.getElementById('btnCopySessionCode');
  if (btnCopyCode) {
    btnCopyCode.addEventListener('click', () => {
      if (!hostState.currentCode && btnGenCode) {
        btnGenCode.click();
      }
      if (window.gsap) {
        gsap.killTweensOf(btnCopyCode);
        gsap.fromTo(btnCopyCode,
          { scale: 0.92 },
          { scale: 1.05, duration: 0.15, yoyo: true, repeat: 1, ease: 'power1.inOut', overwrite: 'auto', clearProps: 'transform' }
        );
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

  // Botao Encerrar Sessao Imediatamente (Encerramento de Emergencia com animacao GSAP)
  const btnTerminate = document.getElementById('btnTerminateSession');
  if (btnTerminate) {
    btnTerminate.addEventListener('click', () => {
      if (window.gsap) {
        const sessionCard = document.getElementById('sessionActiveCard');
        if (sessionCard && sessionCard.style.display !== 'none') {
          gsap.killTweensOf(sessionCard);
          gsap.to(sessionCard, {
            opacity: 0,
            scale: 0.94,
            y: -12,
            duration: 0.22,
            ease: 'power2.in',
            onComplete: () => {
              sendIpcMessage('terminate_session');
              hostState.isSessionActive = false;
              hostState.currentOperator = null;
              renderState();
              sessionCard.style.opacity = '';
              sessionCard.style.transform = '';
            }
          });
          return;
        }
      }
      sendIpcMessage('terminate_session');
      hostState.isSessionActive = false;
      hostState.currentOperator = null;
      renderState();
    });
  }

  // Botoes de Consentimento com animacao suave de saida
  function dismissConsentModal(accept) {
    const consentModal = document.getElementById('consentModalOverlay');
    if (window.gsap && consentModal && consentModal.style.display !== 'none') {
      const dialog = consentModal.querySelector('.consent-dialog');
      if (dialog) {
        gsap.killTweensOf(dialog);
        gsap.to(dialog, {
          opacity: 0,
          scale: 0.92,
          y: 8,
          duration: 0.18,
          ease: 'power2.in',
          onComplete: () => {
            sendIpcMessage('consent_response', { accept });
            hostState.pendingConsent = null;
            renderState();
            dialog.style.opacity = '';
            dialog.style.transform = '';
          }
        });
        return;
      }
    }
    sendIpcMessage('consent_response', { accept });
    hostState.pendingConsent = null;
    renderState();
  }

  const btnAccept = document.getElementById('btnConsentAccept');
  if (btnAccept) {
    btnAccept.addEventListener('click', () => dismissConsentModal(true));
  }

  const btnReject = document.getElementById('btnConsentReject');
  if (btnReject) {
    btnReject.addEventListener('click', () => dismissConsentModal(false));
  }

  // Travas de Seguranca
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

  // Eventos de Autenticacao de Tecnico e Propriedades do Host
  const formTech = document.getElementById('formTechAuth');
  if (formTech) {
    formTech.onsubmit = submitTechAuth;
  }

  const btnCancelTech = document.getElementById('btnCancelTechAuth');
  if (btnCancelTech) {
    btnCancelTech.onclick = closeTechAuthModal;
  }

  const formProps = document.getElementById('formHostProps');
  if (formProps) {
    formProps.onsubmit = submitHostProps;
  }

  const btnCancelProps = document.getElementById('btnCancelHostProps');
  if (btnCancelProps) {
    btnCancelProps.onclick = closeHostPropsModal;
  }

  const btnEditUser = document.getElementById('btnEditUser');
  if (btnEditUser) {
    btnEditUser.onclick = openTechAuthModal;
  }

  const btnSettingsGear = document.getElementById('btnSettingsGear');
  if (btnSettingsGear) {
    btnSettingsGear.onclick = openTechAuthModal;
  }

  const btnEditPwd = document.getElementById('btnEditPassword');
  if (btnEditPwd) {
    btnEditPwd.onclick = openTechAuthModal;
  }

  const btnTogglePwd = document.getElementById('btnToggleShowPassword');
  if (btnTogglePwd) {
    btnTogglePwd.onclick = () => {
      if (!hostState.accessPassword) return;
      hostState.isPasswordRevealed = !hostState.isPasswordRevealed;
      renderState();
    };
  }

  // Gera o primeiro codigo de acesso de 8 digitos automaticamente ao inicializar o Host
  setTimeout(() => {
    if (!hostState.currentCode && btnGenCode) {
      btnGenCode.click();
    }
  }, 100);
});
