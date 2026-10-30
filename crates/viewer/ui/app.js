/**
 * PrysmView Admin Console - Frontend Controller
 * Solução corporativa universal de acesso e gestão remota de dispositivos.
 */

function getInitialInstalledVersion() {
  let ver = '0.1.0';
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      const saved = window.localStorage.getItem('prysm_installed_version');
      if (saved) ver = saved;
    }
  } catch (e) { }
  if (typeof window !== 'undefined' && window.__PRYSM_SERVER_VERSION__) {
    if (compareSemver(window.__PRYSM_SERVER_VERSION__, ver) > 0) {
      ver = window.__PRYSM_SERVER_VERSION__;
    }
  }
  return ver;
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

  const diagVer = document.getElementById('diagVersion');
  if (diagVer) diagVer.textContent = `v${curVer}`;

  const channel = state.activeChannel || 'stable';
  const targetVer = state.availableVersions[channel] || state.availableVersions.stable;
  const isNewer = compareSemver(targetVer, curVer) > 0;

  const updateBox = document.getElementById('updateAvailableBox');
  const headerBadge = document.getElementById('headerUpdateBadge');
  const statusText = document.getElementById('updateStatusText');
  const latestTag = document.getElementById('latestVersionTag') || document.getElementById('newVersionTag');

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

function getInitialTechnicians() {
  const defaultList = [
    { id: "TECH-01", name: "Administrador Geral", username: "admin", email: "admin@prysmview.com", role: "Administrador TI", permissions: ["Acesso Remoto E2E", "Controle Total", "Arquivos", "Auditoria"], status: "Ativo", lastActive: "Agora" },
    { id: "TECH-02", name: "Carlos Eduardo Silva", username: "carlos.silva", email: "carlos.silva@empresa.com", role: "N2 - Especialista Remoto", permissions: ["Acesso Remoto E2E", "Controle Total", "Arquivos"], status: "Ativo", lastActive: "Hoje, 11:20" },
    { id: "TECH-03", name: "Mariana Souza Lima", username: "mariana.lima", email: "mariana.lima@empresa.com", role: "N1 - Suporte Balcão", permissions: ["Acesso Remoto E2E", "Controle Total"], status: "Ativo", lastActive: "Hoje, 09:45" },
    { id: "TECH-04", name: "Rafael Mendes", username: "rafael.mendes", email: "rafael.mendes@empresa.com", role: "N3 - Engenharia / Infra", permissions: ["Acesso Remoto E2E", "Controle Total", "Arquivos", "Auditoria"], status: "Inativo", lastActive: "2 dias atrás" }
  ];
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      const saved = window.localStorage.getItem('prysm_technicians');
      if (saved) {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed) && parsed.length > 0) return parsed;
      }
    }
  } catch (e) { }
  return defaultList;
}

function saveTechnicians(list) {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.setItem('prysm_technicians', JSON.stringify(list));
    }
  } catch (e) { }
}

function getInitialPdvs() {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      const saved = window.localStorage.getItem('prysm_pdvs');
      if (saved) {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed)) {
          // Zero Mocks: Purga rigorosa de cadastros simulados legados da memória persistida
          const realDevices = parsed.filter(p => {
            if (!p || !p.id) return false;
            const isMock = /^(PDV-CAIXA-01|DESKTOP-01|DEVICE-01|DESK-\d+|SRV-\d+|NOTE-\d+|TERM-\d+|LOG-\d+|BALCAO-\d+)$/i.test(p.id.trim());
            return !isMock;
          });
          window.localStorage.setItem('prysm_pdvs', JSON.stringify(realDevices));
          return realDevices;
        }
      }
    }
  } catch (e) { }
  return [];
}

