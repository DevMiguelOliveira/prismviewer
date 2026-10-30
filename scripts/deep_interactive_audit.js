const fs = require('fs');

const html = fs.readFileSync('crates/viewer/ui/index.html', 'utf8');
const js = fs.readFileSync('crates/viewer/ui/app.js', 'utf8');

console.log('=== DEEP INTERACTIVE AUDIT ===');

// Check all elements with onclick or event listeners in app.js
// Find all getElementById in app.js
const lines = js.split('\n');
const handlers = [];

for (let i = 0; i < lines.length; i++) {
  const line = lines[i];
  // Match: const btnX = document.getElementById('X'); ... btnX.onclick = ... or .addEventListener('click', ...)
  // Or document.getElementById('X')?.onclick = ...
  // Or document.querySelectorAll('...').forEach( ... onclick = ...)
  
  const mDirect = line.match(/document\.getElementById\(['"]([^'"]+)['"]\)(?:\?\.|\.)(onclick|addEventListener)/);
  if (mDirect) {
    handlers.push({ id: mDirect[1], type: mDirect[2], line: i + 1 });
  }

  const mAssign = line.match(/(?:const|let|var)\s+([a-zA-Z0-9_]+)\s*=\s*document\.getElementById\(['"]([^'"]+)['"]\)/);
  if (mAssign) {
    const varName = mAssign[1];
    const id = mAssign[2];
    for (let j = i + 1; j < Math.min(i + 40, lines.length); j++) {
      const nextLine = lines[j];
      if (nextLine.includes(`${varName}.onclick =`) || nextLine.includes(`${varName}?.onclick =`) || nextLine.includes(`${varName}.addEventListener('click'`) || nextLine.includes(`${varName}.addEventListener("click"`)) {
        handlers.push({ id, varName, line: j + 1 });
        break;
      }
    }
  }
}

console.log(`Found ${handlers.length} click handlers bound by ID in app.js.`);

// Check which of these IDs do NOT exist in index.html
const missingFromHtml = handlers.filter(h => !html.includes(`id="${h.id}"`) && !html.includes(`id='${h.id}'`));
console.log('Handlers for IDs missing in HTML:', missingFromHtml);

// Now check all buttons in index.html and see if they have handlers
const allBtnMatches = html.match(/<button\b[^>]*>/g) || [];
console.log(`Total <button> elements in HTML: ${allBtnMatches.length}`);

const buttonAudit = [];
for (const btnTag of allBtnMatches) {
  const idM = btnTag.match(/id=['"]([^'"]+)['"]/);
  const classM = btnTag.match(/class=['"]([^'"]+)['"]/);
  const dataTabM = btnTag.match(/data-tab=['"]([^'"]+)['"]/);
  const dataFilterM = btnTag.match(/data-filter=['"]([^'"]+)['"]/);
  const id = idM ? idM[1] : null;
  const cls = classM ? classM[1] : '';
  
  let handled = false;
  let reason = '';

  if (id && handlers.some(h => h.id === id)) {
    handled = true;
    reason = `ID ${id} bound to click in app.js`;
  } else if (dataTabM && js.includes('[data-tab]')) {
    handled = true;
    reason = `data-tab="${dataTabM[1]}" handled by [data-tab] listener`;
  } else if (dataFilterM && (js.includes('[data-filter]') || js.includes('pill-btn'))) {
    handled = true;
    reason = `data-filter="${dataFilterM[1]}" handled by filter listener`;
  } else if (cls.includes('nav-item') && js.includes('.nav-item')) {
    handled = true;
    reason = 'nav-item listener';
  } else if (cls.includes('view-toggle-btn') && (id === 'btnViewCards' || id === 'btnViewTable')) {
    handled = true;
    reason = 'view-toggle-btn listener';
  } else if (cls.includes('page-btn') || cls.includes('page-num')) {
    handled = true;
    reason = 'pagination listener';
  } else if (id && js.includes(id)) {
    handled = true;
    reason = `ID ${id} referenced in app.js`;
  } else {
    // Check if class is matched
    const classList = cls.split(/\s+/);
    for (const c of classList) {
      if (c && js.includes(`.${c}`)) {
        handled = true;
        reason = `Class .${c} referenced in app.js`;
        break;
      }
    }
  }

  buttonAudit.push({ tag: btnTag, id, cls, handled, reason });
}

const unhandled = buttonAudit.filter(b => !b.handled);
console.log(`Unhandled buttons (${unhandled.length}):`, unhandled);


