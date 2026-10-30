const fs = require('fs');

const html = fs.readFileSync('crates/viewer/ui/index.html', 'utf8');
const js = fs.readFileSync('crates/viewer/ui/app.js', 'utf8');

// 1. Extract all element IDs from index.html
const idRegex = /id=['"]([^'"]+)['"]/g;
const htmlIds = new Set();
let m;
while ((m = idRegex.exec(html)) !== null) {
  htmlIds.add(m[1]);
}

// 2. Extract all IDs referenced in app.js
const jsIdRegex = /getElementById\(['"]([^'"]+)['"]/g;
const jsIds = new Set();
while ((m = jsIdRegex.exec(js)) !== null) {
  jsIds.add(m[1]);
}

console.log('Total HTML IDs:', htmlIds.size);
console.log('Total JS getElementById IDs:', jsIds.size);

const missingInHtml = [...jsIds].filter(id => !htmlIds.has(id));
console.log('\n=== IDs in app.js that DO NOT EXIST in index.html ===');
console.log(missingInHtml);

// 3. Extract all buttons and interactive elements in index.html
const buttonRegex = /<button\b([^>]*)>/g;
const htmlButtons = [];
while ((m = buttonRegex.exec(html)) !== null) {
  const attrs = m[1];
  const idM = attrs.match(/id=['"]([^'"]+)['"]/);
  const classM = attrs.match(/class=['"]([^'"]+)['"]/);
  const titleM = attrs.match(/title=['"]([^'"]+)['"]/);
  htmlButtons.push({
    id: idM ? idM[1] : null,
    class: classM ? classM[1] : null,
    title: titleM ? titleM[1] : null,
    tag: m[0]
  });
}

console.log('\n=== Buttons in index.html ===');
console.log('Total buttons:', htmlButtons.length);

const unhandledButtons = [];
for (const b of htmlButtons) {
  let isHandled = false;
  if (b.id && (js.includes(`'${b.id}'`) || js.includes(`"${b.id}"`))) {
    isHandled = true;
  }
  if (!isHandled && b.class) {
    const classes = b.class.split(/\s+/);
    for (const c of classes) {
      if (js.includes(`.${c}`) || js.includes(`"${c}"`) || js.includes(`'${c}'`)) {
        isHandled = true;
        break;
      }
    }
  }
  if (!isHandled) {
    unhandledButtons.push(b);
  }
}
console.log('Unhandled buttons:', unhandledButtons);

// 4. Check all inputs, selects, textareas
const inputRegex = /<(input|select|textarea)\b([^>]*)>/g;
const formElements = [];
while ((m = inputRegex.exec(html)) !== null) {
  const tagType = m[1];
  const attrs = m[2];
  const idM = attrs.match(/id=['"]([^'"]+)['"]/);
  const nameM = attrs.match(/name=['"]([^'"]+)['"]/);
  const typeM = attrs.match(/type=['"]([^'"]+)['"]/);
  formElements.push({
    tag: tagType,
    id: idM ? idM[1] : null,
    name: nameM ? nameM[1] : null,
    type: typeM ? typeM[1] : null
  });
}

console.log('\nTotal Form Elements:', formElements.length);
const unhandledForm = [];
for (const f of formElements) {
  if (f.id && !js.includes(f.id)) {
    if (f.name && (js.includes(`name="${f.name}"`) || js.includes(`name='${f.name}'`))) {
      // handled by name
    } else {
      unhandledForm.push(f);
    }
  }
}
console.log('Form elements with ID not referenced in app.js:', unhandledForm);

// 5. Look for buttons or actions defined in app.js but having no trigger in HTML
console.log('\n=== Functions in app.js that might be uncalled or missing callers ===');
const funcRegex = /function\s+([a-zA-Z0-9_]+)\s*\(/g;
const funcs = [];
while ((m = funcRegex.exec(js)) !== null) {
  funcs.push(m[1]);
}
console.log('Total functions in app.js:', funcs.length);