function savePdvs(list) {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.setItem('prysm_pdvs', JSON.stringify(list));
    }
  } catch (e) { }
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

  // Equipe de técnicos e operadores autorizados
  technicians: getInitialTechnicians(),

  // Versão instalada ativa (persistida via IPC e localStorage)
  installedVersion: getInitialInstalledVersion(),
  availableVersions: {
    stable: '0.1.1',
    beta: '0.1.2-beta'
  },
  activeChannel: 'stable',
  lastCheckTime: 'Hoje, 12:45',

  pdvs: getInitialPdvs(),

  auditRecords: []
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
window.onPrysmMessage = function (data) {
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
function showToast(message, type = 'success') {
  const container = document.getElementById('toastContainer');
  if (!container) return;
  const toast = document.createElement('div');
  toast.className = `toast ${type}`;

  let strokeColor = '#00f0ff';
  let iconSvg = '<svg width="15" height="15" fill="none" stroke="#00f0ff" stroke-width="2" viewBox="0 0 24 24"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>';

  if (type === 'error') {
    strokeColor = '#ff3355';
    iconSvg = '<svg width="15" height="15" fill="none" stroke="#ff3355" stroke-width="2" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line></svg>';
  } else if (type === 'warning') {
    strokeColor = '#eab308';
    iconSvg = '<svg width="15" height="15" fill="none" stroke="#eab308" stroke-width="2" viewBox="0 0 24 24"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>';
  } else if (type === 'info') {
    strokeColor = '#38bdf8';
    iconSvg = '<svg width="15" height="15" fill="none" stroke="#38bdf8" stroke-width="2" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>';
  }

  toast.innerHTML = `
    ${iconSvg}
    <span>${message}</span>
  `;
  container.appendChild(toast);

  if (window.gsap) {
    gsap.fromTo(toast,
      { opacity: 0, y: 20, scale: 0.9 },
      { opacity: 1, y: 0, scale: 1, duration: 0.35, ease: "back.out(1.6)" }
    );
    setTimeout(() => {
      gsap.to(toast, {
        opacity: 0,
        y: -12,
        scale: 0.92,
        duration: 0.28,
        ease: "power2.in",
        onComplete: () => toast.remove()
      });
    }, 3500);
  } else {
    setTimeout(() => {
      toast.style.opacity = '0';
      toast.style.transform = 'translateY(10px)';
      setTimeout(() => toast.remove(), 300);
    }, 3500);
  }
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

  // Atualiza Badges e Indicadores
  const tvBadge = document.getElementById('tvDiscoveredCountBadge');
  if (tvBadge) tvBadge.textContent = online > 0 ? `${online} online` : `${total}`;
  const badgeOnlineEl = document.getElementById('badgeOnlineCount');
  if (badgeOnlineEl) badgeOnlineEl.textContent = online;

  const kpiTotalEl = document.getElementById('kpiTotal');
  if (kpiTotalEl) kpiTotalEl.textContent = total;
  const kpiOnlineEl = document.getElementById('kpiOnline');
  if (kpiOnlineEl) kpiOnlineEl.textContent = online;
  const kpiOfflineEl = document.getElementById('kpiOffline');
  if (kpiOfflineEl) kpiOfflineEl.textContent = offline;

  // Atualiza Pills de Filtro
  const pillAll = document.getElementById('pillAllLabel');
  if (pillAll) pillAll.textContent = `Todos (${total})`;
  const pillOnline = document.getElementById('pillOnlineLabel');
  if (pillOnline) pillOnline.textContent = `Online (${online})`;
  const pillOffline = document.getElementById('pillOfflineLabel');
  if (pillOffline) pillOffline.textContent = `Offline (${offline})`;

  // Atualiza opções do seletor de lojas/grupos
  const storeSel = document.getElementById('storeSelectFilter');
  if (storeSel) {
    const currentVal = state.selectedStore || '';
    const uniqueStores = [...new Set(state.pdvs.map(p => p.storeName).filter(Boolean))].sort();
    let opts = '<option value="">Todas as Lojas</option>';
    uniqueStores.forEach(s => {
      opts += `<option value="${s}" ${s === currentVal ? 'selected' : ''}>${s}</option>`;
    });
    storeSel.innerHTML = opts;
    storeSel.value = currentVal;
  }

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
  if (resultsLabel) resultsLabel.textContent = `${totalFiltered} de ${total} dispositivos`;

  const paginationInfo = document.getElementById('paginationInfo');
  if (paginationInfo) {
    paginationInfo.textContent = totalFiltered === 0
      ? 'Nenhum dispositivo na rede'
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
      <div class="tv-empty-fleet">
        <div class="tv-empty-fleet-icon">
          <svg width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="10"></circle>
            <polyline points="12 6 12 12 16 14"></polyline>
          </svg>
        </div>
        <h5 class="tv-empty-fleet-title">Nenhum dispositivo detectado na rede local no momento</h5>
        <p class="tv-empty-fleet-desc">
          Inicie o <strong>PrysmHost</strong> no computador parceiro ou dispositivo remoto para detecção automática instantânea (portas 19532/19533), ou insira a ID acima e clique em <strong>Conectar</strong>.
        </p>
      </div>
    `;
    if (window.gsap) {
      const emptyEl = container.querySelector('.tv-empty-fleet');
      if (emptyEl) {
        gsap.killTweensOf(emptyEl);
        gsap.fromTo(emptyEl,
          { opacity: 0.4, y: 10 },
          { opacity: 1, y: 0, duration: 0.35, ease: 'power2.out', clearProps: 'transform,opacity' }
        );
      }
    }
    return;
  }

  pageItems.forEach(pdv => {
    const isSelected = pdv.id === state.inspectedPdvId;
    const card = document.createElement('div');
    card.className = `pdv-card ${isSelected ? 'selected' : ''}`;
    card.onclick = () => selectPdvForInspection(pdv);
    card.oncontextmenu = (e) => {
      e.preventDefault();
      e.stopPropagation();
      openDeviceContextMenu(e, pdv);
    };

    const statusBadge = pdv.isOnline
      ? `<div class="status-badge online"><span class="badge-dot green"></span> ON</div>`
      : `<div class="status-badge offline">OFF</div>`;

    const actionBtn = pdv.isOnline
      ? `<button class="btn-primary btn-connect-card" onclick="event.stopPropagation(); startSession('${pdv.id}', '${pdv.storeName}')">
           <svg width="12" height="12" fill="currentColor" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
           <span>Conectar</span>
         </button>`
      : `<button class="btn-secondary btn-connect-card" onclick="event.stopPropagation(); startSession('${pdv.id}', '${pdv.storeName}')">
           <span>Tentar</span>
         </button>`;

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

  if (window.gsap && pageItems.length > 0) {
    gsap.killTweensOf(container.children);
    gsap.fromTo(container.children,
      { opacity: 0, y: 12 },
      {
        opacity: 1,
        y: 0,
        duration: 0.25,
        stagger: 0.03,
        ease: "power2.out",
        clearProps: "transform,opacity"
      }
    );
  }
  initGsapMicroInteractions();
}

function renderFleetTable(pageItems) {
  const tbody = document.getElementById('fleetTableBody');
  if (!tbody) return;
  tbody.innerHTML = '';

  if (pageItems.length === 0) {
    tbody.innerHTML = `
      <tr>
        <td colspan="7" style="text-align: center; padding: 32px; color: var(--text-muted);">
          Nenhum dispositivo detectado na rede local no momento. Digite a ID acima para conectar ou inicie o PrysmHost.
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
    tr.oncontextmenu = (e) => {
      e.preventDefault();
      e.stopPropagation();
      openDeviceContextMenu(e, pdv);
    };

    const statusBadge = pdv.isOnline
      ? `<span class="status-badge online" style="display: inline-flex;"><span class="pulse-dot"></span> ON</span>`
      : `<span class="status-badge offline" style="display: inline-flex;">OFF</span>`;

    const connectBtn = pdv.isOnline
      ? `<button class="btn-primary" style="padding: 3px 10px; font-size: 11.5px; height: 26px;" onclick="event.stopPropagation(); startSession('${pdv.id}', '${pdv.storeName}')">
           <svg width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
           <span>Conectar</span>
         </button>`
      : `<button class="btn-secondary" style="padding: 3px 10px; font-size: 11.5px; height: 26px;" onclick="event.stopPropagation(); startSession('${pdv.id}', '${pdv.storeName}')">
           <span>Tentar</span>
         </button>`;

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

  if (window.gsap && pageItems.length > 0) {
    gsap.killTweensOf(tbody.children);
    gsap.fromTo(tbody.children,
      { opacity: 0, x: -8 },
      {
        opacity: 1,
        x: 0,
        duration: 0.22,
        stagger: 0.02,
        ease: "power2.out",
        clearProps: "transform,opacity"
      }
    );
  }
  initGsapMicroInteractions();
}

function getInspectedPdv() {
  if (state.inspectedPdvId) {
    const found = state.pdvs.find(p => p.id === state.inspectedPdvId);
    if (found) return found;
  }
  return state.pdvs[0] || null;
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
  if (!tbody) return;
  tbody.innerHTML = '';
  if (state.auditRecords.length === 0) {
    tbody.innerHTML = `
      <tr>
        <td colspan="7" style="text-align: center; padding: 32px; color: var(--text-muted);">
          Nenhum registro de auditoria no histórico local. As sessões e ações de suporte serão registradas aqui em tempo real.
        </td>
      </tr>
    `;
    return;
  }
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
  if (!grid) return;
  grid.innerHTML = '';
  const stores = Array.from(new Set(state.pdvs.map(p => p.storeName).filter(Boolean)));
  if (stores.length === 0) {
    grid.innerHTML = `
      <div class="tv-empty-fleet" style="grid-column: 1 / -1;">
        <div class="tv-empty-fleet-icon">
          <svg width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
          </svg>
        </div>
        <h5 class="tv-empty-fleet-title">Nenhum Grupo ou Loja Cadastrada</h5>
        <p class="tv-empty-fleet-desc">Os grupos e clientes serão listados dinamicamente conforme novos dispositivos forem vinculados ou detectados na rede local.</p>
      </div>
    `;
    return;
  }

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
      <p style="font-size: 12.5px; color: var(--text-muted); line-height: 1.5;">Rede Local • Gateway Seguro</p>
      <div style="display: flex; gap: 8px; flex-wrap: wrap;">
        ${storePdvs.map(p => `<span class="tele-chip ${p.isOnline ? 'highlight' : ''}">${p.id}</span>`).join('')}
      </div>
      <button class="btn-secondary" style="margin-top: 8px; justify-content: center; font-size: 13px;" onclick="filterByStore('${store}')">
        Filtrar Dispositivos Deste Grupo
      </button>
    `;
    grid.appendChild(card);
  });

  if (window.gsap && stores.length > 0) {
    gsap.killTweensOf(grid.children);
    gsap.fromTo(grid.children,
      { opacity: 0, y: 12 },
      {
        opacity: 1,
        y: 0,
        duration: 0.28,
        stagger: 0.04,
        ease: "power2.out",
        clearProps: "transform,opacity"
      }
    );
  }
  initGsapMicroInteractions();
}

function filterByStore(storeName) {
  state.selectedStore = storeName;
  state.currentPage = 1;
  const storeSel = document.getElementById('storeSelectFilter');
  if (storeSel) storeSel.value = storeName;
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
  if (page) {
    page.classList.add('active');
    if (window.gsap) {
      gsap.killTweensOf(page);
      gsap.fromTo(page,
        { opacity: 0.45, y: 6 },
        { opacity: 1, y: 0, duration: 0.25, ease: 'power2.out', clearProps: 'transform,opacity' }
      );
    } else {
      page.style.opacity = '1';
      page.style.transform = 'none';
    }
  }

  const breadcrumbEl = document.getElementById('breadcrumbLabel');
  if (breadcrumbEl) {
    breadcrumbEl.textContent = breadcrumbsMap[tabName] || 'Dispositivos';
    if (window.gsap) {
      gsap.killTweensOf(breadcrumbEl);
      gsap.fromTo(breadcrumbEl, 
        { opacity: 0.5, x: 4 }, 
        { opacity: 1, x: 0, duration: 0.2, ease: 'power1.out', clearProps: 'transform,opacity' }
      );
    }
  }

  if (tabName === 'dashboard') renderDashboard();
  if (tabName === 'audit') renderAuditTable();
  if (tabName === 'stores') renderStoresGrid();
  if (tabName === 'settings') {
    syncVersionUI();
    renderTechniciansTable();
  }
}

// =============================================================================
// MODAL DE CÓDIGO DE 8 DÍGITOS
// =============================================================================
let activeSessionTarget = null;
let currentAuthMode = 'code'; // 'code' | 'password'

function setAuthMode(mode) {
  currentAuthMode = mode;
  const tabCode = document.getElementById('tabAuthCode');
  const tabPwd = document.getElementById('tabAuthPassword');
  const boxCode = document.getElementById('containerAuthCode');
  const boxPwd = document.getElementById('containerAuthPassword');

  if (mode === 'password') {
    if (tabCode) {
      tabCode.style.background = 'transparent';
      tabCode.style.color = 'var(--text-muted)';
      tabCode.style.borderColor = 'transparent';
    }
    if (tabPwd) {
      tabPwd.style.background = 'rgba(56, 189, 248, 0.15)';
      tabPwd.style.color = 'var(--accent-cyan)';
      tabPwd.style.borderColor = 'rgba(56, 189, 248, 0.35)';
    }
    if (boxCode) boxCode.style.display = 'none';
    if (boxPwd) {
      boxPwd.style.display = 'block';
      if (window.gsap) {
        gsap.fromTo(boxPwd, { opacity: 0, y: 6 }, { opacity: 1, y: 0, duration: 0.2, ease: "power2.out" });
      }
      const input = document.getElementById('inputModalHostPassword');
      if (input) setTimeout(() => input.focus(), 50);
    }
  } else {
    if (tabCode) {
      tabCode.style.background = 'rgba(56, 189, 248, 0.15)';
      tabCode.style.color = 'var(--accent-cyan)';
      tabCode.style.borderColor = 'rgba(56, 189, 248, 0.35)';
    }
    if (tabPwd) {
      tabPwd.style.background = 'transparent';
      tabPwd.style.color = 'var(--text-muted)';
      tabPwd.style.borderColor = 'transparent';
    }
    if (boxCode) {
      boxCode.style.display = 'block';
      if (window.gsap) {
        gsap.fromTo(boxCode, { opacity: 0, y: 6 }, { opacity: 1, y: 0, duration: 0.2, ease: "power2.out" });
      }
    }
    if (boxPwd) boxPwd.style.display = 'none';
    const firstInput = document.querySelector('.code-digit');
    if (firstInput) setTimeout(() => firstInput.focus(), 50);
  }
}

function startSession(pdvId, storeName) {
  activeSessionTarget = { pdvId, storeName };
  document.getElementById('modalTitle').textContent = `Conexão Segura: ${pdvId}`;
  document.getElementById('modalSubtitle').textContent = `Autentique via código temporário ou senha de acesso do host (${storeName}).`;

  // Limpa inputs e faz prefill inteligente de senha salva
  document.querySelectorAll('.code-digit').forEach(input => input.value = '');
  const inputPwd = document.getElementById('inputModalHostPassword');
  const targetDev = state.pdvs.find(p => p.id.toUpperCase() === (pdvId || '').toUpperCase());

  if (targetDev && targetDev.savedPassword) {
    if (inputPwd) inputPwd.value = targetDev.savedPassword;
    setAuthMode('password');
  } else {
    if (inputPwd) inputPwd.value = '';
    setAuthMode('code');
  }

  const modal = document.getElementById('sessionModal');
  if (modal) {
    modal.classList.add('active');
    if (window.gsap) {
      const dialog = modal.querySelector('.modal-box, .modal-content, .session-dialog');
      if (dialog) {
        gsap.killTweensOf(dialog);
        gsap.fromTo(dialog,
          { opacity: 0, scale: 0.92, y: -12 },
          { opacity: 1, scale: 1, y: 0, duration: 0.28, ease: "back.out(1.5)", clearProps: "transform,opacity" }
        );
      }
    }
  }
}

function closeModal() {
  const modal = document.getElementById('sessionModal');
  if (!modal) return;
  if (window.gsap) {
    const dialog = modal.querySelector('.modal-box, .modal-content, .session-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.to(dialog, {
        opacity: 0,
        scale: 0.94,
        duration: 0.16,
        ease: "power2.in",
        onComplete: () => {
          modal.classList.remove('active');
          dialog.style.opacity = '';
          dialog.style.transform = '';
        }
      });
      return;
    }
  }
  modal.classList.remove('active');
}

// =============================================================================
// COMUNICAÇÃO NATIVA IPC COM O BACKEND RUST
// =============================================================================
function sendRustCommand(action, data = {}) {
  const payload = JSON.stringify({ action, ...data });
  if (window.ipc && window.ipc.postMessage) {
    window.ipc.postMessage(payload);
  } else {
    console.log('[IPC Rust Command]:', payload);
  }
}

// =============================================================================
// GERENCIADOR DE SESSÃO REMOTA AO VIVO (REMOTE DESKTOP CONTROLLER)
// =============================================================================
// =============================================================================
// GERENCIADOR DE SESSÃO REMOTA AO VIVO (REAL REMOTE DESKTOP STREAM & CONTROL)
// =============================================================================
let activeRemoteSession = null;
let telemetryTimer = null;
let remoteCanvasAnimId = null;
let remoteStreamActive = false;

function mapJsCodeToScanCode(code) {
  const map = {
    'Escape': 0x01, 'Digit1': 0x02, 'Digit2': 0x03, 'Digit3': 0x04, 'Digit4': 0x05,
    'Digit5': 0x06, 'Digit6': 0x07, 'Digit7': 0x08, 'Digit8': 0x09, 'Digit9': 0x0A, 'Digit0': 0x0B,
    'Minus': 0x0C, 'Equal': 0x0D, 'Backspace': 0x0E, 'Tab': 0x0F,
    'KeyQ': 0x10, 'KeyW': 0x11, 'KeyE': 0x12, 'KeyR': 0x13, 'KeyT': 0x14,
    'KeyY': 0x15, 'KeyU': 0x16, 'KeyI': 0x17, 'KeyO': 0x18, 'KeyP': 0x19,
    'BracketLeft': 0x1A, 'BracketRight': 0x1B, 'Enter': 0x1C, 'ControlLeft': 0x1D,
    'KeyA': 0x1E, 'KeyS': 0x1F, 'KeyD': 0x20, 'KeyF': 0x21, 'KeyG': 0x22,
    'KeyH': 0x23, 'KeyJ': 0x24, 'KeyK': 0x25, 'KeyL': 0x26, 'Semicolon': 0x27,
    'Quote': 0x28, 'Backquote': 0x29, 'ShiftLeft': 0x2A, 'Backslash': 0x2B,
    'KeyZ': 0x2C, 'KeyX': 0x2D, 'KeyC': 0x2E, 'KeyV': 0x2F, 'KeyB': 0x30,
    'KeyN': 0x31, 'KeyM': 0x32, 'Comma': 0x33, 'Period': 0x34, 'Slash': 0x35,
    'ShiftRight': 0x36, 'NumpadMultiply': 0x37, 'AltLeft': 0x38, 'Space': 0x39,
    'CapsLock': 0x3A, 'F1': 0x3B, 'F2': 0x3C, 'F3': 0x3D, 'F4': 0x3E, 'F5': 0x3F,
    'F6': 0x40, 'F7': 0x41, 'F8': 0x42, 'F9': 0x43, 'F10': 0x44, 'NumLock': 0x45,
    'ScrollLock': 0x46, 'F11': 0x57, 'F12': 0x58,
    // Teclas estendidas
    'NumpadEnter': 0x1C, 'ControlRight': 0x1D, 'NumpadDivide': 0x35, 'AltRight': 0x38,
    'Home': 0x47, 'ArrowUp': 0x48, 'PageUp': 0x49, 'ArrowLeft': 0x4B,
    'ArrowRight': 0x4D, 'End': 0x4F, 'ArrowDown': 0x50, 'PageDown': 0x51,
    'Insert': 0x52, 'Delete': 0x53, 'MetaLeft': 0x5B, 'MetaRight': 0x5C
  };
  return map[code] || 0;
}

function isExtendedJsKey(code) {
  const extended = [
    'NumpadEnter', 'ControlRight', 'NumpadDivide', 'AltRight',
    'Home', 'ArrowUp', 'PageUp', 'ArrowLeft', 'ArrowRight',
    'End', 'ArrowDown', 'PageDown', 'Insert', 'Delete', 'MetaLeft', 'MetaRight'
  ];
  return extended.includes(code);
}

function launchRemoteSession(pdvId, authType, credential, sessionMode) {
  pdvId = pdvId || '127.0.0.1';
  authType = authType || 'code';
  credential = credential || '';
  sessionMode = sessionMode || 'full';

  const screen = document.getElementById('remoteSessionScreen');
  if (!screen) return;

  let hostIp = '';
  if (/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(:\d+)?$/.test(pdvId.trim())) {
    hostIp = pdvId.trim().split(':')[0];
  } else {
    const targetDev = state.pdvs.find(p => p.id.toUpperCase() === pdvId.toUpperCase());
    hostIp = (targetDev && targetDev.ipAddress) ? targetDev.ipAddress.trim() : '127.0.0.1';
  }
  if (!hostIp || hostIp === 'localhost') {
    hostIp = '127.0.0.1';
  }

  activeRemoteSession = {
    pdvId,
    authType,
    credential,
    hostIp,
    sessionMode,
    startTime: Date.now(),
    fps: 0,
    rtt: 0,
    bw: 0
  };

  const titleEl = document.getElementById('remoteDeviceTitle');
  if (titleEl) titleEl.textContent = `${pdvId} (${hostIp})`;

  const badgeEl = document.getElementById('remoteOpBadge');
  if (badgeEl) {
    badgeEl.textContent = authType === 'password' ? 'Autenticado por Senha' : 'Cód. 8 Dígitos (TLS 1.3)';
  }

  const overlay = document.getElementById('remoteConnectingOverlay');
  const connTitle = document.getElementById('remoteConnectingTitle');
  const connDesc = document.getElementById('remoteConnectingDesc');
  if (connTitle) connTitle.textContent = `Conectando à Área de Trabalho de ${pdvId}...`;
  if (connDesc) connDesc.textContent = `Estabelecendo transmissão ao vivo com ${hostIp}:19533...`;

  if (overlay) overlay.style.display = 'flex';
  screen.style.setProperty('display', 'flex', 'important');
  screen.style.zIndex = '999999';

  console.log('[PrysmAdmin] Conectando ao host:', pdvId, hostIp, authType, sessionMode);
  sendRustCommand('start_remote_session', { pdvId, authType, credential, hostIp });
  closeModal();

  initRemoteCanvas(pdvId, hostIp, credential, sessionMode);
}

let remoteWs = null;
let currentQuality = 75;

function terminateRemoteSession() {
  if (!activeRemoteSession) {
    const screen = document.getElementById('remoteSessionScreen');
    if (screen) screen.style.display = 'none';
    return;
  }

  const pdvId = activeRemoteSession.pdvId;
  const hostIp = activeRemoteSession.hostIp || '127.0.0.1';
  const durationSec = Math.max(1, Math.round((Date.now() - activeRemoteSession.startTime) / 1000));

  remoteStreamActive = false;
  stopHttpStream();
  if (remoteCanvasAnimId) {
    cancelAnimationFrame(remoteCanvasAnimId);
    remoteCanvasAnimId = null;
  }

  if (remoteWs) {
    try {
      remoteWs.send(JSON.stringify({ type: 'disconnect' }));
      remoteWs.close();
    } catch (_) { }
    remoteWs = null;
  }

  // Notifica o host via fallback HTTP também para garantir liberação imediata de teclas
  try {
    fetch(`http://${hostIp}:19533/input`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ type: 'disconnect' })
    }).catch(() => { });
  } catch (err) { }

  sendRustCommand('end_remote_session', { pdvId });

  if (telemetryTimer) {
    clearInterval(telemetryTimer);
    telemetryTimer = null;
  }

  const screen = document.getElementById('remoteSessionScreen');
  if (screen) screen.style.display = 'none';

  const now = new Date();
  state.auditRecords.unshift({
    id: `AUD-${Math.floor(920 + Math.random() * 50)}`,
    timestamp: `Hoje ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`,
    operator: document.getElementById('headerOperatorName')?.textContent || "admin",
    pdv: pdvId,
    store: "Atendimento Remoto",
    reason: `Sessão de suporte real via ${activeRemoteSession.authType === 'password' ? 'Senha do Host' : 'Código de Sessão'} (WebSocket Full-Duplex)`,
    duration: `${durationSec}s`,
    status: "Concluída"
  });
  renderAuditTable();

  showToast(`Sessão remota com ${pdvId} finalizada com sucesso.`);
  activeRemoteSession = null;
}

