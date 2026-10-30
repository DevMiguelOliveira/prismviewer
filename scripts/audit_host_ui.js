const fs = require('fs');
const html = fs.readFileSync('crates/tray/ui/index.html', 'utf8');
const js = fs.readFileSync('crates/tray/ui/host.js', 'utf8');

const idRegex = /id=['"]([^'"]+)['"]/g;
const htmlIds = new Set();
let m;
while ((m = idRegex.exec(html)) !== null) htmlIds.add(m[1]);

const jsIdRegex = /getElementById\(['"]([^'"]+)['"]/g;
const jsIds = new Set();
while ((m = jsIdRegex.exec(js)) !== null) jsIds.add(m[1]);

console.log('Host HTML IDs:', htmlIds.size);
console.log('Host JS getElementById IDs:', jsIds.size);

const missingInHtml = [...jsIds].filter(id => !htmlIds.has(id));
console.log('Missing in Host HTML:', missingInHtml);

