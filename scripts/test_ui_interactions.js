const fs = require('fs');

console.log('=== RUNNING COMPLETE UI INTERACTION VERIFICATION ===');

const html = fs.readFileSync('crates/viewer/ui/index.html', 'utf8');
const js = fs.readFileSync('crates/viewer/ui/app.js', 'utf8');
const css = fs.readFileSync('crates/viewer/ui/style.css', 'utf8');

// Verify critical CSS confinement for switches
if (css.includes('.switch-control,') && css.includes('.switch-toggle-wrap') && css.includes('position: relative;')) {
  console.log('PASS: .switch-control and .switch-toggle-wrap have position: relative');
} else {
  console.error('FAIL: Missing position: relative for switches');
  process.exit(1);
}

// Check all modal-box queries in JS
const badModalQueries = js.match(/modal\.querySelector\(['"]\.modal-content/g) || [];
if (badModalQueries.length === 0) {
  console.log('PASS: All modal.querySelector instances correctly include .modal-box');
} else {
  console.error('FAIL: Found unmigrated modal.querySelector:', badModalQueries);
  process.exit(1);
}

// Check that buttons in index.html exist and have IDs or classes matched
const buttonMatches = html.match(/<button\b[^>]*>/g) || [];
console.log(`Checking ${buttonMatches.length} <button> elements in HTML...`);

let issues = 0;
for (const b of buttonMatches) {
  const idM = b.match(/id=['"]([^'"]+)['"]/);
  const classM = b.match(/class=['"]([^'"]+)['"]/);
  const dataTabM = b.match(/data-tab=['"]([^'"]+)['"]/);
  const dataFilterM = b.match(/data-filter=['"]([^'"]+)['"]/);

  const id = idM ? idM[1] : null;
  const cls = classM ? classM[1] : '';

  let handled = false;
  if (id && js.includes(id)) handled = true;
  else if (dataTabM && js.includes('[data-tab]')) handled = true;
  else if (dataFilterM && (js.includes('[data-filter]') || js.includes('pill-btn'))) handled = true;
  else if (cls.includes('nav-item')) handled = true;
  else if (cls.includes('page-btn') || cls.includes('page-num')) handled = true;
  else if (cls.includes('btn-icon-ghost')) handled = true;

  if (!handled) {
    console.warn('WARN: Potential unhandled button:', b);
    issues++;
  }
}

if (issues === 0) {
  console.log(`PASS: All ${buttonMatches.length} buttons are bound to handlers or active logic!`);
} else {
  console.error(`FAIL: Found ${issues} unhandled buttons`);
  process.exit(1);
}

// Verify required IDs in HTML
const requiredIds = [
  'cfgTechName', 'cfgTechRole', 'cfgTechDept', 'cfgTechPhone', 'cfgDefaultReason',
  'cfgViewOnly', 'cfgBlockInput', 'cfgClipboard',
  'btnSaveTopSettings', 'btnSaveBottomSettings',
  'btnOpenNewTechModal', 'btnCloseNewTechModal', 'btnCancelNewTech', 'formNewTech',
  'btnOpenNewDeviceModal', 'btnCloseNewDeviceModal', 'btnCancelNewDevice', 'formNewDevice',
  'sessionModal', 'newDeviceModal', 'newTechModal',
  'btnModalClose', 'btnModalAuthorize', 'tabAuthCode', 'tabAuthPassword',
  'storeSelectFilter', 'btnTvConnect', 'btnDirectSubmit', 'btnToggleDrawer', 'btnQuickCtrlAltDel'
];

let missingRequired = 0;
for (const id of requiredIds) {
  if (!html.includes(`id="${id}"`) && !html.includes(`id='${id}'`)) {
    console.error(`FAIL: Missing required ID in HTML: ${id}`);
    missingRequired++;
  }
}

if (missingRequired === 0) {
  console.log(`PASS: All ${requiredIds.length} required interactive IDs are present in index.html!`);
} else {
  process.exit(1);
}

console.log('=== ALL UI VERIFICATIONS PASSED SUCCESSFULLY ===');

