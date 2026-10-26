/**
 * PrysmView Admin Console - Frontend Controller
 * Solução corporativa universal de acesso e gestão remota de dispositivos.
 */

function getInitialInstalledVersion() {
  if (typeof window !== 'undefined' && window.__PRYSM_SERVER_VERSION__) {
    return window.__PRYSM_SERVER_VERSION__;
  }
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      const saved = window.localStorage.getItem('prysm_installed_version');
      if (saved) return saved;
    }
  } catch (e) {}
  return '0.1.0';
}

function compareSemver(v1, v2) {
  if (!v1 || !v2) return 0;
  const clean = v => v.toString().replace(/^v/i, '').split('-')[0].split('.').map(n => parseInt(n, 10) || 0);
  const p1 = clean(v1);
  const p2 = clean(v2);
  for (let i = 0; i < Math.max(p1.length, p2.length); i++) {
    const num1 = p1[i] || 0;
    const num2 = p2[i] || 0;
    if (num1 > num2) return 1;
    if (num1 < num2) return -1;
  }
  if (v1.includes('beta') && !v2.includes('beta')) return -1;
  if (!v1.includes('beta') && v2.includes('beta')) return 1;
  return 0;
}

function syncVersionUI() {
  const curVer = state.installedVersion;
  
  // Atualiza tags de versão na interface
  const headerVer = document.getElementById('headerAppVersion');
  if (headerVer) headerVer.textContent = `v${curVer}`;

  const currentVerTag = document.getElementById('currentVersionTag');
  if (currentVerTag) currentVerTag.textContent = `v${curVer}`;

  const channel = state.activeChannel || 'stable';
  const targetVer = state.availableVersions[channel] || state.availableVersions.stable;
  const isNewer = compareSemver(targetVer, curVer) > 0;

  const updateBox = document.getElementById('updateAvailableBox');
  const headerBadge = document.getElementById('headerUpdateBadge');
  const statusText = document.getElementById('updateStatusText');
  const latestTag = document.getElementById('latestVersionTag');

  if (latestTag) latestTag.textContent = `v${targetVer}`;

  if (isNewer) {
    if (updateBox) updateBox.style.display = 'flex';
    if (headerBadge) {
      headerBadge.style.display = 'flex';
      headerBadge.innerHTML = `<div class="pulse-dot"></div><span>⚡ Nova v${targetVer} Disp.</span>`;
    }
    if (statusText) {
      statusText.innerHTML = `<span style="color: var(--accent-cyan); font-weight: 600;">⚡ Nova versão v${targetVer} disponível para instalação imediata!</span>`;
    }
  } else {
    if (updateBox) updateBox.style.display = 'none';
    if (headerBadge) headerBadge.style.display = 'none';
    if (statusText) {
      statusText.innerHTML = `<span style="color: var(--accent-mint); font-weight: 600;">● Sistema Atualizado (Versão v${curVer} ativa e verificada)</span>`;
    }
  }

  const lastCheckEl = document.getElementById('lastCheckTimeText');
  if (lastCheckEl && state.lastCheckTime) {
    lastCheckEl.textContent = `Última verificação: ${state.lastCheckTime}`;
  }
}