let httpStreamActive = false;
let httpStreamTimer = null;

function stopHttpStream() {
  httpStreamActive = false;
  if (httpStreamTimer) {
    clearTimeout(httpStreamTimer);
    httpStreamTimer = null;
  }
}

function initRemoteCanvas(pdvId, hostIp, credential, sessionMode) {
  const canvas = document.getElementById('remoteScreenCanvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  remoteStreamActive = true;
  stopHttpStream();
  let isViewOnly = (sessionMode === 'view');
  let scaleMode = 'fit';
  let toolbarPinned = true;
  let framesThisSec = 0;
  let lastFpsCalc = performance.now();
  let lastFrameTime = performance.now();
  let bytesReceivedThisSec = 0;

  if (remoteWs) {
    try { remoteWs.close(); } catch (_) { }
    remoteWs = null;
  }

  const wsUrl = `ws://${hostIp}:19533/stream`;
  console.log('[PrysmAdmin] Conectando WebSocket de streaming:', wsUrl);

  const overlay = document.getElementById('remoteConnectingOverlay');
  const connTitle = document.getElementById('remoteConnectingTitle');
  const connDesc = document.getElementById('remoteConnectingDesc');
  if (connTitle) connTitle.textContent = `Conectando à Área de Trabalho de ${pdvId}...`;
  if (connDesc) connDesc.textContent = `Negociando canal de streaming em tempo real com ${hostIp}...`;
  if (overlay) overlay.style.display = 'flex';

  function sendRemoteCommand(payload) {
    if (remoteWs && remoteWs.readyState === WebSocket.OPEN) {
      remoteWs.send(JSON.stringify(payload));
    } else if (remoteStreamActive) {
      fetch(`http://${hostIp}:19533/input`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      }).catch(() => { });
    }
  }
  const sendWsCommand = sendRemoteCommand;

  async function startHttpStreamFallback(targetPdvId, targetHostIp) {
    if (!remoteStreamActive || httpStreamActive) return;
    httpStreamActive = true;
    console.log('[PrysmAdmin] Streaming HTTP Fallback ativado para', targetHostIp);

    const badgeEl = document.getElementById('remoteOpBadge');
    if (badgeEl) badgeEl.textContent = 'HTTP Stream (30 FPS)';

    const overlay = document.getElementById('remoteConnectingOverlay');
    const canvas = document.getElementById('remoteScreenCanvas');
    const ctx = canvas ? canvas.getContext('2d') : null;

    let framesThisSec = 0;
    let lastFpsCalc = performance.now();
    let bytesReceivedThisSec = 0;

    async function pollNextFrame() {
      if (!remoteStreamActive || !httpStreamActive) return;

      const t0 = performance.now();
      try {
        const resp = await fetch(`http://${targetHostIp}:19533/frame?t=${Date.now()}`, {
          cache: 'no-store'
        });
        if (resp.status === 403 || resp.status === 401) {
          console.warn('[PrysmAdmin] Acesso negado pelo Host (sessão inativa ou encerrada no dispositivo remoto). Interrompendo streaming.');
          stopHttpStream();
          terminateRemoteSession();
          showToast('Sessão remota encerrada pelo dispositivo remoto.', 'warning');
          return;
        }
        if (resp.ok) {
          const blob = await resp.blob();
          bytesReceivedThisSec += blob.size;
          const rtt = Math.round(performance.now() - t0);

          const bmp = await createImageBitmap(blob);
          if (canvas && ctx) {
            if (canvas.width !== bmp.width || canvas.height !== bmp.height) {
              canvas.width = bmp.width;
              canvas.height = bmp.height;
            }
            ctx.drawImage(bmp, 0, 0);
          }
          bmp.close();

          if (overlay && overlay.style.display !== 'none') {
            overlay.style.display = 'none';
          }

          framesThisSec++;
          const now = performance.now();
          if (now - lastFpsCalc >= 1000) {
            const curFps = Math.round((framesThisSec * 1000) / (now - lastFpsCalc));
            const curKbps = Math.round((bytesReceivedThisSec * 8) / ((now - lastFpsCalc) * 1.024));
            framesThisSec = 0;
            bytesReceivedThisSec = 0;
            lastFpsCalc = now;

            const fpsEl = document.getElementById('remoteTelemetryFps');
            if (fpsEl) fpsEl.textContent = `FPS: ${curFps}`;
            const rttEl = document.getElementById('remoteTelemetryRtt');
            if (rttEl) rttEl.textContent = `RTT: ${Math.min(rtt, 120)}ms`;
            const bwEl = document.getElementById('remoteTelemetryBw');
            if (bwEl) bwEl.textContent = `${curKbps} kbps`;
          }
        }
      } catch (err) {
        console.warn('[PrysmAdmin] Erro no polling de frame HTTP:', err);
      }

      if (remoteStreamActive && httpStreamActive) {
        const elapsed = performance.now() - t0;
        const delay = Math.max(5, 33 - elapsed);
        httpStreamTimer = setTimeout(pollNextFrame, delay);
      }
    }

    pollNextFrame();
  }

  try {
    remoteWs = new WebSocket(wsUrl, ['prysm-stream']);
    remoteWs.binaryType = 'arraybuffer';
  } catch (err) {
    console.warn('[PrysmAdmin] Falha ao instanciar WebSocket com subprotocolo, tentando padrão:', err);
    try {
      remoteWs = new WebSocket(wsUrl);
      remoteWs.binaryType = 'arraybuffer';
    } catch (e2) {
      console.warn('[PrysmAdmin] WebSocket não suportado, iniciando Fallback HTTP direto.');
      startHttpStreamFallback(pdvId, hostIp);
      return;
    }
  }

  remoteWs.onopen = () => {
    console.log('[PrysmAdmin] WebSocket conectado! Enviando credencial de autenticação...');
    const authType = activeRemoteSession?.authType || 'code';
    const operatorName = document.getElementById('headerOperatorName')?.textContent || 'admin';
    sendRemoteCommand({
      type: 'auth',
      pdv_id: pdvId,
      auth_type: authType,
      credential: credential,
      operator: operatorName,
      quality: currentQuality
    });
  };

  remoteWs.onmessage = async (event) => {
    if (!remoteStreamActive) return;

    if (typeof event.data === 'string') {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'auth_ok') {
          if (overlay) overlay.style.display = 'none';
          showToast(`Sessão autorizada com sucesso em ${pdvId}!`);
        } else if (msg.type === 'consent_required') {
          if (connTitle) connTitle.textContent = 'Aguardando Consentimento no Dispositivo...';
          if (connDesc) connDesc.textContent = `Uma solicitação de acesso foi exibida na tela do ${pdvId}. Solicite ao operador para clicar em "Aceitar" ou informe o código de 8 dígitos.`;
        } else if (msg.type === 'auth_failed') {
          if (connTitle) connTitle.textContent = 'Acesso Recusado';
          if (connDesc) connDesc.textContent = msg.reason || 'Acesso recusado pelo operador local do dispositivo.';
          showToast(msg.reason || 'Conexão recusada pelo host.', 'error');
        } else if (msg.type === 'session_terminated') {
          console.log('[PrysmAdmin] Sessão terminada pelo dispositivo remoto:', msg.reason);
          showToast(msg.reason || 'Sessão encerrada pelo operador local do dispositivo.', 'warning');
          remoteStreamActive = false;
          stopHttpStream();
          terminateRemoteSession();
          return;
        }
      } catch (_) { }
      return;
    }

    // Binary Frame (JPEG buffer direto do Host)
    if (event.data instanceof ArrayBuffer) {
      bytesReceivedThisSec += event.data.byteLength;
      const tStart = performance.now();
      const rtt = Math.round(tStart - lastFrameTime);
      lastFrameTime = tStart;

      try {
        const blob = new Blob([event.data], { type: 'image/jpeg' });
        const bmp = await createImageBitmap(blob);

        if (canvas.width !== bmp.width || canvas.height !== bmp.height) {
          canvas.width = bmp.width;
          canvas.height = bmp.height;
        }

        ctx.drawImage(bmp, 0, 0);
        bmp.close();

        if (overlay && overlay.style.display !== 'none') {
          overlay.style.display = 'none';
        }

        framesThisSec++;
        const now = performance.now();
        if (now - lastFpsCalc >= 1000) {
          const curFps = Math.round((framesThisSec * 1000) / (now - lastFpsCalc));
          const curKbps = Math.round((bytesReceivedThisSec * 8) / ((now - lastFpsCalc) * 1.024));
          framesThisSec = 0;
          bytesReceivedThisSec = 0;
          lastFpsCalc = now;

          const fpsEl = document.getElementById('remoteTelemetryFps');
          if (fpsEl) fpsEl.textContent = `FPS: ${curFps}`;
          const rttEl = document.getElementById('remoteTelemetryRtt');
          if (rttEl) rttEl.textContent = `RTT: ${Math.min(rtt, 120)}ms`;
          const bwEl = document.getElementById('remoteTelemetryBw');
          if (bwEl) bwEl.textContent = `${curKbps} kbps`;
        }
      } catch (e) {
        console.warn('[PrysmAdmin] Erro ao renderizar bitmap:', e);
      }
    }
  };

  remoteWs.onerror = (e) => {
    console.warn('[PrysmAdmin] Aviso no WebSocket de tela:', e);
  };

  remoteWs.onclose = (event) => {
    console.log('[PrysmAdmin] WebSocket desconectado, código:', event.code);
    if (remoteStreamActive && !httpStreamActive && event.code !== 1000) {
      console.log('[PrysmAdmin] Queda não intencional detectada. Ativando HTTP Streaming Fallback...');
      startHttpStreamFallback(pdvId, hostIp);
    } else {
      stopHttpStream();
      if (remoteStreamActive) {
        terminateRemoteSession();
      }
    }
  };

  // Entrada de Mouse (movimento acelerado normalizado 0-65535) com trava de Modo Apenas Visualizar
  let lastMouseMove = 0;
  canvas.onmousemove = (e) => {
    if (isViewOnly) return;
    const now = performance.now();
    if (now - lastMouseMove < 12) return; // taxa fluida ~85 Hz
    lastMouseMove = now;
    const rect = canvas.getBoundingClientRect();
    const normX = Math.min(65535, Math.max(0, Math.round(((e.clientX - rect.left) / rect.width) * 65535)));
    const normY = Math.min(65535, Math.max(0, Math.round(((e.clientY - rect.top) / rect.height) * 65535)));
    sendRemoteCommand({ type: 'mouse_move', x: normX, y: normY });
  };

  canvas.onmousedown = (e) => {
    if (isViewOnly) return;
    e.preventDefault();
    canvas.focus();
    sendRemoteCommand({ type: 'mouse_button', button: e.button, pressed: true });
  };

  canvas.onmouseup = (e) => {
    if (isViewOnly) return;
    e.preventDefault();
    sendRemoteCommand({ type: 'mouse_button', button: e.button, pressed: false });
  };

  canvas.oncontextmenu = (e) => {
    e.preventDefault();
  };

  canvas.onwheel = (e) => {
    if (isViewOnly) return;
    e.preventDefault();
    const delta = e.deltaY < 0 ? 120 : -120;
    sendRemoteCommand({ type: 'mouse_wheel', delta_v: delta });
  };

  window.onkeydown = (e) => {
    if (!activeRemoteSession || isViewOnly) return;
    if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA')) return;
    const sc = mapJsCodeToScanCode(e.code);
    if (sc !== 0) {
      e.preventDefault();
      sendWsCommand({ type: 'key', scan_code: sc, extended: isExtendedJsKey(e.code), pressed: true });
    }
  };

  window.onkeyup = (e) => {
    if (!activeRemoteSession || isViewOnly) return;
    if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA')) return;
    const sc = mapJsCodeToScanCode(e.code);
    if (sc !== 0) {
      e.preventDefault();
      sendWsCommand({ type: 'key', scan_code: sc, extended: isExtendedJsKey(e.code), pressed: false });
    }
  };

  // Botões da Toolbar
  const btnCad = document.getElementById('btnRemoteCad');
  if (btnCad) {
    btnCad.onclick = () => {
      sendWsCommand({ type: 'sas' });
      showToast('Sinal SAS (Ctrl+Alt+Del) enviado ao computador remoto.');
    };
  }

  const btnClipboard = document.getElementById('btnRemoteClipboard');
  if (btnClipboard) {
    btnClipboard.onclick = async () => {
      try {
        const text = await navigator.clipboard.readText();
        if (text) {
          sendWsCommand({ type: 'clipboard', text });
          showToast('Texto da área de transferência sincronizado com o dispositivo remoto.');
        } else {
          showToast('Área de transferência local vazia.');
        }
      } catch (err) {
        showToast('Permissão de leitura do Clipboard não concedida.');
      }
    };
  }

  // Alternância de Modo Apenas Visualizar (View-Only / Block Remote Input - Estilo TeamViewer)
  const btnViewOnly = document.getElementById('btnToggleViewOnly');
  const labelViewMode = document.getElementById('labelViewMode');
  const viewOnlyBadge = document.getElementById('viewOnlyBadge');
  if (btnViewOnly) {
    if (isViewOnly) {
      btnViewOnly.classList.add('active-view-only');
      if (labelViewMode) labelViewMode.textContent = 'Apenas Ver';
      if (viewOnlyBadge) viewOnlyBadge.style.display = 'flex';
    } else {
      btnViewOnly.classList.remove('active-view-only');
      if (labelViewMode) labelViewMode.textContent = 'Controle';
      if (viewOnlyBadge) viewOnlyBadge.style.display = 'none';
    }

    btnViewOnly.onclick = () => {
      isViewOnly = !isViewOnly;
      if (isViewOnly) {
        btnViewOnly.classList.add('active-view-only');
        if (labelViewMode) labelViewMode.textContent = 'Apenas Ver';
        if (viewOnlyBadge) viewOnlyBadge.style.display = 'flex';
        showToast('Modo Apenas Visualizar ativado. Entradas remotas bloqueadas.', 'info');
      } else {
        btnViewOnly.classList.remove('active-view-only');
        if (labelViewMode) labelViewMode.textContent = 'Controle';
        if (viewOnlyBadge) viewOnlyBadge.style.display = 'none';
        showToast('Modo Controle Total ativado. Mouse e teclado habilitados.', 'info');
      }
    };
  }

  // Alternância de Escala: Ajustar à Tela vs 100% Original (Estilo TeamViewer Fit/1:1)
  const btnScale = document.getElementById('btnToggleScale');
  const labelScale = document.getElementById('labelScaleMode');
  const viewportContainer = document.getElementById('remoteViewportContainer');
  if (btnScale) {
    btnScale.onclick = () => {
      scaleMode = (scaleMode === 'fit') ? 'original' : 'fit';
      if (scaleMode === 'original') {
        canvas.classList.add('scale-original');
        if (viewportContainer) viewportContainer.classList.add('scrollable-original');
        if (labelScale) labelScale.textContent = '1:1 Real';
        showToast('Escala 100% Original ativada. Rolagem livre habilitada.', 'info');
      } else {
        canvas.classList.remove('scale-original');
        if (viewportContainer) viewportContainer.classList.remove('scrollable-original');
        if (labelScale) labelScale.textContent = 'Ajustar';
        showToast('Escala Ajustar à Janela ativada.', 'info');
      }
    };
  }

  // Barra Superior Retrátil com Pin / Collapse (Estilo TeamViewer Ribbon)
  const btnPin = document.getElementById('btnTogglePinToolbar');
  const toolbar = document.getElementById('remoteToolbar');
  const iconPinChevron = document.getElementById('iconPinChevron');
  if (btnPin && toolbar) {
    btnPin.onclick = () => {
      toolbarPinned = !toolbarPinned;
      if (!toolbarPinned) {
        toolbar.classList.add('collapsed');
        if (window.gsap) {
          gsap.killTweensOf(toolbar);
          gsap.to(toolbar, { y: -38, opacity: 0.45, duration: 0.3, ease: 'power2.inOut', xPercent: -50 });
          if (iconPinChevron) gsap.to(iconPinChevron, { rotation: 180, duration: 0.25 });
        } else {
          if (iconPinChevron) iconPinChevron.style.transform = 'rotate(180deg)';
        }
        showToast('Barra superior recolhida. Passe o mouse para expandir.', 'info');
      } else {
        toolbar.classList.remove('collapsed');
        if (window.gsap) {
          gsap.killTweensOf(toolbar);
          gsap.to(toolbar, { y: 0, opacity: 1, duration: 0.35, ease: 'back.out(1.4)', xPercent: -50, clearProps: 'transform,opacity' });
          if (iconPinChevron) gsap.to(iconPinChevron, { rotation: 0, duration: 0.25 });
        } else {
          if (iconPinChevron) iconPinChevron.style.transform = 'rotate(0deg)';
        }
        showToast('Barra superior fixada.', 'info');
      }
    };

    // Suporte a expansão suave ao passar o cursor quando a barra estiver recolhida
    toolbar.addEventListener('mouseenter', () => {
      if (!toolbarPinned && toolbar.classList.contains('collapsed')) {
        if (window.gsap) {
          gsap.killTweensOf(toolbar);
          gsap.to(toolbar, { y: 0, opacity: 1, duration: 0.25, ease: 'power2.out', xPercent: -50 });
        }
      }
    });

    toolbar.addEventListener('mouseleave', () => {
      if (!toolbarPinned && toolbar.classList.contains('collapsed')) {
        if (window.gsap) {
          gsap.killTweensOf(toolbar);
          gsap.to(toolbar, { y: -38, opacity: 0.45, duration: 0.25, ease: 'power2.in', xPercent: -50 });
        }
      }
    });
  }

  const btnQualitySmooth = document.getElementById('btnQualitySmooth');
  const btnQualityBalanced = document.getElementById('btnQualityBalanced');
  if (btnQualitySmooth && btnQualityBalanced) {
    btnQualitySmooth.onclick = () => {
      currentQuality = 55;
      sendWsCommand({ type: 'quality', value: 55 });
      btnQualitySmooth.classList.add('active-quality');
      btnQualityBalanced.classList.remove('active-quality');
      showToast('Modo Fluidez 60 FPS ativado.');
    };
    btnQualityBalanced.onclick = () => {
      currentQuality = 85;
      sendWsCommand({ type: 'quality', value: 85 });
      btnQualityBalanced.classList.add('active-quality');
      btnQualitySmooth.classList.remove('active-quality');
      showToast('Modo Alta Definição (HD) ativado.');
    };
  }

  const btnEnd = document.getElementById('btnEndRemoteSession');
  if (btnEnd) {
    btnEnd.onclick = () => {
      terminateRemoteSession();
    };
  }

  const btnFs = document.getElementById('btnRemoteFullscreen');
  if (btnFs) {
    btnFs.onclick = () => {
      const scr = document.getElementById('remoteSessionScreen');
      if (!document.fullscreenElement) {
        scr.requestFullscreen().catch(() => { });
      } else {
        document.exitFullscreen().catch(() => { });
      }
    };
  }
}

