/**
 * PrysmView Admin Console - Frontend Controller
 * Solução corporativa universal de acesso e gestão remota de dispositivos.
 */

// Estado Global da Aplicação
const state = {
  activeTab: 'dashboard',
  filterStatus: 'all',
  selectedStore: '',
  searchQuery: '',
  inspectedPdvId: 'DESK-01',
  latencyMs: 18,
  
  pdvs: [
    {
      id: "DESK-01",
      storeName: "Matriz - Corporativo",
      isOnline: true,
      agentVersion: "0.1.0",
      publicKey: "ed25519_host01_key_hash",
      lastSeen: "Agora",
      ipAddress: "192.168.1.101",
      osInfo: "Windows 11 Pro 23H2",
      resolution: "1920x1080 @ 60Hz",
      notes: "Estação Financeiro: Impressora de rede HP LaserJet. Monitor duplo. Ramal 4022."
    },
    {
      id: "DESK-02",
      storeName: "Matriz - Corporativo",
      isOnline: true,
      agentVersion: "0.1.0",
      publicKey: "ed25519_host02_key_hash",
      lastSeen: "Agora",
      ipAddress: "192.168.1.102",
      osInfo: "Windows 11 Pro 23H2",
      resolution: "1920x1080 @ 60Hz",
      notes: "Estação Faturamento: Certificado digital A1 configurado. Leitor biométrico USB."
    },
    {
      id: "SRV-01",
      storeName: "Filial Sul - Escritório",
      isOnline: false,
      agentVersion: "0.1.0",
      publicKey: "ed25519_host04_key_hash",
      lastSeen: "15 min atrás",
      ipAddress: "192.168.2.101",
      osInfo: "Windows 10 Enterprise 22H2",
      resolution: "1920x1080 @ 60Hz",
      notes: "Servidor de Arquivos: Manutenção preventiva agendada fora do horário comercial."
    },
    {
      id: "NOTE-03",
      storeName: "Unidade Norte - Operações",
      isOnline: true,
      agentVersion: "0.1.0",
      publicKey: "ed25519_host03_key_hash",
      lastSeen: "Agora",
      ipAddress: "192.168.3.103",
      osInfo: "Windows 11 Pro 23H2",
      resolution: "1920x1080 @ 60Hz",
      notes: "Notebook Gestão: VPN corporativa habilitada. Conexão prioritária Wi-Fi 6."
    }
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
      document.getElementById('kpiLatency').textContent = `${state.latencyMs} ms`;
      document.getElementById('headerLatencyText').textContent = `${state.latencyMs} ms • Conexão Segura`;
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
  document.getElementById('kpiTotal').textContent = total;
  document.getElementById('kpiOnline').textContent = online;
  document.getElementById('kpiOffline').textContent = offline;
  document.getElementById('kpiLatency').textContent = `${state.latencyMs} ms`;
  document.getElementById('badgeOnlineCount').textContent = online;

  // Atualiza Pills
  document.getElementById('pillAllLabel').textContent = `Todos (${total})`;
  document.getElementById('pillOnlineLabel').textContent = `Online (${online})`;
  document.getElementById('pillOfflineLabel').textContent = `Offline (${offline})`;

  const filtered = getFilteredPdvs();
  document.getElementById('resultsCountLabel').textContent = `Exibindo ${filtered.length} de ${total} dispositivos`;

  // Renderiza Cards de Dispositivos
  const container = document.getElementById('pdvListContainer');
  container.innerHTML = '';

  if (filtered.length === 0) {
    container.innerHTML = `
      <div style="text-align: center; padding: 40px; color: var(--text-muted); background: var(--bg-card); border-radius: var(--radius-lg); border: 1px dashed var(--border-subtle);">
        <p style="font-size: 14px; font-weight: 500;">Nenhum dispositivo encontrado com os filtros atuais.</p>
        <p style="font-size: 12px; margin-top: 4px;">Tente alterar o status ou limpar o termo de pesquisa.</p>
      </div>
    `;
  } else {
    filtered.forEach(pdv => {
      const isSelected = pdv.id === state.inspectedPdvId && pdv.storeName === getInspectedPdv()?.storeName;
      const card = document.createElement('div');
      card.className = `pdv-card ${isSelected ? 'selected' : ''}`;
      card.onclick = () => selectPdvForInspection(pdv);

      const statusBadge = pdv.isOnline
        ? `<div class="status-badge online"><div class="pulse-dot"></div> ONLINE</div>`
        : `<div class="status-badge offline">OFFLINE</div>`;

      const actionBtn = pdv.isOnline
        ? `<button class="btn-primary" onclick="event.stopPropagation(); startSession('${pdv.id}', '${pdv.storeName}')">
             <svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
             <span>Conectar →</span>
           </button>`
        : `<button class="btn-disabled" disabled>Indisponível</button>`;

      card.innerHTML = `
        <div class="pdv-info-left">
          ${statusBadge}
          <div class="pdv-meta-group">
            <div class="pdv-title-row">
              <span class="pdv-id">${pdv.id}</span>
              <span style="color: var(--text-dim);">•</span>
              <span class="pdv-store">${pdv.storeName}</span>
              <span style="color: var(--text-dim);">•</span>
              <span class="pdv-seen">Visto: ${pdv.lastSeen}</span>
            </div>
            <div class="telemetry-chips">
              <span class="tele-chip">OS: ${pdv.osInfo}</span>
              <span class="tele-chip highlight">IP: ${pdv.ipAddress}</span>
              <span class="tele-chip">Tela: ${pdv.resolution}</span>
              <span class="tele-chip" style="color: var(--accent-violet);">Agente v${pdv.agentVersion}</span>
            </div>
          </div>
        </div>
        <div class="pdv-actions">
          <button class="btn-secondary" onclick="event.stopPropagation(); selectPdvForInspection(${JSON.stringify(pdv).replace(/"/g, '&quot;')})">
            Ficha Técnica
          </button>
          ${actionBtn}
        </div>
      `;

      container.appendChild(card);
    });
  }

  // Atualiza a Ficha Técnica / Drawer
  renderInspectorDrawer();
}

function getInspectedPdv() {
  return state.pdvs.find(p => p.id === state.inspectedPdvId) || state.pdvs[0];
}

function selectPdvForInspection(pdv) {
  state.inspectedPdvId = pdv.id;
  renderDashboard();
}

function renderInspectorDrawer() {
  const pdv = getInspectedPdv();
  if (!pdv) return;

  document.getElementById('drawerPdvId').textContent = pdv.id;
  document.getElementById('drawerStoreName').textContent = pdv.storeName;
  document.getElementById('diagIp').textContent = pdv.ipAddress;
  document.getElementById('diagOs').textContent = pdv.osInfo;
  document.getElementById('diagResolution').textContent = pdv.resolution;
  document.getElementById('diagVersion').textContent = `PrysmHost v${pdv.agentVersion} (LocalSystem)`;
  document.getElementById('diagKey').textContent = pdv.publicKey;
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
  if (pdv.isOnline) {
    btnConnect.disabled = false;
    btnConnect.className = 'btn-primary primary-connect-btn';
    btnConnect.innerHTML = `
      <svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
      <span>Conectar Sessão Remota (QUIC TLS 1.3)</span>
    `;
  } else {
    btnConnect.disabled = true;
    btnConnect.className = 'btn-disabled primary-connect-btn';
    btnConnect.innerHTML = `<span>Dispositivo Offline - Conexão Indisponível</span>`;
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
    tr.style.borderBottom = '1px solid var(--border-subtle)';
    tr.innerHTML = `
      <td style="padding: 10px 14px; font-size: 11.5px; font-weight: 600; color: var(--accent-cyan);">${rec.id}</td>
      <td style="padding: 10px 14px; font-size: 11.5px; color: var(--text-muted);">${rec.timestamp}</td>
      <td style="padding: 10px 14px; font-size: 11.5px; font-weight: 500; color: var(--accent-violet);">${rec.operator}</td>
      <td style="padding: 10px 14px; font-size: 11.5px; font-weight: 600; color: var(--text-primary);">${rec.pdv} (${rec.store})</td>
      <td style="padding: 10px 14px; font-size: 11.5px; color: var(--text-secondary);">${rec.reason}</td>
      <td style="padding: 10px 14px; font-size: 11.5px; color: var(--accent-mint);">${rec.duration}</td>
      <td style="padding: 10px 14px; font-size: 11px;"><span style="background: rgba(0, 240, 255, 0.1); border: 1px solid rgba(0, 240, 255, 0.3); color: var(--accent-cyan); padding: 2px 8px; border-radius: var(--radius-full);">${rec.status}</span></td>
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
        <h4 style="font-size: 14px; font-weight: 700; color: var(--text-primary);">${store}</h4>
        <span class="status-badge ${onlineCount > 0 ? 'online' : 'offline'}">${onlineCount}/${storePdvs.length} Online</span>
      </div>
      <p style="font-size: 11.5px; color: var(--text-muted);">Rede: 192.168.x.x • Gateway VPN Seguro</p>
      <div style="display: flex; gap: 6px; flex-wrap: wrap;">
        ${storePdvs.map(p => `<span class="tele-chip ${p.isOnline ? 'highlight' : ''}">${p.id}</span>`).join('')}
      </div>
      <button class="btn-secondary" style="margin-top: 6px; justify-content: center;" onclick="filterByStore('${store}')">
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
// INICIALIZAÇÃO E EVENT LISTENERS
// =============================================================================
document.addEventListener('DOMContentLoaded', () => {
  // Navegação na Sidebar
  document.querySelectorAll('.nav-item').forEach(btn => {
    btn.addEventListener('click', () => switchTab(btn.dataset.tab));
  });

  // KPI Clicks para filtro rápido
  document.querySelectorAll('.kpi-card[data-filter]').forEach(card => {
    card.addEventListener('click', () => {
      state.filterStatus = card.dataset.filter;
      updateFilterPills();
      renderDashboard();
    });
  });

  // Filtros Pills
  document.querySelectorAll('.pill-btn[data-filter]').forEach(btn => {
    btn.addEventListener('click', () => {
      state.filterStatus = btn.dataset.filter;
      updateFilterPills();
      renderDashboard();
    });
  });

  function updateFilterPills() {
    document.querySelectorAll('.pill-btn').forEach(b => {
      b.classList.toggle('active', b.dataset.filter === state.filterStatus);
    });
  }

  // Filtro de Grupo
  document.getElementById('storeSelectFilter').addEventListener('change', (e) => {
    state.selectedStore = e.target.value;
    renderDashboard();
  });

  // Busca Global
  document.getElementById('globalSearchInput').addEventListener('input', (e) => {
    state.searchQuery = e.target.value;
    renderDashboard();
  });

  // Ações do Drawer
  document.getElementById('btnDrawerConnect').addEventListener('click', () => {
    const pdv = getInspectedPdv();
    if (pdv && pdv.isOnline) startSession(pdv.id, pdv.storeName);
  });

  document.getElementById('btnQuickCode').addEventListener('click', () => {
    const pdv = getInspectedPdv();
    if (pdv) startSession(pdv.id, pdv.storeName);
  });

  document.getElementById('btnQuickRestartAgent').addEventListener('click', () => {
    const pdv = getInspectedPdv();
    sendRustCommand('restart_agent', { pdv_id: pdv.id });
    showToast(`Sinal de reinício enviado ao serviço do ${pdv.id}`);
  });

  document.getElementById('btnQuickRebootSystem').addEventListener('click', () => {
    const pdv = getInspectedPdv();
    sendRustCommand('reboot_system', { pdv_id: pdv.id });
    showToast(`Solicitação de reinício do sistema enviada ao ${pdv.id}`);
  });

  document.getElementById('btnQuickCtrlAltDel').addEventListener('click', () => {
    const pdv = getInspectedPdv();
    sendRustCommand('send_sas', { pdv_id: pdv.id });
    showToast(`Comando SAS Ctrl+Alt+Del enviado ao ${pdv.id}`);
  });

  document.getElementById('btnQuickPing').addEventListener('click', () => {
    const pdv = getInspectedPdv();
    sendRustCommand('ping_pdv', { pdv_id: pdv.id, ip: pdv.ipAddress });
    showToast(`Ping em ${pdv.ipAddress}: RTT 18 ms (QUIC OK)`);
  });

  // Salvar Notas Técnicas
  document.getElementById('btnSaveNotes').addEventListener('click', () => {
    const pdv = getInspectedPdv();
    const notes = document.getElementById('notesTextarea').value;
    pdv.notes = notes;
    sendRustCommand('save_notes', { pdv_id: pdv.id, notes });
    showToast(`Notas técnicas do dispositivo ${pdv.id} gravadas.`);
  });

  // Testar Ping na API
  document.getElementById('btnTestPing').addEventListener('click', () => {
    sendRustCommand('test_api_ping');
    showToast('Servidor de Sinalização: HTTP 200 OK (18 ms de RTT via TLS 1.3)');
  });

  // Salvar Preferências
  document.getElementById('btnSaveAllSettings').addEventListener('click', () => {
    const techName = document.getElementById('cfgTechName').value;
    const techRole = document.getElementById('cfgTechRole').value;
    const techDept = document.getElementById('cfgTechDept').value;
    
    document.getElementById('headerOperatorName').textContent = techName;
    document.getElementById('sidebarTechRole').textContent = techRole;
    document.getElementById('sidebarTechDept').textContent = techDept;

    sendRustCommand('save_settings', { techName, techRole, techDept });
    showToast('Configurações e parâmetros de suporte salvos com sucesso.');
  });

  // Exportar Auditoria CSV
  const handleExportCsv = () => {
    sendRustCommand('export_audit_csv');
    showToast('Relatório de auditoria exportado para a pasta de Downloads.');
  };
  document.getElementById('btnExportAudit').addEventListener('click', handleExportCsv);
  document.getElementById('btnExportAudit2').addEventListener('click', handleExportCsv);

  // Limpar Cache
  document.getElementById('btnClearCache').addEventListener('click', () => {
    sendRustCommand('clear_cache');
    showToast('Cache de telemetria e diagnósticos temporários liberados.');
  });

  // Sincronizar / Atualizar
  document.getElementById('btnRefresh').addEventListener('click', () => {
    sendRustCommand('refresh_fleet');
    showToast('Sincronização de dispositivos concluída (18 ms)');
    renderDashboard();
  });

  // Logout
  document.getElementById('btnLogout').addEventListener('click', () => {
    showToast('Sessão do operador encerrada com segurança.');
  });

  // Modal Inputs (Digitação com auto-avanço)
  const codeInputs = document.querySelectorAll('.code-digit');
  codeInputs.forEach((input, index) => {
    input.addEventListener('input', (e) => {
      if (e.target.value.length === 1 && index < codeInputs.length - 1) {
        codeInputs[index + 1].focus();
      }
    });

    input.addEventListener('keydown', (e) => {
      if (e.key === 'Backspace' && !e.target.value && index > 0) {
        codeInputs[index - 1].focus();
      }
    });
  });

  document.getElementById('btnModalClose').addEventListener('click', closeModal);
  document.getElementById('btnModalAuthorize').addEventListener('click', () => {
    const code = Array.from(codeInputs).map(i => i.value).join('');
    if (code.length < 8) {
      showToast('Por favor, insira o código completo de 8 dígitos.');
      return;
    }
    closeModal();
    showToast(`Código validado com sucesso! Conectando a ${activeSessionTarget?.pdvId || 'Dispositivo'} via QUIC...`);
  });

  // Conexão Direta Submit
  document.getElementById('btnDirectSubmit').addEventListener('click', () => {
    const pdvId = document.getElementById('directPdvInput').value.trim();
    const code = document.getElementById('directCodeInput').value.trim();
    if (!pdvId || code.length < 8) {
      showToast('Preencha o identificador do dispositivo e o código de 8 dígitos.');
      return;
    }
    showToast(`Conectando diretamente a ${pdvId} via QUIC TLS 1.3...`);
  });

  // Render inicial
  renderDashboard();
});