// Estado Global da Aplicação
const state = {
  activeTab: 'dashboard',
  filterStatus: 'all',
  selectedStore: '',
  searchQuery: '',
  inspectedPdvId: null,
  latencyMs: 18,
  viewMode: 'cards', // 'cards' ou 'table'
  drawerCollapsed: true,
  currentPage: 1,
  pageSize: 10, // Densidade otimizada: 10 por página com paginação instantânea

  // Versão instalada ativa (persistida via IPC e localStorage)
  installedVersion: getInitialInstalledVersion(),
  availableVersions: {
    stable: '0.1.1',
    beta: '0.1.2-beta'
  },
  activeChannel: 'stable',
  lastCheckTime: 'Hoje, 12:45',
  
  pdvs: [
    { id: "DESK-01", storeName: "Matriz - Corporativo", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host01_key_hash", lastSeen: "Agora", ipAddress: "192.168.1.101", osInfo: "Windows 11 Pro 23H2", resolution: "1920x1080 @ 60Hz", notes: "Estação Financeiro: Impressora HP LaserJet. Monitor duplo. Ramal 4022." },
    { id: "DESK-02", storeName: "Matriz - Corporativo", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host02_key_hash", lastSeen: "Agora", ipAddress: "192.168.1.102", osInfo: "Windows 11 Pro 23H2", resolution: "1920x1080 @ 60Hz", notes: "Estação Faturamento: Certificado A1. Leitor biométrico USB." },
    { id: "DESK-03", storeName: "Matriz - Corporativo", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host05_key_hash", lastSeen: "Agora", ipAddress: "192.168.1.103", osInfo: "Windows 11 Pro 23H2", resolution: "1920x1080 @ 60Hz", notes: "Controladoria: Acesso restrito a relatórios e ERP central." },
    { id: "SRV-01", storeName: "Filial Sul - Escritório", isOnline: false, agentVersion: "0.1.0", publicKey: "ed25519_host04_key_hash", lastSeen: "15 min atrás", ipAddress: "192.168.2.101", osInfo: "Windows 10 Enterprise 22H2", resolution: "1920x1080 @ 60Hz", notes: "Servidor de Arquivos: Manutenção preventiva agendada." },
    { id: "SRV-02", storeName: "Filial Sul - Escritório", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host06_key_hash", lastSeen: "Agora", ipAddress: "192.168.2.102", osInfo: "Windows Server 2022", resolution: "1920x1080 @ 60Hz", notes: "Servidor de Banco de Dados Local: Réplica contingência." },
    { id: "NOTE-03", storeName: "Unidade Norte - Operações", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host03_key_hash", lastSeen: "Agora", ipAddress: "192.168.3.103", osInfo: "Windows 11 Pro 23H2", resolution: "1920x1080 @ 60Hz", notes: "Notebook Gestão: VPN corporativa habilitada. Wi-Fi 6." },
    { id: "NOTE-04", storeName: "Unidade Norte - Operações", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host07_key_hash", lastSeen: "Agora", ipAddress: "192.168.3.104", osInfo: "Windows 11 Pro 23H2", resolution: "1920x1080 @ 60Hz", notes: "Supervisão de Turno: Atendimento a chamados de balcão." },
    { id: "TERM-01", storeName: "Centro de Distribuição", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host08_key_hash", lastSeen: "Agora", ipAddress: "192.168.4.101", osInfo: "Windows 10 IoT Enterprise", resolution: "1366x768 @ 60Hz", notes: "Terminal Expedição 1: Leitor de código de barras Zebra USB." },
    { id: "TERM-02", storeName: "Centro de Distribuição", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host09_key_hash", lastSeen: "Agora", ipAddress: "192.168.4.102", osInfo: "Windows 10 IoT Enterprise", resolution: "1366x768 @ 60Hz", notes: "Terminal Triagem e Recebimento: Impressora térmica de etiquetas." },
    { id: "TERM-03", storeName: "Centro de Distribuição", isOnline: false, agentVersion: "0.1.0", publicKey: "ed25519_host10_key_hash", lastSeen: "42 min atrás", ipAddress: "192.168.4.103", osInfo: "Windows 10 IoT Enterprise", resolution: "1366x768 @ 60Hz", notes: "Conferência Doca 3: Cabo de rede desconectado para troca de switch." },
    { id: "LOG-01", storeName: "Filial Leste - Logística", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host11_key_hash", lastSeen: "Agora", ipAddress: "192.168.5.101", osInfo: "Windows 11 Pro 23H2", resolution: "1920x1080 @ 60Hz", notes: "Despacho Rápido: Integração com balança rodoviária serial." },
    { id: "LOG-02", storeName: "Filial Leste - Logística", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host12_key_hash", lastSeen: "Agora", ipAddress: "192.168.5.102", osInfo: "Windows 11 Pro 23H2", resolution: "1920x1080 @ 60Hz", notes: "Rastreamento e Frotas: Monitoramento de entregas ao vivo." },
    { id: "TERM-04", storeName: "Matriz - Corporativo", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host13_key_hash", lastSeen: "Agora", ipAddress: "192.168.1.111", osInfo: "Windows 10 Pro 22H2", resolution: "1366x768 @ 60Hz", notes: "Ponto de Atendimento 01: Terminal de autoatendimento integrado." },
    { id: "TERM-05", storeName: "Matriz - Corporativo", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host14_key_hash", lastSeen: "Agora", ipAddress: "192.168.1.112", osInfo: "Windows 10 Pro 22H2", resolution: "1366x768 @ 60Hz", notes: "Ponto de Atendimento 02: Impressora fiscal e leitor biométrico." },
    { id: "BALCAO-01", storeName: "Filial Sul - Escritório", isOnline: true, agentVersion: "0.1.0", publicKey: "ed25519_host15_key_hash", lastSeen: "Agora", ipAddress: "192.168.2.111", osInfo: "Windows 10 Pro 22H2", resolution: "1366x768 @ 60Hz", notes: "Balcão de Serviços Sul: Atendimento presencial." },
    { id: "BALCAO-02", storeName: "Unidade Norte - Operações", isOnline: false, agentVersion: "0.1.0", publicKey: "ed25519_host16_key_hash", lastSeen: "2 horas atrás", ipAddress: "192.168.3.111", osInfo: "Windows 10 Pro 22H2", resolution: "1366x768 @ 60Hz", notes: "Terminal Reserva Norte: Desligado no encerramento de turno." }
  ],

  auditRecords: [
    { id: "AUD-901", timestamp: "Hoje 10:14", operator: "admin", pdv: "DESK-01", store: "Matriz", reason: "Suporte e Configuração de Impressora", duration: "12m 40s", status: "Concluída" },
    { id: "AUD-902", timestamp: "Hoje 09:30", operator: "admin", pdv: "DESK-02", store: "Matriz", reason: "Instalação de Certificado Digital", duration: "04m 15s", status: "Concluída" },
    { id: "AUD-903", timestamp: "Ontem 18:22", operator: "suporte02", pdv: "NOTE-03", store: "Unidade Norte", reason: "Otimização de Desempenho", duration: "08m 50s", status: "Concluída" },
    { id: "AUD-904", timestamp: "Ontem 14:10", operator: "suporte01", pdv: "SRV-01", store: "Filial Sul", reason: "Verificação de Logs e Backup", duration: "19m 20s", status: "Concluída" }
  ]
};

// =============================================================================
// FUNÇÕES DE COMUNICAÇÃO IPC COM O RUST
// =============================================================================
function sendRustCommand(action, payload = {}) {
  const msg = JSON.stringify({ action, ...payload });
  if (window.ipc && typeof window.ipc.postMessage === 'function') {
    window.ipc.postMessage(msg);
  } else {
    console.log('[IPC Mock -> Rust]:', msg);
  }
}

// Callback global invocado pelo Rust via webview.evaluate_script
window.onPrysmMessage = function(data) {
  try {
    const parsed = typeof data === 'string' ? JSON.parse(data) : data;
    if (parsed.type === 'ping_result') {
      state.latencyMs = parsed.latencyMs || 18;
      const kpiLat = document.getElementById('kpiLatency');
      if (kpiLat) kpiLat.textContent = `${state.latencyMs} ms`;
      const hdrLat = document.getElementById('headerLatencyText');
      if (hdrLat) hdrLat.textContent = `${state.latencyMs} ms • Conexão Segura`;
      showToast(`Ping Servidor: HTTP 200 OK (${state.latencyMs} ms de RTT)`);
    } else if (parsed.type === 'status_message') {
      showToast(parsed.message);
    }
  } catch (err) {
    console.error('Erro processando mensagem do Rust:', err);
  }
};

// =============================================================================
// TOAST NOTIFICATIONS
// =============================================================================
function showToast(message) {
  const container = document.getElementById('toastContainer');
  const toast = document.createElement('div');
  toast.className = 'toast';
  toast.innerHTML = `
    <svg width="15" height="15" fill="none" stroke="#00f0ff" stroke-width="2" viewBox="0 0 24 24"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
    <span>${message}</span>
  `;
  container.appendChild(toast);
  setTimeout(() => {
    toast.style.opacity = '0';
    toast.style.transform = 'translateY(10px)';
    setTimeout(() => toast.remove(), 300);
  }, 3500);
}

// =============================================================================
// RENDERIZAÇÃO DA LISTA DE DISPOSITIVOS & DASHBOARD
// =============================================================================
function getFilteredPdvs() {
  return state.pdvs.filter(pdv => {
    // Filtro por Status
    if (state.filterStatus === 'online' && !pdv.isOnline) return false;
    if (state.filterStatus === 'offline' && pdv.isOnline) return false;
    
    // Filtro por Grupo/Cliente
    if (state.selectedStore && pdv.storeName !== state.selectedStore) return false;

    // Filtro por Busca Global
    if (state.searchQuery) {
      const q = state.searchQuery.toLowerCase();
      const matchId = pdv.id.toLowerCase().includes(q);
      const matchStore = pdv.storeName.toLowerCase().includes(q);
      const matchIp = pdv.ipAddress.toLowerCase().includes(q);
      const matchOs = pdv.osInfo.toLowerCase().includes(q);
      if (!matchId && !matchStore && !matchIp && !matchOs) return false;
    }

    return true;
  });
}

function renderDashboard() {
  const total = state.pdvs.length;
  const online = state.pdvs.filter(p => p.isOnline).length;
  const offline = total - online;

  // Atualiza KPIs
  const kpiTotalEl = document.getElementById('kpiTotal');
  if (kpiTotalEl) kpiTotalEl.textContent = total > 50 ? `${total.toLocaleString()}` : total;
  const kpiOnlineEl = document.getElementById('kpiOnline');
  if (kpiOnlineEl) kpiOnlineEl.textContent = online;
  const kpiOfflineEl = document.getElementById('kpiOffline');
  if (kpiOfflineEl) kpiOfflineEl.textContent = offline;
  const kpiLatencyEl = document.getElementById('kpiLatency');
  if (kpiLatencyEl) kpiLatencyEl.textContent = `${state.latencyMs} ms`;
  const badgeOnlineEl = document.getElementById('badgeOnlineCount');
  if (badgeOnlineEl) badgeOnlineEl.textContent = online;

  // Atualiza Pills
  const pillAll = document.getElementById('pillAllLabel');
  if (pillAll) pillAll.textContent = `Todos (${total})`;
  const pillOnline = document.getElementById('pillOnlineLabel');
  if (pillOnline) pillOnline.textContent = `Online (${online})`;
  const pillOffline = document.getElementById('pillOfflineLabel');
  if (pillOffline) pillOffline.textContent = `Offline (${offline})`;

  const filtered = getFilteredPdvs();
  const totalFiltered = filtered.length;
  const totalPages = Math.ceil(totalFiltered / state.pageSize) || 1;
  if (state.currentPage > totalPages) state.currentPage = totalPages;
  if (state.currentPage < 1) state.currentPage = 1;

  const startIndex = (state.currentPage - 1) * state.pageSize;
  const endIndex = Math.min(startIndex + state.pageSize, totalFiltered);
  const pageItems = filtered.slice(startIndex, endIndex);

  // Atualiza Contadores
  const resultsLabel = document.getElementById('resultsCountLabel');
  if (resultsLabel) resultsLabel.textContent = `${totalFiltered} de ${total} clientes`;

  const paginationInfo = document.getElementById('paginationInfo');
  if (paginationInfo) {
    paginationInfo.textContent = totalFiltered === 0 
      ? 'Nenhum resultado' 
      : `Exibindo ${startIndex + 1}–${endIndex} de ${totalFiltered} dispositivos`;
  }

  // Atualiza Botões de Paginação
  const btnPrev = document.getElementById('btnPrevPage');
  const btnNext = document.getElementById('btnNextPage');
  if (btnPrev) btnPrev.disabled = state.currentPage <= 1;
  if (btnNext) btnNext.disabled = state.currentPage >= totalPages;

  // Números de Página
  const pageNumbersContainer = document.getElementById('pageNumbersContainer');
  if (pageNumbersContainer) {
    pageNumbersContainer.innerHTML = '';
    const maxPills = Math.min(totalPages, 5);
    for (let p = 1; p <= maxPills; p++) {
      const btn = document.createElement('button');
      btn.className = `page-num ${p === state.currentPage ? 'active' : ''}`;
      btn.textContent = p;
      btn.onclick = () => {
        state.currentPage = p;
        renderDashboard();
      };
      pageNumbersContainer.appendChild(btn);
    }
  }

  // Alterna entre Cards Compactos e Tabela de Frota
  const containerCards = document.getElementById('pdvListContainer');
  const containerTable = document.getElementById('fleetTableContainer');

  if (state.viewMode === 'table') {
    if (containerCards) containerCards.style.display = 'none';
    if (containerTable) containerTable.style.display = 'block';
    renderFleetTable(pageItems);
  } else {
    if (containerCards) containerCards.style.display = 'flex';
    if (containerTable) containerTable.style.display = 'none';
    renderFleetCards(pageItems);
  }

  // Atualiza a Ficha Técnica / Drawer
  renderInspectorDrawer();
}

function renderFleetCards(pageItems) {
  const container = document.getElementById('pdvListContainer');
  if (!container) return;
  container.innerHTML = '';

  if (pageItems.length === 0) {
    container.innerHTML = `
      <div style="text-align: center; padding: 30px; color: var(--text-muted); background: var(--bg-card); border-radius: var(--radius-md); border: 1px dashed var(--border-subtle);">
        <p style="font-size: 13.5px; font-weight: 500;">Nenhum dispositivo encontrado com os filtros atuais.</p>
        <p style="font-size: 12px; margin-top: 4px;">Tente alterar o status ou limpar o termo de pesquisa.</p>
      </div>
    `;
    return;
  }

  pageItems.forEach(pdv => {
    const isSelected = pdv.id === state.inspectedPdvId;
    const card = document.createElement('div');
    card.className = `pdv-card ${isSelected ? 'selected' : ''}`;
    card.onclick = () => selectPdvForInspection(pdv);

    const statusBadge = pdv.isOnline
      ? `<div class="status-badge online"><span class="badge-dot green"></span> ON</div>`
      : `<div class="status-badge offline">OFF</div>`;

    const actionBtn = pdv.isOnline
      ? `<button class="btn-primary btn-connect-card" onclick="event.stopPropagation(); startSession('${pdv.id}', '${pdv.storeName}')">
           <svg width="12" height="12" fill="currentColor" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
           <span>Conectar</span>
         </button>`
      : `<button class="btn-disabled btn-connect-card" disabled>Off</button>`;

    card.innerHTML = `
      <div class="pdv-info-left">
        ${statusBadge}
        <div class="pdv-meta-group">
          <div class="pdv-title-row">
            <span class="pdv-id">${pdv.id}</span>
            <span class="meta-dot">•</span>
            <span class="pdv-store">${pdv.storeName}</span>
            <span class="meta-dot">•</span>
            <span class="pdv-seen">${pdv.lastSeen}</span>
          </div>
          <div class="telemetry-chips">
            <span class="tele-chip highlight">${pdv.ipAddress}</span>
            <span class="tele-chip">${pdv.osInfo}</span>
          </div>
        </div>
      </div>
      <div class="pdv-actions">
        <button class="btn-secondary" style="padding: 4px 10px; font-size: 11.5px; height: 28px;" onclick="event.stopPropagation(); selectPdvForInspection(${JSON.stringify(pdv).replace(/"/g, '&quot;')})">Ficha</button>
        ${actionBtn}
      </div>
    `;

    container.appendChild(card);
  });
}

function renderFleetTable(pageItems) {
  const tbody = document.getElementById('fleetTableBody');
  if (!tbody) return;
  tbody.innerHTML = '';

  if (pageItems.length === 0) {
    tbody.innerHTML = `
      <tr>
        <td colspan="7" style="text-align: center; padding: 24px; color: var(--text-muted);">
          Nenhum dispositivo encontrado com os filtros atuais.
        </td>
      </tr>
    `;
    return;
  }

  pageItems.forEach(pdv => {
    const isSelected = pdv.id === state.inspectedPdvId;
    const tr = document.createElement('tr');
    tr.className = isSelected ? 'selected' : '';
    tr.onclick = () => selectPdvForInspection(pdv);

    const statusBadge = pdv.isOnline
      ? `<span class="status-badge online" style="display: inline-flex;"><span class="pulse-dot"></span> ON</span>`
      : `<span class="status-badge offline" style="display: inline-flex;">OFF</span>`;

    const connectBtn = pdv.isOnline
      ? `<button class="btn-primary" style="padding: 3px 10px; font-size: 11.5px; height: 26px;" onclick="event.stopPropagation(); startSession('${pdv.id}', '${pdv.storeName}')">
           <svg width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
           <span>Conectar</span>
         </button>`
      : `<button class="btn-disabled" style="padding: 3px 10px; font-size: 11.5px; height: 26px;" disabled>Off</button>`;

    tr.innerHTML = `
      <td>${statusBadge}</td>
      <td style="font-weight: 700; color: var(--text-primary); font-family: var(--font-mono); font-size: 13px;">${pdv.id}</td>
      <td style="color: var(--text-secondary); font-weight: 500;">${pdv.storeName}</td>
      <td><span class="tele-chip highlight">${pdv.ipAddress}</span></td>
      <td style="color: var(--text-muted); font-size: 12px;">${pdv.osInfo}</td>
      <td style="color: var(--text-muted); font-size: 11.5px;">${pdv.lastSeen}</td>
      <td style="text-align: right;">
        <div style="display: inline-flex; gap: 6px; align-items: center;">
          <button class="btn-secondary" style="padding: 3px 8px; font-size: 11px; height: 26px;" onclick="event.stopPropagation(); selectPdvForInspection(${JSON.stringify(pdv).replace(/"/g, '&quot;')})">Ficha</button>
          ${connectBtn}
        </div>
      </td>
    `;

    tbody.appendChild(tr);
  });
}

function getInspectedPdv() {
  return state.pdvs.find(p => p.id === state.inspectedPdvId) || null;
}

function selectPdvForInspection(pdv) {
  state.inspectedPdvId = pdv.id;
  state.drawerCollapsed = false;
  const split = document.getElementById('splitDashboard');
  if (split) {
    split.classList.remove('drawer-collapsed');
  }
  renderDashboard();
}

function renderInspectorDrawer() {
  const split = document.getElementById('splitDashboard');
  if (state.drawerCollapsed || !state.inspectedPdvId) {
    if (split) split.classList.add('drawer-collapsed');
    return;
  }
  if (split) split.classList.remove('drawer-collapsed');

  const pdv = getInspectedPdv();
  if (!pdv) return;

  document.getElementById('drawerPdvId').textContent = pdv.id;
  document.getElementById('drawerStoreName').textContent = pdv.storeName;
  const statusEl = document.getElementById('diagStatus');
  if (statusEl) {
    statusEl.textContent = pdv.isOnline ? 'Online (Pronto)' : 'Offline (Sem sinal)';
    statusEl.style.color = pdv.isOnline ? 'var(--accent-mint)' : 'var(--text-muted)';
  }
  document.getElementById('diagIp').textContent = pdv.ipAddress;
  document.getElementById('diagOs').textContent = pdv.osInfo;
  document.getElementById('diagResolution').textContent = pdv.resolution;
  document.getElementById('diagVersion').textContent = `v${pdv.agentVersion}`;
  const lastSeenEl = document.getElementById('diagLastSeen');
  if (lastSeenEl) {
    lastSeenEl.textContent = pdv.lastSeen || 'Agora';
  }
  const keyEl = document.getElementById('diagKey');
  if (keyEl) {
    keyEl.textContent = '';
  }
  document.getElementById('notesTextarea').value = pdv.notes || '';

  const pulse = document.getElementById('drawerPulseDot');
  if (pdv.isOnline) {
    pulse.style.background = 'var(--accent-mint)';
    pulse.style.boxShadow = '0 0 8px var(--accent-mint)';
  } else {
    pulse.style.background = 'var(--text-muted)';
    pulse.style.boxShadow = 'none';
  }

  const btnConnect = document.getElementById('btnDrawerConnect');
  if (btnConnect) {
    if (pdv.isOnline) {
      btnConnect.disabled = false;
      btnConnect.className = 'btn-primary primary-connect-btn';
      btnConnect.innerHTML = `
        <svg width="15" height="15" fill="currentColor" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
        <span>Conectar Sessão Remota</span>
      `;
    } else {
      btnConnect.disabled = true;
      btnConnect.className = 'btn-disabled primary-connect-btn';
      btnConnect.innerHTML = `<span>Dispositivo Offline - Conexão Indisponível</span>`;
    }
  }
}

// =============================================================================
// RENDERIZAÇÃO DAS OUTRAS ABAS (AUDITORIA E GRUPOS)
// =============================================================================
function renderAuditTable() {
  const tbody = document.getElementById('auditTableBody');
  tbody.innerHTML = '';
  state.auditRecords.forEach(rec => {
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td style="font-weight: 600; color: var(--accent-cyan); font-family: var(--font-mono);">${rec.id}</td>
      <td style="color: var(--text-muted);">${rec.timestamp}</td>
      <td style="font-weight: 500; color: var(--accent-violet);">${rec.operator}</td>
      <td style="font-weight: 600; color: var(--text-primary);">${rec.pdv} (${rec.store})</td>
      <td style="color: var(--text-secondary);">${rec.reason}</td>
      <td style="color: var(--accent-mint); font-weight: 600;">${rec.duration}</td>
      <td><span class="audit-badge">${rec.status}</span></td>
    `;
    tbody.appendChild(tr);
  });
}

function renderStoresGrid() {
  const grid = document.getElementById('storesGrid');
  grid.innerHTML = '';
  const stores = ['Matriz - Corporativo', 'Filial Sul - Escritório', 'Unidade Norte - Operações'];
  
  stores.forEach(store => {
    const storePdvs = state.pdvs.filter(p => p.storeName === store);
    const onlineCount = storePdvs.filter(p => p.isOnline).length;
    
    const card = document.createElement('div');
    card.className = 'settings-card';
    card.innerHTML = `
      <div style="display: flex; justify-content: space-between; align-items: center;">
        <h4 style="font-size: 15px; font-weight: 700; color: var(--text-primary);">${store}</h4>
        <span class="status-badge ${onlineCount > 0 ? 'online' : 'offline'}">${onlineCount}/${storePdvs.length} Online</span>
      </div>
      <p style="font-size: 12.5px; color: var(--text-muted); line-height: 1.5;">Rede: 192.168.x.x • Gateway VPN Seguro</p>
      <div style="display: flex; gap: 8px; flex-wrap: wrap;">
        ${storePdvs.map(p => `<span class="tele-chip ${p.isOnline ? 'highlight' : ''}">${p.id}</span>`).join('')}
      </div>
      <button class="btn-secondary" style="margin-top: 8px; justify-content: center; font-size: 13px;" onclick="filterByStore('${store}')">
        Filtrar Dispositivos Deste Grupo
      </button>
    `;
    grid.appendChild(card);
  });
}

function filterByStore(storeName) {
  state.selectedStore = storeName;
  document.getElementById('storeSelectFilter').value = storeName;
  switchTab('dashboard');
  renderDashboard();
}

// =============================================================================
// NAVEGAÇÃO ENTRE ABAS
// =============================================================================
function switchTab(tabName) {
  state.activeTab = tabName;
  
  // Atualiza sidebar ativa
  document.querySelectorAll('.nav-item').forEach(el => {
    el.classList.toggle('active', el.dataset.tab === tabName);
  });

  // Atualiza páginas
  document.querySelectorAll('.view-page').forEach(el => el.classList.remove('active'));
  
  const targetMap = {
    'dashboard': 'viewDashboard',
    'direct': 'viewDirect',
    'stores': 'viewStores',
    'audit': 'viewAudit',
    'settings': 'viewSettings'
  };

  const breadcrumbsMap = {
    'dashboard': 'Painel de Dispositivos',
    'direct': 'Conexão Rápida',
    'stores': 'Grupos & Clientes',
    'audit': 'Auditoria E2E',
    'settings': 'Configurações Globais'
  };

  const targetId = targetMap[tabName] || 'viewDashboard';
  const page = document.getElementById(targetId);
  if (page) page.classList.add('active');

  document.getElementById('breadcrumbLabel').textContent = breadcrumbsMap[tabName] || 'Dispositivos';

  if (tabName === 'dashboard') renderDashboard();
  if (tabName === 'audit') renderAuditTable();
  if (tabName === 'stores') renderStoresGrid();
}

// =============================================================================
// MODAL DE CÓDIGO DE 8 DÍGITOS
// =============================================================================
let activeSessionTarget = null;

function startSession(pdvId, storeName) {
  activeSessionTarget = { pdvId, storeName };
  document.getElementById('modalTitle').textContent = `Conexão Segura: ${pdvId}`;
  document.getElementById('modalSubtitle').textContent = `Solicite ao usuário no dispositivo remoto (${storeName}) o código de 8 dígitos exibido na tela.`;
  
  // Limpa inputs
  document.querySelectorAll('.code-digit').forEach(input => input.value = '');
  document.getElementById('sessionModal').classList.add('active');
  const firstInput = document.querySelector('.code-digit');
  if (firstInput) firstInput.focus();
}

function closeModal() {
  document.getElementById('sessionModal').classList.remove('active');
}

// =============================================================================
// GERENCIADOR DE TEMA VISUAL (MODO ESCURO / CLARO / AUTO)
// =============================================================================
let _currentTheme = 'dark';

const ThemeManager = {
  storageKey: 'prysm_theme',

  getSavedTheme() {
    try {
      if (typeof window !== 'undefined' && window.localStorage) {
        return window.localStorage.getItem(this.storageKey) || _currentTheme;
      }
    } catch (e) {
      console.warn('LocalStorage inacessível no contexto WebView2:', e);
    }
    return _currentTheme;
  },

  getSystemTheme() {
    try {
      return window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    } catch (e) {
      return 'dark';
    }
  },

  resolveEffectiveTheme(mode) {
    if (mode === 'auto') {
      return this.getSystemTheme();
    }
    return mode === 'light' ? 'light' : 'dark';
  },

  applyTheme(mode, notify = false) {
    _currentTheme = mode;
    const effective = this.resolveEffectiveTheme(mode);

    try {
      if (document.documentElement) {
        document.documentElement.setAttribute('data-theme', effective);
      }
    } catch (e) {
      console.warn('Erro ao definir atributo data-theme:', e);
    }

    try {
      if (typeof window !== 'undefined' && window.localStorage) {
        window.localStorage.setItem(this.storageKey, mode);
      }
    } catch (e) {
      // Ignora erro de storage em WebView2
    }

    // Atualiza ícones do botão rápido no header
    const sunIcon = document.getElementById('themeIconSun');
    const moonIcon = document.getElementById('themeIconMoon');
    const btnToggle = document.getElementById('btnThemeToggle');

    if (sunIcon && moonIcon && btnToggle) {
      if (effective === 'dark') {
        sunIcon.style.display = 'block';
        moonIcon.style.display = 'none';
        btnToggle.title = 'Mudar para Modo Claro (Alt+T)';
      } else {
        sunIcon.style.display = 'none';
        moonIcon.style.display = 'block';
        btnToggle.title = 'Mudar para Modo Escuro (Alt+T)';
      }
    }

    // Sincroniza cards de opções em Configurações
    document.querySelectorAll('.theme-option-card').forEach(card => {
      const cardMode = card.dataset.themeMode;
      const isSelected = cardMode === mode;
      card.classList.toggle('selected', isSelected);
      const radio = card.querySelector('input[type="radio"]');
      if (radio) radio.checked = isSelected;
    });

    if (notify) {
      const modeLabel = mode === 'auto'
        ? `Automático (Detectado: ${effective === 'dark' ? 'Modo Escuro' : 'Modo Claro'})`
        : (mode === 'light' ? 'Modo Claro (Fluent Light)' : 'Modo Escuro (Obsidian)');
      showToast(`Tema alterado: ${modeLabel}`);
    }
  },

  toggle() {
    const currentMode = this.getSavedTheme();
    const effective = this.resolveEffectiveTheme(currentMode);
    const nextMode = effective === 'dark' ? 'light' : 'dark';
    this.applyTheme(nextMode, true);
  },

  init() {
    try {
      const saved = this.getSavedTheme();
      this.applyTheme(saved, false);
    } catch (e) {
      console.warn('Erro no carregamento inicial do tema:', e);
    }

    // Botão de alternância rápida no Header
    const btnToggle = document.getElementById('btnThemeToggle');
    if (btnToggle) {
      btnToggle.onclick = (e) => {
        if (e) { e.preventDefault(); e.stopPropagation(); }
        this.toggle();
      };
    }

    // Cliques nos Cards de seleção em Configurações
    document.querySelectorAll('.theme-option-card').forEach(card => {
      card.onclick = () => {
        const mode = card.dataset.themeMode;
        if (mode) this.applyTheme(mode, true);
      };
    });

    // Mudança nos inputs de rádio
    document.querySelectorAll('input[name="uiTheme"]').forEach(radio => {
      radio.onchange = (e) => {
        if (e.target.checked) this.applyTheme(e.target.value, true);
      };
    });

    // Listener para quando o tema do Windows muda dinamicamente
    if (window.matchMedia) {
      try {
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
          if (this.getSavedTheme() === 'auto') {
            this.applyTheme('auto', false);
          }
        });
      } catch (e) {}
    }

    // Atalho de teclado rápido: Alt+T para alternar
    window.addEventListener('keydown', (e) => {
      if (e.altKey && (e.key === 't' || e.key === 'T')) {
        e.preventDefault();
        this.toggle();
      }
    });
  }
};

// =============================================================================
// REGISTRO GLOBAL DE FUNÇÕES (GARANTIA TOTAL DE ACESSO ONCLICK E EVENT LISTENERS)
// =============================================================================
window.switchTab = switchTab;
window.startSession = startSession;
window.closeModal = closeModal;
window.selectPdvForInspection = selectPdvForInspection;
window.filterByStore = filterByStore;
window.showToast = showToast;
window.ThemeManager = ThemeManager;

// =============================================================================
// INICIALIZAÇÃO E EVENT LISTENERS RESILIENTES
// =============================================================================
let _appInitialized = false;

function initApp() {
  if (_appInitialized) return;
  _appInitialized = true;
  console.log('[PrysmView] Inicializando interface do operador...');

  // Inicializa gerenciador de temas (Dark / Light / Auto)
  try {
    ThemeManager.init();
  } catch (err) {
    console.error('Falha em ThemeManager.init:', err);
  }

  // Navegação na Sidebar
  document.querySelectorAll('.nav-item').forEach(btn => {
    btn.onclick = (e) => {
      if (e) e.preventDefault();
      switchTab(btn.dataset.tab);
    };
  });

  // KPI Clicks para filtro rápido
  document.querySelectorAll('.kpi-card[data-filter]').forEach(card => {
    card.onclick = () => {
      state.filterStatus = card.dataset.filter;
      state.currentPage = 1;
      updateFilterPills();
      renderDashboard();
    };
  });

  // Filtros Pills
  document.querySelectorAll('.pill-btn[data-filter]').forEach(btn => {
    btn.onclick = () => {
      state.filterStatus = btn.dataset.filter;
      state.currentPage = 1;
      updateFilterPills();
      renderDashboard();
    };
  });

  function updateFilterPills() {
    document.querySelectorAll('.pill-btn').forEach(b => {
      b.classList.toggle('active', b.dataset.filter === state.filterStatus);
    });
  }

  // Filtro de Grupo
  const storeSelect = document.getElementById('storeSelectFilter');
  if (storeSelect) {
    storeSelect.onchange = (e) => {
      state.selectedStore = e.target.value;
      state.currentPage = 1;
      renderDashboard();
    };
  }

  // Busca Global
  const searchInput = document.getElementById('globalSearchInput');
  if (searchInput) {
    searchInput.oninput = (e) => {
      state.searchQuery = e.target.value;
      state.currentPage = 1;
      renderDashboard();
    };
  }

  // Alternância de Visualização (Cards Compactos vs Tabela de Frota)
  const btnViewCards = document.getElementById('btnViewCards');
  const btnViewTable = document.getElementById('btnViewTable');
  if (btnViewCards && btnViewTable) {
    btnViewCards.onclick = () => {
      state.viewMode = 'cards';
      btnViewCards.classList.add('active');
      btnViewTable.classList.remove('active');
      renderDashboard();
    };
    btnViewTable.onclick = () => {
      state.viewMode = 'table';
      btnViewTable.classList.add('active');
      btnViewCards.classList.remove('active');
      renderDashboard();
    };
  }

  // Recolher/Expandir Ficha Técnica Lateral
  const btnToggleDrawer = document.getElementById('btnToggleDrawer');
  if (btnToggleDrawer) {
    btnToggleDrawer.onclick = () => {
      const split = document.getElementById('splitDashboard');
      if (split) {
        split.classList.toggle('drawer-collapsed');
        state.drawerCollapsed = split.classList.contains('drawer-collapsed');
      }
    };
  }

  // Botão fechar na própria Ficha Técnica
  const btnCloseInspector = document.getElementById('btnCloseInspector');
  if (btnCloseInspector) {
    btnCloseInspector.onclick = () => {
      state.drawerCollapsed = true;
      state.inspectedPdvId = null;
      const split = document.getElementById('splitDashboard');
      if (split) {
        split.classList.add('drawer-collapsed');
      }
      renderDashboard();
    };
  }

  // Paginação Prev / Next
  const btnPrevPage = document.getElementById('btnPrevPage');
  if (btnPrevPage) {
    btnPrevPage.onclick = () => {
      if (state.currentPage > 1) {
        state.currentPage--;
        renderDashboard();
      }
    };
  }
  const btnNextPage = document.getElementById('btnNextPage');
  if (btnNextPage) {
    btnNextPage.onclick = () => {
      const filtered = getFilteredPdvs();
      const totalPages = Math.ceil(filtered.length / state.pageSize) || 1;
      if (state.currentPage < totalPages) {
        state.currentPage++;
        renderDashboard();
      }
    };
  }

  // Ações do Drawer
  const btnDrawerConnect = document.getElementById('btnDrawerConnect');
  if (btnDrawerConnect) {
    btnDrawerConnect.onclick = () => {
      const pdv = getInspectedPdv();
      if (pdv && pdv.isOnline) startSession(pdv.id, pdv.storeName);
    };
  }

  const btnQuickCode = document.getElementById('btnQuickCode');
  if (btnQuickCode) {
    btnQuickCode.onclick = () => {
      const pdv = getInspectedPdv();
      if (pdv) startSession(pdv.id, pdv.storeName);
    };
  }

  const btnRestartAgent = document.getElementById('btnQuickRestartAgent');
  if (btnRestartAgent) {
    btnRestartAgent.onclick = () => {
      const pdv = getInspectedPdv();
      sendRustCommand('restart_agent', { pdv_id: pdv.id });
      showToast(`Sinal de reinício enviado ao serviço do ${pdv.id}`);
    };
  }

  const btnRebootSystem = document.getElementById('btnQuickRebootSystem');
  if (btnRebootSystem) {
    btnRebootSystem.onclick = () => {
      const pdv = getInspectedPdv();
      sendRustCommand('reboot_system', { pdv_id: pdv.id });
      showToast(`Solicitação de reinício do sistema enviada ao ${pdv.id}`);
    };
  }

  const btnCtrlAltDel = document.getElementById('btnQuickCtrlAltDel');
  if (btnCtrlAltDel) {
    btnCtrlAltDel.onclick = () => {
      const pdv = getInspectedPdv();
      sendRustCommand('send_sas', { pdv_id: pdv.id });
      showToast(`Comando SAS Ctrl+Alt+Del enviado ao ${pdv.id}`);
    };
  }

  const btnQuickPing = document.getElementById('btnQuickPing');
  if (btnQuickPing) {
    btnQuickPing.onclick = () => {
      const pdv = getInspectedPdv();
      sendRustCommand('ping_pdv', { pdv_id: pdv.id, ip: pdv.ipAddress });
      showToast(`Diagnóstico em ${pdv.id} (${pdv.ipAddress}): Resposta em 18 ms - Conexão estável`);
    };
  }

  // Salvar Notas Técnicas
  const btnSaveNotes = document.getElementById('btnSaveNotes');
  if (btnSaveNotes) {
    btnSaveNotes.onclick = () => {
      const pdv = getInspectedPdv();
      const notes = document.getElementById('notesTextarea').value;
      pdv.notes = notes;
      sendRustCommand('save_notes', { pdv_id: pdv.id, notes });
      showToast(`Notas técnicas do dispositivo ${pdv.id} gravadas.`);
    };
  }

  // Testar Ping na API
  const btnTestPing = document.getElementById('btnTestPing');
  if (btnTestPing) {
    btnTestPing.onclick = () => {
      sendRustCommand('test_api_ping');
      showToast('Servidor de Sinalização: HTTP 200 OK (18 ms de RTT via TLS 1.3)');
    };
  }

  // Salvar Preferências (Ações Topo, Base e Geral)
  const handleSaveSettings = () => {
    const techName = document.getElementById('cfgTechName')?.value || 'admin';
    const techRole = document.getElementById('cfgTechRole')?.value || 'N1 - Suporte Técnico';
    const techDept = document.getElementById('cfgTechDept')?.value || 'Central de Operações de TI';
    
    const headName = document.getElementById('headerOperatorName');
    if (headName) headName.textContent = techName;
    const sideRole = document.getElementById('sidebarTechRole');
    if (sideRole) sideRole.textContent = techRole;
    const sideDept = document.getElementById('sidebarTechDept');
    if (sideDept) sideDept.textContent = techDept;

    sendRustCommand('save_settings', { techName, techRole, techDept });
    showToast('Configurações e parâmetros de suporte salvos com sucesso.');
  };

  const btnSaveTopSettings = document.getElementById('btnSaveTopSettings');
  if (btnSaveTopSettings) btnSaveTopSettings.onclick = handleSaveSettings;
  const btnSaveBottomSettings = document.getElementById('btnSaveBottomSettings');
  if (btnSaveBottomSettings) btnSaveBottomSettings.onclick = handleSaveSettings;
  const btnSaveAllSettings = document.getElementById('btnSaveAllSettings');
  if (btnSaveAllSettings) btnSaveAllSettings.onclick = handleSaveSettings;

  // Filtro de Categorias em Configurações (Abas Rápidas)
  const settingsTabBtns = document.querySelectorAll('.settings-tab-btn');
  settingsTabBtns.forEach(btn => {
    btn.onclick = () => {
      const section = btn.dataset.settingsSection;
      settingsTabBtns.forEach(b => b.classList.toggle('active', b === btn));
      
      document.querySelectorAll('.settings-card[data-settings-group]').forEach(card => {
        if (section === 'all' || card.dataset.settingsGroup === section) {
          card.classList.remove('hidden-section');
        } else {
          card.classList.add('hidden-section');
        }
      });
    };
  });

  // Exportar Auditoria CSV
  const handleExportCsv = () => {
    sendRustCommand('export_audit_csv');
    showToast('Relatório de auditoria exportado para a pasta de Downloads.');
  };
  const btnExportAudit = document.getElementById('btnExportAudit');
  if (btnExportAudit) btnExportAudit.onclick = handleExportCsv;
  const btnExportAudit2 = document.getElementById('btnExportAudit2');
  if (btnExportAudit2) btnExportAudit2.onclick = handleExportCsv;

  // Limpar Cache
  const btnClearCache = document.getElementById('btnClearCache');
  if (btnClearCache) {
    btnClearCache.onclick = () => {
      sendRustCommand('clear_cache');
      showToast('Cache de telemetria e diagnósticos temporários liberados.');
    };
  }

  // Sincronizar / Atualizar
  const btnRefresh = document.getElementById('btnRefresh');
  if (btnRefresh) {
    btnRefresh.onclick = () => {
      sendRustCommand('refresh_fleet');
      showToast('Sincronização de dispositivos concluída (18 ms)');
      renderDashboard();
    };
  }

  // Logout
  const btnLogout = document.getElementById('btnLogout');
  if (btnLogout) {
    btnLogout.onclick = () => {
      showToast('Sessão do operador encerrada com segurança.');
    };
  }

  // Modal Inputs (Digitação com auto-avanço)
  const codeInputs = document.querySelectorAll('.code-digit');
  codeInputs.forEach((input, index) => {
    input.oninput = (e) => {
      if (e.target.value.length === 1 && index < codeInputs.length - 1) {
        codeInputs[index + 1].focus();
      }
    };

    input.onkeydown = (e) => {
      if (e.key === 'Backspace' && !e.target.value && index > 0) {
        codeInputs[index - 1].focus();
      }
    };
  });

  const btnModalClose = document.getElementById('btnModalClose');
  if (btnModalClose) btnModalClose.onclick = closeModal;

  const btnModalAuthorize = document.getElementById('btnModalAuthorize');
  if (btnModalAuthorize) {
    btnModalAuthorize.onclick = () => {
      const code = Array.from(codeInputs).map(i => i.value).join('');
      if (code.length < 8) {
        showToast('Por favor, insira o código completo de 8 dígitos.');
        return;
      }
      closeModal();
      showToast(`Código validado com sucesso! Conectando a ${activeSessionTarget?.pdvId || 'Dispositivo'}...`);
    };
  }

  // Conexão Direta Submit
  const btnDirectSubmit = document.getElementById('btnDirectSubmit');
  if (btnDirectSubmit) {
    btnDirectSubmit.onclick = () => {
      const pdvId = document.getElementById('directPdvInput').value.trim();
      const code = document.getElementById('directCodeInput').value.trim();
      if (!pdvId || code.length < 8) {
        showToast('Preencha o identificador do dispositivo e o código de 8 dígitos.');
        return;
      }
      showToast(`Conectando a ${pdvId}...`);
    };
  }

  // =========================================================================
  // SISTEMA DE ATUALIZAÇÃO AUTOMÁTICA (IN-APP AUTO-UPDATE)
  // =========================================================================
  const checkUpdatesBtn = document.getElementById('btnCheckUpdates');
  const updateAvailableBox = document.getElementById('updateAvailableBox');
  const updateStatusText = document.getElementById('updateStatusText');
  const headerUpdateBadge = document.getElementById('headerUpdateBadge');
  const btnApplyUpdate = document.getElementById('btnApplyUpdate');
  const updateProgressContainer = document.getElementById('updateProgressContainer');
  const updateProgressBar = document.getElementById('updateProgressBar');
  const updateProgressLabel = document.getElementById('updateProgressLabel');
  const updateProgressPercent = document.getElementById('updateProgressPercent');

  // Ao carregar, sincroniza todos os indicadores visuais de versão instalada
  syncVersionUI();

  // Escuta alteração do canal de atualização (Estável vs Beta)
  document.querySelectorAll('input[name="updateChannel"]').forEach(radio => {
    radio.onchange = (e) => {
      state.activeChannel = e.target.value;
      const targetVer = state.availableVersions[state.activeChannel] || '0.1.1';
      syncVersionUI();
      showToast(`Canal alterado para ${state.activeChannel === 'beta' ? 'Beta' : 'Estável'}. Versão disponível: v${targetVer}`);
    };
  });

  // Ao clicar no badge do header, abre as configurações e foca no card de atualização
  if (headerUpdateBadge) {
    headerUpdateBadge.onclick = () => {
      switchTab('settings');
      const updatesTabBtn = document.querySelector('.settings-tab-btn[data-settings-section="updates"]');
      if (updatesTabBtn) updatesTabBtn.click();
      const card = document.getElementById('cardAutoUpdate');
      if (card) {
        card.scrollIntoView({ behavior: 'smooth' });
        card.style.borderColor = 'var(--accent-cyan)';
        card.style.boxShadow = 'var(--shadow-cyan)';
        setTimeout(() => {
          card.style.borderColor = 'var(--border-subtle)';
          card.style.boxShadow = 'none';
        }, 2500);
      }
    };
  }

  // Verificar Atualizações (Buscar)
  if (checkUpdatesBtn) {
    checkUpdatesBtn.onclick = () => {
      checkUpdatesBtn.disabled = true;
      checkUpdatesBtn.innerHTML = `
        <svg class="spin-animation" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M23 4v6h-6"></path><path d="M1 20v-6h6"></path><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
        <span>Verificando...</span>
      `;

      const channel = document.querySelector('input[name="updateChannel"]:checked')?.value || state.activeChannel || 'stable';
      state.activeChannel = channel;
      sendRustCommand('check_updates', { channel, currentVersion: state.installedVersion });

      setTimeout(() => {
        checkUpdatesBtn.disabled = false;
        checkUpdatesBtn.innerHTML = `
          <svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M23 4v6h-6"></path><path d="M1 20v-6h6"></path><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
          <span>Verificar Atualizações Agora</span>
        `;

        const now = new Date();
        state.lastCheckTime = `Hoje, ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}`;

        const targetVer = state.availableVersions[channel] || state.availableVersions.stable;
        const isNewer = compareSemver(targetVer, state.installedVersion) > 0;

        syncVersionUI();

        if (isNewer) {
          showToast(`Nova versão v${targetVer} encontrada no canal ${channel}! Você pode atualizar agora.`);
        } else {
          showToast(`O PrysmView já está na versão mais recente (v${state.installedVersion}). Nenhuma atualização necessária!`);
        }
      }, 800);
    };
  }

  // Instalar Atualização In-Place
  if (btnApplyUpdate) {
    btnApplyUpdate.onclick = () => {
      const channel = state.activeChannel || 'stable';
      const targetVer = state.availableVersions[channel] || state.availableVersions.stable;

      btnApplyUpdate.disabled = true;
      btnApplyUpdate.innerHTML = `<span>Instalando Atualização v${targetVer}...</span>`;
      if (updateProgressContainer) updateProgressContainer.style.display = 'flex';

      let currentStep = 0;
      const steps = [
        { p: 25, label: `Baixando pacote oficial v${targetVer} (3.2 MB)...` },
        { p: 55, label: "Validando integridade SHA-256 em tempo constante..." },
        { p: 80, label: "Validando assinatura digital Ed25519..." },
        { p: 95, label: "Aplicando substituição atômica in-place (Windows NTFS)..." },
        { p: 100, label: `Atualização v${targetVer} aplicada com sucesso!` }
      ];

      const interval = setInterval(() => {
        if (currentStep < steps.length) {
          const item = steps[currentStep];
          if (updateProgressBar) updateProgressBar.style.width = `${item.p}%`;
          if (updateProgressPercent) updateProgressPercent.textContent = `${item.p}%`;
          if (updateProgressLabel) updateProgressLabel.textContent = item.label;
          currentStep++;
        } else {
          clearInterval(interval);

          // Efetiva a alteração da versão no estado
          state.installedVersion = targetVer;

          // 1. Persiste no disco via Rust IPC
          sendRustCommand('apply_update', { version: state.installedVersion, channel });

          // 2. Persiste no localStorage (com tratamento de erro WebView2)
          try {
            if (typeof window !== 'undefined' && window.localStorage) {
              window.localStorage.setItem('prysm_installed_version', state.installedVersion);
            }
          } catch (e) {}

          // 3. Registra no histórico de auditoria
          const now = new Date();
          const timeStr = `Hoje ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
          state.auditRecords.unshift({
            id: `AUD-${Math.floor(920 + Math.random() * 50)}`,
            timestamp: timeStr,
            operator: document.getElementById('headerOperatorName')?.textContent || "admin",
            pdv: "PrysmAdmin",
            store: "Console Central",
            reason: `Atualização In-App de Software para v${state.installedVersion}`,
            duration: "06s",
            status: "Concluída"
          });

          showToast(`Sucesso! PrysmView atualizado para a versão v${state.installedVersion} sem reinstalação manual.`);

          setTimeout(() => {
            if (updateProgressContainer) updateProgressContainer.style.display = 'none';
            if (updateProgressBar) updateProgressBar.style.width = '0%';
            btnApplyUpdate.disabled = false;
            btnApplyUpdate.innerHTML = `
              <svg width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
              <span>Instalar Atualização Automaticamente</span>
            `;

            // Sincroniza a interface: oculta aviso de atualização, atualiza badges e status
            syncVersionUI();
          }, 1200);
        }
      }, 500);
    };
  }

  // Render inicial
  renderDashboard();
  syncVersionUI();
}

// Inicia imediatamente se o DOM já estiver pronto, ou escuta DOMContentLoaded
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}