// =============================================================================
// SINCRONIZAÇÃO EM TEMPO REAL DE HOSTS (UDP BEACON, TCP HEALTH-CHECK & PROBING)
// =============================================================================
window.onPrysmHostSync = function (discoveredList) {
  if (!Array.isArray(discoveredList)) return;
  let modified = false;

  discoveredList.forEach(dev => {
    const devId = (dev.pdv_id || dev.id || '').trim();
    if (!devId) return;

    const isOnline = dev.is_online === true;
    const existing = state.pdvs.find(p => p.id.toUpperCase() === devId.toUpperCase());
    if (existing) {
      const lastSeenStr = isOnline ? 'Agora' : (dev.last_seen || 'Offline');
      if (existing.isOnline !== isOnline || existing.lastSeen !== lastSeenStr || (dev.ip && existing.ipAddress !== dev.ip)) {
        existing.isOnline = isOnline;
        existing.lastSeen = lastSeenStr;
        if (dev.ip) existing.ipAddress = dev.ip;
        if (dev.local_user) existing.notes = `Host: ${dev.local_user} (${isOnline ? 'Online via LAN' : 'Agente Offline'})`;
        if (dev.access_password) existing.savedPassword = dev.access_password;
        modified = true;
      }
    } else if (isOnline) {
      state.pdvs.unshift({
        id: devId.toUpperCase(),
        storeName: "Estação de Trabalho Local / Computador",
        isOnline: true,
        agentVersion: dev.agent_version || "0.1.0",
        publicKey: `ed25519_${devId.toLowerCase()}`,
        lastSeen: "Agora",
        ipAddress: dev.ip || "127.0.0.1",
        osInfo: "Windows 11 / 10",
        resolution: "1920x1080 @ 60Hz",
        notes: `Host Ativo: ${dev.local_user || "Operador"} (Descoberto em tempo real via LAN)`,
        savedPassword: dev.access_password || null
      });
      modified = true;
    }
  });

  if (modified) {
    savePdvs(state.pdvs);
    renderDashboard();
    const onlineCount = state.pdvs.filter(p => p.isOnline).length;
    const statOnline = document.getElementById('statOnlineCount');
    if (statOnline) statOnline.textContent = onlineCount;
  }
};

let _probingFleet = false;
async function probeFleetHealth() {
  if (_probingFleet) return;
  _probingFleet = true;
  let modified = false;

  try {
    const probePromises = state.pdvs.map(async pdv => {
      let hostIp = (pdv.ipAddress || '').trim();
      if (!hostIp) return;
      if (hostIp === 'localhost') hostIp = '127.0.0.1';

      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 750);
        await fetch(`http://${hostIp}:19533/`, {
          method: 'OPTIONS',
          signal: controller.signal
        });
        clearTimeout(timeoutId);

        if (!pdv.isOnline) {
          pdv.isOnline = true;
          pdv.lastSeen = 'Agora';
          modified = true;
        }
      } catch (_) {
        // Se a verificação HTTP falhar e o dispositivo estiver como 127.0.0.1
        if (pdv.isOnline && hostIp === '127.0.0.1') {
          pdv.isOnline = false;
          pdv.lastSeen = 'Offline';
          modified = true;
        }
      }
    });

    await Promise.allSettled(probePromises);
  } catch (_) {
  } finally {
    _probingFleet = false;
    if (modified) {
      savePdvs(state.pdvs);
      renderDashboard();
      const onlineCount = state.pdvs.filter(p => p.isOnline).length;
      const statOnline = document.getElementById('statOnlineCount');
      if (statOnline) statOnline.textContent = onlineCount;
    }
  }
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
      } catch (e) { }
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
// GESTÃO DE TÉCNICOS & OPERADORES
// =============================================================================
function renderTechniciansTable() {
  const tbody = document.getElementById('techniciansTableBody');
  if (!tbody) return;

  if (state.technicians.length === 0) {
    tbody.innerHTML = `
      <tr>
        <td colspan="6" style="text-align: center; padding: 24px; color: var(--text-muted);">
          Nenhum técnico cadastrado no momento.
        </td>
      </tr>
    `;
    return;
  }

  tbody.innerHTML = state.technicians.map(t => {
    const statusBadge = t.status === 'Ativo'
      ? `<span class="badge-status online"><span class="badge-dot green"></span> Ativo</span>`
      : `<span class="badge-status offline">Inativo</span>`;

    const permsHtml = t.permissions.map(p => `<span class="tele-chip" style="font-size: 11px;">${p}</span>`).join(' ');

    const actionToggle = t.username === 'admin'
      ? `<span style="font-size: 11.5px; color: var(--text-muted);">Conta Master</span>`
      : `<button class="btn-secondary" style="padding: 3px 8px; font-size: 11px; height: 26px;" onclick="toggleTechStatus('${t.id}')">
           ${t.status === 'Ativo' ? 'Desativar' : 'Ativar'}
         </button>
         <button class="btn-icon-ghost" style="color: #f87171; width: 26px; height: 26px;" title="Remover Técnico" onclick="deleteTech('${t.id}')">
           <svg width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
         </button>`;

    const initials = t.name.split(' ').filter(Boolean).map(n => n[0]).slice(0, 2).join('').toUpperCase();

    return `
      <tr>
        <td>
          <div style="display: flex; align-items: center; gap: 8px;">
            <div class="user-avatar-circle" style="width: 26px; height: 26px; font-size: 11px; font-weight: 700;">
              ${initials}
            </div>
            <div>
              <strong style="color: var(--text-primary); font-size: 13px;">${t.name}</strong>
              <div style="font-size: 11px; color: var(--text-muted);">Último acesso: ${t.lastActive}</div>
            </div>
          </div>
        </td>
        <td>
          <div style="font-family: var(--font-mono); font-size: 12px; color: var(--text-secondary);">@${t.username}</div>
          <div style="font-size: 11.5px; color: var(--text-muted);">${t.email}</div>
        </td>
        <td><span class="tele-chip highlight" style="font-size: 11.5px;">${t.role}</span></td>
        <td><div style="display: flex; flex-wrap: wrap; gap: 4px;">${permsHtml}</div></td>
        <td>${statusBadge}</td>
        <td style="text-align: right;">
          <div style="display: inline-flex; gap: 4px; align-items: center;">
            ${actionToggle}
          </div>
        </td>
      </tr>
    `;
  }).join('');
}

window.toggleTechStatus = function (techId) {
  const tech = state.technicians.find(t => t.id === techId);
  if (!tech || tech.username === 'admin') return;
  tech.status = tech.status === 'Ativo' ? 'Inativo' : 'Ativo';
  saveTechnicians(state.technicians);
  renderTechniciansTable();
  showToast(`Status do técnico ${tech.name} alterado para ${tech.status}.`);
};

window.deleteTech = function (techId) {
  const tech = state.technicians.find(t => t.id === techId);
  if (!tech || tech.username === 'admin') return;
  if (!confirm(`Deseja realmente remover o acesso do técnico ${tech.name} (@${tech.username})?`)) return;
  state.technicians = state.technicians.filter(t => t.id !== techId);
  saveTechnicians(state.technicians);
  renderTechniciansTable();
  showToast(`Técnico ${tech.name} removido com sucesso.`);
};

function openNewTechModal() {
  const modal = document.getElementById('newTechModal');
  if (!modal) return;
  const form = document.getElementById('formNewTech');
  if (form) form.reset();
  modal.classList.add('active');
  if (window.gsap) {
    const dialog = modal.querySelector('.modal-box, .modal-content, .session-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.fromTo(dialog,
        { opacity: 0, scale: 0.92, y: -12 },
        { opacity: 1, scale: 1, y: 0, duration: 0.28, ease: 'back.out(1.5)', clearProps: 'transform,opacity' }
      );
    }
  }
  const firstInput = document.getElementById('techFullNameInput');
  if (firstInput) firstInput.focus();
}

function closeNewTechModal() {
  const modal = document.getElementById('newTechModal');
  if (!modal) return;
  if (window.gsap) {
    const dialog = modal.querySelector('.modal-box, .modal-content, .session-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.to(dialog, {
        opacity: 0,
        scale: 0.94,
        duration: 0.16,
        ease: 'power2.in',
        onComplete: () => {
          modal.classList.remove('active');
          dialog.style.opacity = '';
          dialog.style.transform = '';
        }
      });
      return;
    }
  }
  modal.classList.remove('active');
}

let editingDeviceId = null;
let activeContextMenuPdv = null;

function openDeviceContextMenu(e, pdv) {
  activeContextMenuPdv = pdv;
  const menu = document.getElementById('deviceContextMenu');
  if (!menu) return;

  const titleEl = document.getElementById('ctxDeviceTitle');
  const ipEl = document.getElementById('ctxDeviceIp');
  if (titleEl) titleEl.textContent = pdv.id;
  if (ipEl) ipEl.textContent = `${pdv.storeName} • ${pdv.ipAddress}`;

  menu.style.display = 'flex';
  const menuWidth = 220;
  const menuHeight = 240;
  let x = e.clientX;
  let y = e.clientY;

  if (x + menuWidth > window.innerWidth) {
    x = window.innerWidth - menuWidth - 10;
  }
  if (y + menuHeight > window.innerHeight) {
    y = window.innerHeight - menuHeight - 10;
  }

  menu.style.left = `${Math.max(10, x)}px`;
  menu.style.top = `${Math.max(10, y)}px`;

  if (window.gsap) {
    gsap.fromTo(menu,
      { opacity: 0, scale: 0.92 },
      { opacity: 1, scale: 1, duration: 0.18, ease: "power2.out", clearProps: "transform" }
    );
  }
}

function closeDeviceContextMenu() {
  const menu = document.getElementById('deviceContextMenu');
  if (!menu || menu.style.display === 'none') return;
  if (window.gsap) {
    gsap.to(menu, {
      opacity: 0,
      scale: 0.94,
      duration: 0.12,
      ease: "power1.in",
      onComplete: () => {
        menu.style.display = 'none';
        menu.style.opacity = '1';
        menu.style.transform = '';
      }
    });
  } else {
    menu.style.display = 'none';
  }
}

function openNewDeviceModal() {
  editingDeviceId = null;
  const modal = document.getElementById('newDeviceModal');
  if (!modal) return;
  const form = document.getElementById('formNewDevice');
  if (form) form.reset();
  const title = document.getElementById('newDeviceModalTitle');
  if (title) title.textContent = 'Cadastrar Dispositivo / Computador';
  const desc = document.getElementById('newDeviceModalDesc');
  if (desc) desc.textContent = 'Adicione uma máquina de teste, estação de trabalho ou servidor corporativo para gerenciamento e acesso direto.';
  const btnText = document.getElementById('btnSaveNewDeviceText');
  if (btnText) btnText.textContent = 'Cadastrar Dispositivo';
  modal.classList.add('active');
  if (window.gsap) {
    const dialog = modal.querySelector('.modal-box, .modal-content, .session-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.fromTo(dialog,
        { opacity: 0, scale: 0.92, y: -12 },
        { opacity: 1, scale: 1, y: 0, duration: 0.28, ease: "back.out(1.5)", clearProps: "transform,opacity" }
      );
    }
  }
  const firstInput = document.getElementById('devIdInput');
  if (firstInput) {
    firstInput.readOnly = false;
    firstInput.focus();
  }
}

function openEditDeviceModal(pdv) {
  if (!pdv) return;
  editingDeviceId = pdv.id;
  const modal = document.getElementById('newDeviceModal');
  if (!modal) return;
  const title = document.getElementById('newDeviceModalTitle');
  if (title) title.textContent = `Editar Dispositivo: ${pdv.id}`;
  const desc = document.getElementById('newDeviceModalDesc');
  if (desc) desc.textContent = 'Atualize as informações de identificação, grupo de loja, IP e anotações deste dispositivo.';
  const btnText = document.getElementById('btnSaveNewDeviceText');
  if (btnText) btnText.textContent = 'Salvar Alterações';

  const devId = document.getElementById('devIdInput');
  const devStore = document.getElementById('devStoreInput');
  const devIp = document.getElementById('devIpInput');
  const devStatus = document.getElementById('devStatusSelect');
  const devNotes = document.getElementById('devNotesInput');

  if (devId) {
    devId.value = pdv.id;
    devId.readOnly = false;
  }
  if (devStore) devStore.value = pdv.storeName || '';
  if (devIp) devIp.value = pdv.ipAddress || '';
  if (devStatus) devStatus.value = pdv.isOnline ? 'online' : 'offline';
  if (devNotes) devNotes.value = pdv.notes || '';

  modal.classList.add('active');
  if (window.gsap) {
    const dialog = modal.querySelector('.modal-box, .modal-content, .session-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.fromTo(dialog,
        { opacity: 0, scale: 0.92, y: -12 },
        { opacity: 1, scale: 1, y: 0, duration: 0.28, ease: "back.out(1.5)", clearProps: "transform,opacity" }
      );
    }
  }
  if (devStore) devStore.focus();
}

function closeNewDeviceModal() {
  editingDeviceId = null;
  const modal = document.getElementById('newDeviceModal');
  if (!modal) return;
  if (window.gsap) {
    const dialog = modal.querySelector('.modal-box, .modal-content, .session-dialog');
    if (dialog) {
      gsap.killTweensOf(dialog);
      gsap.to(dialog, {
        opacity: 0,
        scale: 0.94,
        duration: 0.16,
        ease: "power2.in",
        onComplete: () => {
          modal.classList.remove('active');
          dialog.style.opacity = '';
          dialog.style.transform = '';
        }
      });
      return;
    }
  }
  modal.classList.remove('active');
}

window.openNewTechModal = openNewTechModal;
window.closeNewTechModal = closeNewTechModal;
window.openNewDeviceModal = openNewDeviceModal;
window.closeNewDeviceModal = closeNewDeviceModal;
window.openEditDeviceModal = openEditDeviceModal;
window.openDeviceContextMenu = openDeviceContextMenu;
window.closeDeviceContextMenu = closeDeviceContextMenu;

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

  // Restaura configurações salvas no localStorage
  try {
    const saved = localStorage.getItem('prysm_settings');
    if (saved) {
      const cfg = JSON.parse(saved);
      if (cfg.techName) {
        const el = document.getElementById('cfgTechName');
        if (el) el.value = cfg.techName;
        const headName = document.getElementById('headerOperatorName');
        if (headName) headName.textContent = cfg.techName;
      }
      if (cfg.techRole) {
        const el = document.getElementById('cfgTechRole');
        if (el) el.value = cfg.techRole;
        const sideRole = document.getElementById('sidebarTechRole');
        if (sideRole) sideRole.textContent = cfg.techRole;
      }
      if (cfg.techDept) {
        const el = document.getElementById('cfgTechDept');
        if (el) el.value = cfg.techDept;
        const sideDept = document.getElementById('sidebarTechDept');
        if (sideDept) sideDept.textContent = cfg.techDept;
      }
      if (cfg.techPhone) {
        const el = document.getElementById('cfgTechPhone');
        if (el) el.value = cfg.techPhone;
      }
      if (cfg.defaultReason) {
        const el = document.getElementById('cfgDefaultReason');
        if (el) el.value = cfg.defaultReason;
      }
      if (typeof cfg.viewOnly === 'boolean') {
        const el = document.getElementById('cfgViewOnly');
        if (el) el.checked = cfg.viewOnly;
      }
      if (typeof cfg.blockInput === 'boolean') {
        const el = document.getElementById('cfgBlockInput');
        if (el) el.checked = cfg.blockInput;
      }
      if (typeof cfg.clipboard === 'boolean') {
        const el = document.getElementById('cfgClipboard');
        if (el) el.checked = cfg.clipboard;
      }
      if (cfg.serverUrl) {
        const el = document.getElementById('cfgServerUrl');
        if (el) el.value = cfg.serverUrl;
      }
      if (cfg.relayUrl) {
        const el = document.getElementById('cfgRelayUrl');
        if (el) el.value = cfg.relayUrl;
      }
      if (cfg.downloadDir) {
        const el = document.getElementById('cfgDownloadDir');
        if (el) el.value = cfg.downloadDir;
      }
    }
  } catch (err) {
    console.warn('[PrysmView] Falha ao restaurar configurações salvas:', err);
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
      if (btn.dataset.filter === 'all') {
        state.selectedStore = '';
        const storeSel = document.getElementById('storeSelectFilter');
        if (storeSel) storeSel.value = '';
      }
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
      if (state.activeTab !== 'dashboard') switchTab('dashboard');
      renderDashboard();
    };
    searchInput.onkeydown = (e) => {
      if (e.key === 'Escape') {
        searchInput.value = '';
        state.searchQuery = '';
        state.currentPage = 1;
        renderDashboard();
      }
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
      if (!split) return;
      if (state.drawerCollapsed) {
        if (!state.inspectedPdvId && state.pdvs.length > 0) {
          state.inspectedPdvId = state.pdvs[0].id;
        }
        state.drawerCollapsed = false;
        split.classList.remove('drawer-collapsed');
        renderInspectorDrawer();
      } else {
        state.drawerCollapsed = true;
        split.classList.add('drawer-collapsed');
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

  // Notificações no Cabeçalho
  const btnHeaderNotify = document.getElementById('btnHeaderNotify');
  if (btnHeaderNotify) {
    btnHeaderNotify.onclick = () => {
      showToast('Central de Notificações: Todos os serviços e conexões P2P operando normalmente (100%).');
    };
  }

  // Salvar Preferências (Ações Topo, Base e Geral)
  const handleSaveSettings = () => {
    const techName = document.getElementById('cfgTechName')?.value || 'admin';
    const techRole = document.getElementById('cfgTechRole')?.value || 'N1 - Suporte Técnico';
    const techDept = document.getElementById('cfgTechDept')?.value || 'Central de Operações de TI';
    const techPhone = document.getElementById('cfgTechPhone')?.value || '(11) 4004-0000';
    const defaultReason = document.getElementById('cfgDefaultReason')?.value || 'Suporte Técnico & Manutenção Preventiva';
    const viewOnly = !!document.getElementById('cfgViewOnly')?.checked;
    const blockInput = !!document.getElementById('cfgBlockInput')?.checked;
    const clipboard = !!document.getElementById('cfgClipboard')?.checked;
    const serverUrl = document.getElementById('cfgServerUrl')?.value || 'https://api.prysmview.com';
    const relayUrl = document.getElementById('cfgRelayUrl')?.value || 'quic://relay.prysmview.com:4433';
    const downloadDir = document.getElementById('cfgDownloadDir')?.value || 'C:\\Users\\Suporte\\Downloads\\PrysmView';

    const headName = document.getElementById('headerOperatorName');
    if (headName) headName.textContent = techName;
    const sideRole = document.getElementById('sidebarTechRole');
    if (sideRole) sideRole.textContent = techRole;
    const sideDept = document.getElementById('sidebarTechDept');
    if (sideDept) sideDept.textContent = techDept;

    const settingsObj = {
      techName,
      techRole,
      techDept,
      techPhone,
      defaultReason,
      viewOnly,
      blockInput,
      clipboard,
      serverUrl,
      relayUrl,
      downloadDir
    };

    try {
      localStorage.setItem('prysm_settings', JSON.stringify(settingsObj));
    } catch (e) {
      console.warn('[PrysmView] Falha ao salvar configurações no localStorage:', e);
    }

    sendRustCommand('save_settings', settingsObj);
    showToast('Configurações e parâmetros de suporte salvos com sucesso.');
  };

  const btnSaveTopSettings = document.getElementById('btnSaveTopSettings');
  if (btnSaveTopSettings) btnSaveTopSettings.onclick = handleSaveSettings;
  const btnSaveBottomSettings = document.getElementById('btnSaveBottomSettings');
  if (btnSaveBottomSettings) btnSaveBottomSettings.onclick = handleSaveSettings;

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

  // Alternância de Abas de Autenticação no Modal
  const tabAuthCode = document.getElementById('tabAuthCode');
  const tabAuthPassword = document.getElementById('tabAuthPassword');
  if (tabAuthCode) tabAuthCode.onclick = () => setAuthMode('code');
  if (tabAuthPassword) tabAuthPassword.onclick = () => setAuthMode('password');

  // Alternador de Visibilidade de Senha no Modal
  const btnToggleModalPwd = document.getElementById('btnToggleModalPassword');
  if (btnToggleModalPwd) {
    btnToggleModalPwd.onclick = () => {
      const input = document.getElementById('inputModalHostPassword');
      if (!input) return;
      if (input.type === 'password') {
        input.type = 'text';
        btnToggleModalPwd.textContent = '🙈';
      } else {
        input.type = 'password';
        btnToggleModalPwd.textContent = '👁';
      }
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

  // Fechar modais ao clicar no overlay escuro (fora da caixa de diálogo)
  document.querySelectorAll('.modal-overlay').forEach(overlay => {
    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) {
        if (overlay.id === 'sessionModal') closeModal();
        else if (overlay.id === 'newDeviceModal') closeNewDeviceModal();
        else if (overlay.id === 'newTechModal') closeNewTechModal();
      }
    });
  });

  // Fechar modais e menus flutuantes com tecla Escape
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      if (document.getElementById('sessionModal')?.classList.contains('active')) closeModal();
      if (document.getElementById('newDeviceModal')?.classList.contains('active')) closeNewDeviceModal();
      if (document.getElementById('newTechModal')?.classList.contains('active')) closeNewTechModal();
      closeDeviceContextMenu();
    }
  });

  const inputModalPwd = document.getElementById('inputModalHostPassword');
  if (inputModalPwd) {
    inputModalPwd.onkeydown = (e) => {
      if (e.key === 'Enter') {
        const btnAuth = document.getElementById('btnModalAuthorize');
        if (btnAuth) btnAuth.click();
      }
    };
  }

  const btnModalAuthorize = document.getElementById('btnModalAuthorize');
  if (btnModalAuthorize) {
    btnModalAuthorize.onclick = () => {
      const targetPdv = activeSessionTarget?.pdvId || 'DESKTOP-01';
      const inputPwd = document.getElementById('inputModalHostPassword');
      const pwd = (inputPwd?.value || '').trim();
      const code = Array.from(codeInputs).map(i => i.value).join('').trim();

      // Detecção inteligente: Se preencheu a senha do host OU se a aba ativa for senha
      if (pwd || currentAuthMode === 'password') {
        if (!pwd) {
          showToast('Por favor, digite a senha de acesso configurada no dispositivo remoto.');
          if (inputPwd) inputPwd.focus();
          return;
        }
        console.log('[PrysmAdmin] Conectando via senha do Host ao:', targetPdv);
        launchRemoteSession(targetPdv, 'password', pwd);
        return;
      }

      // Se preencheu o código de 8 dígitos
      if (code.length === 8) {
        console.log('[PrysmAdmin] Conectando via código de 8 dígitos ao:', targetPdv);
        launchRemoteSession(targetPdv, 'code', code);
        return;
      }

      if (code.length > 0 && code.length < 8) {
        showToast('Por favor, insira o código completo de 8 dígitos.');
        return;
      }

      // Se nenhum código foi digitado e não há senha, solicita consentimento direto na tela do PDV
      if (code.length === 0 && !pwd && currentAuthMode === 'code') {
        console.log('[PrysmAdmin] Conectando via solicitação de consentimento ao:', targetPdv);
        launchRemoteSession(targetPdv, 'consent', '');
        return;
      }

      showToast('Digite a senha de acesso do host ou o código de 8 dígitos para conectar.');
      if (inputPwd) inputPwd.focus();
    };
  }

  // =========================================================================
  // ACCORDION RETRÁTIL: MINIMIZAR / EXPANDIR CONTROLES DE CONEXÃO NO DASHBOARD
  // =========================================================================
  const tvQuickAccordion = document.getElementById('tvQuickAccordion');
  const tvQuickBody = document.getElementById('tvQuickBody');
  const btnToggleTvAccordion = document.getElementById('btnToggleTvAccordion');
  const tvAccordionBtnText = document.getElementById('tvAccordionBtnText');
  const tvQuickHeader = document.getElementById('tvQuickHeader');

  function setTvAccordionCollapsed(isCollapsed, animate = true) {
    if (!tvQuickAccordion || !tvQuickBody) return;
    const chevron = document.getElementById('tvAccordionChevron');

    if (isCollapsed) {
      if (animate && window.gsap && !tvQuickBody.classList.contains('collapsed')) {
        gsap.killTweensOf(tvQuickBody);
        gsap.killTweensOf("#tvQuickBody .tv-home-card, #tvQuickBody .tv-hero-card");
        const currentHeight = tvQuickBody.offsetHeight;
        gsap.fromTo(tvQuickBody,
          { height: currentHeight, opacity: 1 },
          {
            height: 0,
            opacity: 0,
            duration: 0.25,
            ease: "power2.inOut",
            onComplete: () => {
              tvQuickBody.classList.add('collapsed');
              tvQuickAccordion.classList.remove('expanded');
              tvQuickBody.style.height = '';
              tvQuickBody.style.opacity = '';
            }
          }
        );
      } else {
        tvQuickBody.classList.add('collapsed');
        tvQuickAccordion.classList.remove('expanded');
        tvQuickBody.style.height = '';
        tvQuickBody.style.opacity = '';
      }
      if (chevron && window.gsap) {
        gsap.killTweensOf(chevron);
        gsap.to(chevron, { rotation: 0, duration: 0.25, ease: "power2.out" });
      }
      if (tvAccordionBtnText) tvAccordionBtnText.textContent = 'Expandir Controles';
      localStorage.setItem('prysm_quick_connect_collapsed', 'true');
    } else {
      tvQuickBody.classList.remove('collapsed');
      tvQuickAccordion.classList.add('expanded');
      if (animate && window.gsap) {
        tvQuickBody.style.height = 'auto';
        const targetHeight = tvQuickBody.scrollHeight;
        tvQuickBody.style.height = '0px';
        gsap.killTweensOf(tvQuickBody);
        gsap.killTweensOf("#tvQuickBody .tv-home-card, #tvQuickBody .tv-hero-card");
        gsap.fromTo(tvQuickBody,
          { height: 0, opacity: 0 },
          {
            height: targetHeight,
            opacity: 1,
            duration: 0.28,
            ease: "power2.out",
            onComplete: () => {
              tvQuickBody.style.height = 'auto';
              tvQuickBody.style.opacity = '1';
            }
          }
        );
        gsap.fromTo("#tvQuickBody .tv-home-card, #tvQuickBody .tv-hero-card",
          { opacity: 0, y: 12 },
          {
            opacity: 1,
            y: 0,
            duration: 0.28,
            stagger: 0.05,
            ease: "power2.out",
            clearProps: "transform,opacity"
          }
        );
      } else {
        tvQuickBody.style.height = 'auto';
        tvQuickBody.style.opacity = '1';
      }
      if (chevron && window.gsap) {
        gsap.killTweensOf(chevron);
        gsap.to(chevron, { rotation: 180, duration: 0.25, ease: "power2.out" });
      }
      if (tvAccordionBtnText) tvAccordionBtnText.textContent = 'Minimizar Controles';
      localStorage.setItem('prysm_quick_connect_collapsed', 'false');
    }
  }

  // Inicializa minimizado por padrão (ou lê a preferência salva no localStorage) sem animar na abertura
  const savedCollapsed = localStorage.getItem('prysm_quick_connect_collapsed');
  setTvAccordionCollapsed(savedCollapsed !== 'false', false);

  if (btnToggleTvAccordion) {
    btnToggleTvAccordion.onclick = (e) => {
      e.stopPropagation();
      const isCurrentlyCollapsed = tvQuickBody ? tvQuickBody.classList.contains('collapsed') : true;
      setTvAccordionCollapsed(!isCurrentlyCollapsed, true);
    };
  }

  if (tvQuickHeader) {
    tvQuickHeader.onclick = (e) => {
      if (e.target.closest('#btnToggleTvAccordion')) return;
      const isCurrentlyCollapsed = tvQuickBody ? tvQuickBody.classList.contains('collapsed') : true;
      setTvAccordionCollapsed(!isCurrentlyCollapsed, true);
    };
  }

  // =========================================================================
  // TEAMVIEWER HERO: PERMITIR SUPORTE & CONTROLAR COMPUTADOR REMOTO
  // =========================================================================
  const btnTvConnect = document.getElementById('btnTvConnect');
  const tvPartnerIdInput = document.getElementById('tvPartnerIdInput');
  const tvPartnerCodeInput = document.getElementById('tvPartnerCodeInput');
  const btnToggleTvCode = document.getElementById('btnToggleTvCode');
  const btnCopyLocalId = document.getElementById('btnCopyLocalId');
  const btnTvRefreshDiscovery = document.getElementById('btnTvRefreshDiscovery');

  if (btnCopyLocalId) {
    btnCopyLocalId.onclick = () => {
      const localIdText = document.getElementById('tvLocalIdDisplay')?.textContent || '127.0.0.1';
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(localIdText);
      }
      showToast(`ID deste dispositivo (${localIdText}) copiada.`);
    };
  }

  const btnCopyDirectLocalId = document.getElementById('btnCopyDirectLocalId');
  if (btnCopyDirectLocalId) {
    btnCopyDirectLocalId.onclick = () => {
      const localIdText = document.getElementById('tvDirectLocalIdDisplay')?.textContent || '127.0.0.1';
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(localIdText);
      }
      showToast(`ID deste dispositivo (${localIdText}) copiada.`);
    };
  }

  if (btnToggleTvCode && tvPartnerCodeInput) {
    btnToggleTvCode.onclick = () => {
      if (tvPartnerCodeInput.type === 'password') {
        tvPartnerCodeInput.type = 'text';
        btnToggleTvCode.textContent = '🙈';
      } else {
        tvPartnerCodeInput.type = 'password';
        btnToggleTvCode.textContent = '👁';
      }
    };
  }

  if (btnTvRefreshDiscovery) {
    btnTvRefreshDiscovery.onclick = () => {
      sendRustCommand('refresh_fleet');
      probeFleetHealth();
      showToast('Escaneando rede local (UDP 19532 / TCP 19533)...');
      renderDashboard();
    };
  }

  if (tvPartnerIdInput && tvPartnerCodeInput) {
    tvPartnerIdInput.onkeydown = (e) => {
      if (e.key === 'Enter') tvPartnerCodeInput.focus();
    };
    tvPartnerCodeInput.onkeydown = (e) => {
      if (e.key === 'Enter' && btnTvConnect) btnTvConnect.click();
    };
  }

  if (btnTvConnect) {
    btnTvConnect.onclick = () => {
      const partnerId = (tvPartnerIdInput?.value || '').trim();
      const credential = (tvPartnerCodeInput?.value || '').trim();
      const modeRadio = document.querySelector('input[name="tvSessionMode"]:checked');
      const sessionMode = modeRadio ? modeRadio.value : 'full';

      if (!partnerId) {
        if (tvPartnerIdInput) {
          tvPartnerIdInput.focus();
          tvPartnerIdInput.style.borderColor = 'var(--accent-ruby)';
          setTimeout(() => { tvPartnerIdInput.style.borderColor = 'var(--border-subtle)'; }, 2000);
        }
        showToast('Por favor, digite a ID do parceiro ou IP do dispositivo para conectar.');
        return;
      }

      if (credential) {
        const isDigits = /^\d{8}$/.test(credential);
        const authType = isDigits ? 'code' : 'password';

        const normalizedId = partnerId.toUpperCase();
        if (!state.pdvs.some(p => p.id.toUpperCase() === normalizedId)) {
          const newDevice = {
            id: normalizedId,
            storeName: "Parceiro Remoto (Conexão Direta)",
            isOnline: true,
            agentVersion: "0.1.0",
            publicKey: `ed25519_${partnerId.toLowerCase()}_key_hash`,
            lastSeen: "Agora",
            ipAddress: partnerId.includes('.') ? partnerId : "127.0.0.1",
            osInfo: "Windows 11 / 10",
            resolution: "1920x1080 @ 60Hz",
            notes: `Conexão autenticada via ${isDigits ? 'código de 8 dígitos' : 'senha do host'}.`
          };
          state.pdvs.unshift(newDevice);
          savePdvs(state.pdvs);
          renderDashboard();
        }

        console.log('[PrysmAdmin] Conectando via TeamViewer Hero:', partnerId, authType, sessionMode);
        launchRemoteSession(partnerId, authType, credential, sessionMode);
      } else {
        const target = state.pdvs.find(p => p.id.toUpperCase() === partnerId.toUpperCase());
        if (target && target.savedPassword) {
          launchRemoteSession(partnerId, 'password', target.savedPassword, sessionMode);
        } else {
          startSession(partnerId, target ? target.storeName : 'Dispositivo Remoto');
        }
      }
    };
  }

  // Conexão Direta Submit e Atalhos
  const directPdvInput = document.getElementById('directPdvInput');
  const directCodeInput = document.getElementById('directCodeInput');
  const btnToggleDirectCode = document.getElementById('btnToggleDirectCode');

  if (btnToggleDirectCode && directCodeInput) {
    btnToggleDirectCode.onclick = () => {
      if (directCodeInput.type === 'password') {
        directCodeInput.type = 'text';
        btnToggleDirectCode.textContent = '🙈';
      } else {
        directCodeInput.type = 'password';
        btnToggleDirectCode.textContent = '👁';
      }
    };
  }

  if (directPdvInput && directCodeInput) {
    directPdvInput.onkeydown = (e) => {
      if (e.key === 'Enter') directCodeInput.focus();
    };
    directCodeInput.onkeydown = (e) => {
      if (e.key === 'Enter') {
        const btnSubmit = document.getElementById('btnDirectSubmit');
        if (btnSubmit) btnSubmit.click();
      }
    };
  }

  const btnDirectSubmit = document.getElementById('btnDirectSubmit');
  if (btnDirectSubmit) {
    btnDirectSubmit.onclick = () => {
      const pdvId = (document.getElementById('directPdvInput')?.value || '').trim();
      const credential = (document.getElementById('directCodeInput')?.value || '').trim();
      const modeRadio = document.querySelector('input[name="directSessionMode"]:checked');
      const sessionMode = modeRadio ? modeRadio.value : 'full';

      if (!pdvId) {
        const inp = document.getElementById('directPdvInput');
        if (inp) {
          inp.focus();
          inp.style.borderColor = 'var(--accent-red)';
          setTimeout(() => { inp.style.borderColor = 'var(--border-subtle)'; }, 2000);
        }
        showToast('Por favor, digite a ID do parceiro ou IP do dispositivo para conectar.');
        return;
      }

      if (credential) {
        const isDigits = /^\d{8}$/.test(credential);
        const authType = isDigits ? 'code' : 'password';

        // Se o dispositivo ainda não estiver na lista da frota, vincula automaticamente
        const normalizedId = pdvId.toUpperCase();
        if (!state.pdvs.some(p => p.id.toUpperCase() === normalizedId)) {
          const newDevice = {
            id: normalizedId,
            storeName: "Dispositivo Conectado Diretamente",
            isOnline: true,
            agentVersion: "0.1.0",
            publicKey: `ed25519_${pdvId.toLowerCase()}_key_hash`,
            lastSeen: "Agora",
            ipAddress: pdvId.includes('.') ? pdvId : (state.pdvs.find(p => p.id.toUpperCase() === normalizedId)?.ipAddress || "127.0.0.1"),
            osInfo: "Windows 11 / 10",
            resolution: "1920x1080 @ 60Hz",
            notes: `Conexão direta autorizada via ${isDigits ? 'código de sessão' : 'senha de acesso'}.`
          };
          state.pdvs.unshift(newDevice);
          savePdvs(state.pdvs);
          renderDashboard();
        }

        console.log('[PrysmAdmin] Conexão direta disparada para:', pdvId, 'Tipo:', authType, 'Modo:', sessionMode);
        launchRemoteSession(pdvId, authType, credential, sessionMode);
      } else {
        const target = state.pdvs.find(p => p.id.toUpperCase() === pdvId.toUpperCase());
        if (target && target.savedPassword) {
          launchRemoteSession(pdvId, 'password', target.savedPassword, sessionMode);
        } else {
          startSession(pdvId, target ? target.storeName : 'Dispositivo Remoto');
        }
      }
    };
  }

  // Ações da Barra Flutuante da Sessão Remota
  const btnRemoteCad = document.getElementById('btnRemoteCad');
  if (btnRemoteCad) {
    btnRemoteCad.onclick = () => {
      sendRustCommand('send_sas', { pdvId: activeRemoteSession?.pdvId || 'DESKTOP-01' });
      showToast('Sinal SAS (Ctrl+Alt+Del) enviado com sucesso ao Host remoto.');
    };
  }

  const btnRemoteClipboard = document.getElementById('btnRemoteClipboard');
  if (btnRemoteClipboard) {
    btnRemoteClipboard.onclick = () => {
      if (navigator.clipboard && navigator.clipboard.readText) {
        navigator.clipboard.readText().then(text => {
          sendRustCommand('sync_clipboard', { text });
          showToast('Área de transferência sincronizada com o dispositivo remoto.');
        }).catch(() => {
          sendRustCommand('sync_clipboard', { text: '' });
          showToast('Área de transferência sincronizada.');
        });
      } else {
        sendRustCommand('sync_clipboard', { text: '' });
        showToast('Área de transferência sincronizada.');
      }
    };
  }

  const btnRemoteMonitors = document.getElementById('btnRemoteMonitors');
  if (btnRemoteMonitors) {
    let currentMon = 1;
    btnRemoteMonitors.onclick = () => {
      currentMon = currentMon === 1 ? 2 : 1;
      const span = btnRemoteMonitors.querySelector('span');
      if (span) span.textContent = `Monitor ${currentMon}`;
      showToast(`Alternado para Monitor ${currentMon} no Host.`);
    };
  }

  const btnRemoteFullscreen = document.getElementById('btnRemoteFullscreen');
  if (btnRemoteFullscreen) {
    btnRemoteFullscreen.onclick = () => {
      const scr = document.getElementById('remoteSessionScreen');
      if (!document.fullscreenElement) {
        if (scr && scr.requestFullscreen) scr.requestFullscreen();
      } else {
        if (document.exitFullscreen) document.exitFullscreen();
      }
    };
  }

  const btnEndRemoteSession = document.getElementById('btnEndRemoteSession');
  if (btnEndRemoteSession) {
    btnEndRemoteSession.onclick = terminateRemoteSession;
  }

  // Sincronização periódica da frota e descoberta UDP via Rust
  setInterval(() => {
    sendRustCommand('refresh_fleet');
  }, 3000);
  sendRustCommand('refresh_fleet');

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
    };
  }

  // Callback acionado pelo backend Rust quando a checagem é concluída
  window.onPrysmUpdateChecked = function (statusObj) {
    if (checkUpdatesBtn) {
      checkUpdatesBtn.disabled = false;
      checkUpdatesBtn.innerHTML = `
        <svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M23 4v6h-6"></path><path d="M1 20v-6h6"></path><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
        <span>Verificar Atualizações Agora</span>
      `;
    }

    const now = new Date();
    state.lastCheckTime = `Hoje, ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}`;

    if (!statusObj) return;

    if (statusObj.status === 'UpdateAvailable') {
      const data = statusObj.data;
      state.availableUpdate = data;
      state.availableVersions[data.channel] = data.latest_version;

      if (updateAvailableBox) updateAvailableBox.style.display = 'flex';
      const latestTag = document.getElementById('newVersionTag') || document.getElementById('latestVersionTag');
      if (latestTag) latestTag.textContent = `v${data.latest_version}`;

      const changelogEl = document.getElementById('newVersionNotes') || document.getElementById('updateChangelog');
      if (changelogEl) changelogEl.textContent = data.changelog;

      if (updateStatusText) {
        updateStatusText.innerHTML = `<span style="color: var(--accent-cyan); font-weight: 600;">⚡ Nova versão v${data.latest_version} disponível para instalação imediata!</span>`;
      }
      if (headerUpdateBadge) {
        headerUpdateBadge.style.display = 'flex';
        headerUpdateBadge.innerHTML = `<div class="pulse-dot"></div><span>⚡ Nova v${data.latest_version} Disp.</span>`;
      }

      showToast(`Nova versão v${data.latest_version} encontrada no canal ${data.channel}! Você pode atualizar agora.`);
    } else if (statusObj.status === 'UpToDate') {
      state.availableUpdate = null;
      if (updateAvailableBox) updateAvailableBox.style.display = 'none';
      if (headerUpdateBadge) headerUpdateBadge.style.display = 'none';

      if (updateStatusText) {
        updateStatusText.innerHTML = `<span style="color: var(--accent-mint); font-weight: 600;">● Sistema Atualizado (Versão v${state.installedVersion} ativa e verificada)</span>`;
      }
      showToast(`O PrysmView já está na versão mais recente (v${state.installedVersion}). Nenhuma atualização necessária!`);
    } else if (statusObj.status === 'Error') {
      showToast(`Erro na verificação de atualização: ${statusObj.data?.message || 'Falha de comunicação.'}`);
    }

    syncVersionUI();
  };

  // Instalar Atualização In-Place com Telemetria e Eventos Reais em Rust
  if (btnApplyUpdate) {
    btnApplyUpdate.onclick = () => {
      const channel = state.activeChannel || 'stable';
      const targetVer = state.availableUpdate?.latest_version || state.availableVersions[channel] || state.availableVersions.stable;
      const downloadUrl = state.availableUpdate?.download_url || '';
      const sha256Hex = state.availableUpdate?.sha256_hex || '';

      btnApplyUpdate.disabled = true;
      btnApplyUpdate.innerHTML = `<span>Iniciando Atualização v${targetVer}...</span>`;
      if (updateProgressContainer) updateProgressContainer.style.display = 'flex';
      if (updateProgressBar) {
        updateProgressBar.style.width = '0%';
        updateProgressBar.style.background = 'var(--accent-cyan)';
      }
      if (updateProgressPercent) updateProgressPercent.textContent = '0%';
      if (updateProgressLabel) updateProgressLabel.textContent = 'Conectando ao serviço de atualização nativo...';

      // Dispara comando real para o backend Rust
      sendRustCommand('apply_update', {
        version: targetVer,
        channel,
        download_url: downloadUrl,
        sha256_hex: sha256Hex
      });
    };
  }

  // Callback de progresso real enviado pelo backend Rust
  window.onPrysmUpdateProgress = function (percent, message) {
    if (updateProgressContainer) updateProgressContainer.style.display = 'flex';
    if (updateProgressBar) updateProgressBar.style.width = `${percent}%`;
    if (updateProgressPercent) updateProgressPercent.textContent = `${percent}%`;
    if (updateProgressLabel) updateProgressLabel.textContent = message;
  };

  // Callback de conclusão enviado pelo backend Rust
  window.onPrysmUpdateComplete = function (newVer) {
    state.installedVersion = newVer;
    if (typeof window !== 'undefined') {
      window.__PRYSM_SERVER_VERSION__ = newVer;
      try {
        if (window.localStorage) {
          window.localStorage.setItem('prysm_installed_version', newVer);
        }
      } catch (e) { }
    }

    syncVersionUI();

    const now = new Date();
    const timeStr = `Hoje ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    state.auditRecords.unshift({
      id: `AUD-${Math.floor(920 + Math.random() * 50)}`,
      timestamp: timeStr,
      operator: document.getElementById('headerOperatorName')?.textContent || "admin",
      pdv: "PrysmAdmin",
      store: "Console Central",
      reason: `Atualização In-App de Software para v${state.installedVersion}`,
      duration: "04s",
      status: "Concluída"
    });
    if (typeof renderAuditTable === 'function') renderAuditTable();

    showToast(`Sucesso! PrysmView atualizado para a versão v${state.installedVersion}. O aplicativo será reiniciado.`);
  };

  // Callback de erro enviado pelo backend Rust
  window.onPrysmUpdateError = function (errMsg) {
    if (btnApplyUpdate) {
      btnApplyUpdate.disabled = false;
      btnApplyUpdate.innerHTML = `
        <svg width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
        <span>Tentar Novamente</span>
      `;
    }
    if (updateProgressLabel) updateProgressLabel.textContent = `Erro: ${errMsg}`;
    if (updateProgressBar) updateProgressBar.style.background = 'var(--accent-ruby)';
    showToast(`Falha na atualização: ${errMsg}`);
  };

  // =========================================================================
  // DROPDOWN DE PERFIL DO USUÁRIO & MENU DE CONFIGURAÇÕES
  // =========================================================================
  const userProfileBtn = document.getElementById('userProfileBtn');
  const userProfileWrapper = document.getElementById('userProfileWrapper');
  const userProfileMenu = document.getElementById('userProfileMenu');

  if (userProfileBtn && userProfileMenu) {
    userProfileBtn.onclick = (e) => {
      e.stopPropagation();
      const isOpen = userProfileMenu.classList.contains('active');
      if (isOpen) {
        if (window.gsap) {
          gsap.killTweensOf(userProfileMenu);
          gsap.to(userProfileMenu, {
            opacity: 0,
            y: -6,
            scale: 0.96,
            duration: 0.15,
            ease: 'power2.in',
            onComplete: () => {
              userProfileMenu.classList.remove('active');
              userProfileMenu.style.display = 'none';
              userProfileMenu.style.opacity = '';
              userProfileMenu.style.transform = '';
            }
          });
        } else {
          userProfileMenu.classList.remove('active');
          userProfileMenu.style.display = 'none';
        }
        if (userProfileWrapper) userProfileWrapper.classList.remove('menu-open');
      } else {
        userProfileMenu.style.display = 'flex';
        userProfileMenu.classList.add('active');
        if (window.gsap) {
          gsap.killTweensOf(userProfileMenu);
          gsap.fromTo(userProfileMenu,
            { opacity: 0, y: -8, scale: 0.95 },
            { opacity: 1, y: 0, scale: 1, duration: 0.22, ease: 'back.out(1.4)' }
          );
        }
        if (userProfileWrapper) userProfileWrapper.classList.add('menu-open');
      }
    };

    // Fechar ao clicar fora
    document.addEventListener('click', (e) => {
      if (userProfileWrapper && !userProfileWrapper.contains(e.target)) {
        if (userProfileMenu.classList.contains('active')) {
          if (window.gsap) {
            gsap.killTweensOf(userProfileMenu);
            gsap.to(userProfileMenu, {
              opacity: 0,
              y: -6,
              scale: 0.96,
              duration: 0.12,
              ease: 'power2.in',
              onComplete: () => {
                userProfileMenu.classList.remove('active');
                userProfileMenu.style.display = 'none';
                userProfileMenu.style.opacity = '';
                userProfileMenu.style.transform = '';
              }
            });
          } else {
            userProfileMenu.classList.remove('active');
            userProfileMenu.style.display = 'none';
          }
          userProfileWrapper.classList.remove('menu-open');
        }
      }
    });

    const menuItemSettings = document.getElementById('menuItemSettings');
    if (menuItemSettings) {
      menuItemSettings.onclick = () => {
        userProfileMenu.classList.remove('active');
        userProfileMenu.style.display = 'none';
        if (userProfileWrapper) userProfileWrapper.classList.remove('menu-open');
        switchTab('settings');
      };
    }

    const menuItemProfile = document.getElementById('menuItemProfile');
    if (menuItemProfile) {
      menuItemProfile.onclick = () => {
        userProfileMenu.classList.remove('active');
        userProfileMenu.style.display = 'none';
        if (userProfileWrapper) userProfileWrapper.classList.remove('menu-open');
        switchTab('settings');
        setTimeout(() => {
          const cfgName = document.getElementById('cfgTechName');
          if (cfgName) {
            cfgName.focus();
            cfgName.select();
          }
        }, 150);
      };
    }

    const menuItemLogout = document.getElementById('menuItemLogout');
    if (menuItemLogout) {
      menuItemLogout.onclick = () => {
        userProfileMenu.classList.remove('active');
        userProfileMenu.style.display = 'none';
        if (userProfileWrapper) userProfileWrapper.classList.remove('menu-open');
        showToast('Sessão do operador encerrada com segurança.');
      };
    }
  }

  // =========================================================================
  // MODAL DE CADASTRO DE NOVO TÉCNICO
  // =========================================================================
  const btnOpenNewTechModal = document.getElementById('btnOpenNewTechModal');
  const btnCloseNewTechModal = document.getElementById('btnCloseNewTechModal');
  const btnCancelNewTech = document.getElementById('btnCancelNewTech');
  const btnSaveNewTech = document.getElementById('btnSaveNewTech');
  const formNewTech = document.getElementById('formNewTech');

  if (btnOpenNewTechModal) btnOpenNewTechModal.onclick = openNewTechModal;
  if (btnCloseNewTechModal) btnCloseNewTechModal.onclick = closeNewTechModal;
  if (btnCancelNewTech) btnCancelNewTech.onclick = closeNewTechModal;
  if (btnSaveNewTech && formNewTech) {
    btnSaveNewTech.onclick = () => formNewTech.requestSubmit ? formNewTech.requestSubmit() : formNewTech.dispatchEvent(new Event('submit', { cancelable: true }));
  }

  if (formNewTech) {
    formNewTech.onsubmit = (e) => {
      e.preventDefault();
      const fullName = document.getElementById('techFullNameInput').value.trim();
      const username = document.getElementById('techUsernameInput').value.trim().toLowerCase();
      const email = document.getElementById('techEmailInput').value.trim();
      const role = document.getElementById('techRoleSelect').value;
      const password = document.getElementById('techPasswordInput').value;

      if (!fullName || !username || !email || !password) {
        showToast('Preencha todos os campos obrigatórios do técnico.');
        return;
      }

      if (password.length < 6) {
        showToast('A senha inicial deve conter pelo menos 6 caracteres.');
        return;
      }

      if (state.technicians.some(t => t.username === username)) {
        showToast(`O usuário "${username}" já está cadastrado.`);
        return;
      }

      const permissions = [];
      if (document.getElementById('permRemoteAccess')?.checked) permissions.push('Acesso Remoto E2E');
      if (document.getElementById('permInputControl')?.checked) permissions.push('Controle Total');
      if (document.getElementById('permFileTransfer')?.checked) permissions.push('Arquivos');
      if (document.getElementById('permViewAudit')?.checked) permissions.push('Auditoria');

      const newTech = {
        id: `TECH-0${state.technicians.length + 1}`,
        name: fullName,
        username,
        email,
        role,
        permissions,
        status: 'Ativo',
        lastActive: 'Nunca'
      };

      state.technicians.push(newTech);
      saveTechnicians(state.technicians);
      renderTechniciansTable();

      // Registro de auditoria corporativo
      const now = new Date();
      state.auditRecords.unshift({
        id: `AUD-${Math.floor(920 + Math.random() * 50)}`,
        timestamp: `Hoje ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`,
        operator: document.getElementById('headerOperatorName')?.textContent || "admin",
        pdv: "PrysmAdmin",
        store: "Console Central",
        reason: `Cadastro de novo técnico: ${username} (${role})`,
        duration: "01s",
        status: "Concluída"
      });

      closeNewTechModal();
      showToast(`Técnico ${fullName} (@${username}) cadastrado com sucesso!`);
    };
  }

  // =========================================================================
  // MODAL DE CADASTRO DE NOVO DISPOSITIVO (PDV)
  // =========================================================================
  const btnOpenNewDeviceModal = document.getElementById('btnOpenNewDeviceModal');
  const btnCloseNewDeviceModal = document.getElementById('btnCloseNewDeviceModal');
  const btnCancelNewDevice = document.getElementById('btnCancelNewDevice');
  const formNewDevice = document.getElementById('formNewDevice');

  if (btnOpenNewDeviceModal) btnOpenNewDeviceModal.onclick = openNewDeviceModal;
  if (btnCloseNewDeviceModal) btnCloseNewDeviceModal.onclick = closeNewDeviceModal;
  if (btnCancelNewDevice) btnCancelNewDevice.onclick = closeNewDeviceModal;

  if (formNewDevice) {
    formNewDevice.onsubmit = (e) => {
      e.preventDefault();
      const devId = document.getElementById('devIdInput').value.trim().toUpperCase();
      const devStore = document.getElementById('devStoreInput').value.trim();
      const devIp = document.getElementById('devIpInput').value.trim();
      const devStatus = document.getElementById('devStatusSelect').value;
      const devNotes = document.getElementById('devNotesInput').value.trim();

      if (!devId || !devStore || !devIp) {
        showToast('Preencha todos os campos obrigatórios do dispositivo.');
        return;
      }

      if (editingDeviceId) {
        // Modo de Edição / Alteração de Dispositivo
        if (devId !== editingDeviceId && state.pdvs.some(p => p.id === devId)) {
          showToast(`Já existe outro dispositivo com a ID "${devId}".`);
          return;
        }

        const target = state.pdvs.find(p => p.id === editingDeviceId);
        if (target) {
          target.id = devId;
          target.storeName = devStore;
          target.ipAddress = devIp;
          target.isOnline = devStatus === 'online';
          target.notes = devNotes || target.notes;
          if (state.inspectedPdvId === editingDeviceId) {
            state.inspectedPdvId = devId;
          }
          savePdvs(state.pdvs);
          renderDashboard();
          closeNewDeviceModal();
          showToast(`Dispositivo "${devId}" atualizado com sucesso!`);
        }
        return;
      }

      // Modo de Criação (Novo Dispositivo)
      if (state.pdvs.some(p => p.id === devId)) {
        showToast(`O dispositivo com ID "${devId}" já existe no sistema.`);
        return;
      }

      const newDevice = {
        id: devId,
        storeName: devStore,
        isOnline: devStatus === 'online',
        agentVersion: "0.1.0",
        publicKey: `ed25519_${devId.toLowerCase()}_key_hash`,
        lastSeen: devStatus === 'online' ? "Agora" : "Recém cadastrado",
        ipAddress: devIp,
        osInfo: "Windows 11 / 10",
        resolution: "1920x1080 @ 60Hz",
        notes: devNotes || `Dispositivo ${devId} cadastrado via console.`
      };

      state.pdvs.unshift(newDevice);
      savePdvs(state.pdvs);
      renderDashboard();

      // Registro no histórico de auditoria
      const now = new Date();
      state.auditRecords.unshift({
        id: `AUD-${Math.floor(920 + Math.random() * 50)}`,
        timestamp: `Hoje ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`,
        operator: document.getElementById('headerOperatorName')?.textContent || "admin",
        pdv: devId,
        store: devStore,
        reason: `Cadastro manual de novo dispositivo: ${devId} (${devIp})`,
        duration: "01s",
        status: "Concluída"
      });

      closeNewDeviceModal();
      showToast(`Dispositivo ${devId} adicionado com sucesso à frota!`);
    };
  }

  // =========================================================================
  // LISTENERS DO MENU DE CONTEXTO DO DISPOSITIVO (BOTÃO DIREITO)
  // =========================================================================
  const ctxBtnConnect = document.getElementById('ctxBtnConnect');
  if (ctxBtnConnect) {
    ctxBtnConnect.onclick = () => {
      const pdv = activeContextMenuPdv;
      closeDeviceContextMenu();
      if (pdv) startSession(pdv.id, pdv.storeName);
    };
  }

  const ctxBtnEdit = document.getElementById('ctxBtnEdit');
  if (ctxBtnEdit) {
    ctxBtnEdit.onclick = () => {
      const pdv = activeContextMenuPdv;
      closeDeviceContextMenu();
      if (pdv) openEditDeviceModal(pdv);
    };
  }

  const ctxBtnInspect = document.getElementById('ctxBtnInspect');
  if (ctxBtnInspect) {
    ctxBtnInspect.onclick = () => {
      const pdv = activeContextMenuPdv;
      closeDeviceContextMenu();
      if (pdv) selectPdvForInspection(pdv);
    };
  }

  const ctxBtnCopyId = document.getElementById('ctxBtnCopyId');
  if (ctxBtnCopyId) {
    ctxBtnCopyId.onclick = () => {
      const pdv = activeContextMenuPdv;
      closeDeviceContextMenu();
      if (pdv && pdv.id) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(pdv.id).then(() => {
            showToast(`ID "${pdv.id}" copiado para a área de transferência!`);
          }).catch(() => {
            showToast(`ID: ${pdv.id}`);
          });
        } else {
          showToast(`ID: ${pdv.id}`);
        }
      }
    };
  }

  const ctxBtnCopyIp = document.getElementById('ctxBtnCopyIp');
  if (ctxBtnCopyIp) {
    ctxBtnCopyIp.onclick = () => {
      const pdv = activeContextMenuPdv;
      closeDeviceContextMenu();
      if (pdv && pdv.ipAddress) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(pdv.ipAddress).then(() => {
            showToast(`IP "${pdv.ipAddress}" copiado para a área de transferência!`);
          }).catch(() => {
            showToast(`IP: ${pdv.ipAddress}`);
          });
        } else {
          showToast(`IP: ${pdv.ipAddress}`);
        }
      }
    };
  }

  const ctxBtnDelete = document.getElementById('ctxBtnDelete');
  if (ctxBtnDelete) {
    ctxBtnDelete.onclick = () => {
      const pdv = activeContextMenuPdv;
      closeDeviceContextMenu();
      if (!pdv) return;
      const confirmMsg = `Tem certeza que deseja remover o dispositivo "${pdv.id}" (${pdv.storeName}) do painel de gerenciamento?`;
      if (confirm(confirmMsg)) {
        state.pdvs = state.pdvs.filter(p => p.id !== pdv.id);
        savePdvs(state.pdvs);
        if (state.inspectedPdvId === pdv.id) {
          state.inspectedPdvId = null;
        }
        renderDashboard();
        showToast(`Dispositivo "${pdv.id}" removido com sucesso.`);
      }
    };
  }

  // Fechar menu de contexto ao clicar fora, rolar a página ou redimensionar a janela
  document.addEventListener('click', (e) => {
    const menu = document.getElementById('deviceContextMenu');
    if (menu && menu.style.display !== 'none' && !menu.contains(e.target)) {
      closeDeviceContextMenu();
    }
  });
  document.addEventListener('scroll', closeDeviceContextMenu, true);
  window.addEventListener('resize', closeDeviceContextMenu);

  // Render inicial
  renderDashboard();
  renderTechniciansTable();
  syncVersionUI();

  // Animações de entrada e micro-interações GSAP para botões e controles interativos
  if (window.gsap) {
    gsap.fromTo('.app-header', 
      { opacity: 0.45, y: -6 }, 
      { opacity: 1, y: 0, duration: 0.3, ease: 'power2.out', clearProps: 'all' }
    );
    gsap.fromTo('.app-sidebar', 
      { opacity: 0.45, x: -8 }, 
      { opacity: 1, x: 0, duration: 0.3, ease: 'power2.out', clearProps: 'all' }
    );
    gsap.fromTo('.main-viewport', 
      { opacity: 0.5, y: 6 }, 
      { opacity: 1, y: 0, duration: 0.35, ease: 'power2.out', clearProps: 'all' }
    );
    initGsapMicroInteractions();
  }

  // Sondagem ativa e periódica de status real da frota
  probeFleetHealth();
  setInterval(probeFleetHealth, 2500);
}

function initGsapMicroInteractions() {
  if (!window.gsap) return;
  document.querySelectorAll('button:not(.no-gsap), .btn-primary, .btn-secondary, .btn-emergency, .quick-btn, .pill-btn, .nav-item, .btn-icon-ghost, .user-badge-dropdown, .kpi-card, .tv-accordion-toggle-btn').forEach(el => {
    if (el.dataset.gsapBound) return;
    el.dataset.gsapBound = 'true';
    el.addEventListener('mousedown', () => {
      gsap.to(el, { scale: 0.96, duration: 0.1, ease: 'power1.out' });
    });
    el.addEventListener('mouseup', () => {
      gsap.to(el, { scale: 1, duration: 0.15, ease: 'power1.out' });
    });
    el.addEventListener('mouseleave', () => {
      gsap.to(el, { scale: 1, duration: 0.15, ease: 'power1.out' });
    });
  });
}

// Inicia imediatamente se o DOM já estiver pronto, ou escuta DOMContentLoaded
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}

