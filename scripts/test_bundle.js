const fs = require('fs');
const raw_html = fs.readFileSync('crates/viewer/ui/index.html', 'utf8');
const css_content = fs.readFileSync('crates/viewer/ui/style.css', 'utf8');
const gsap_content = fs.readFileSync('crates/viewer/ui/gsap.min.js', 'utf8');
const js_content = fs.readFileSync('crates/viewer/ui/app.js', 'utf8');

const full_js = '<script>\nwindow.__PRYSM_SERVER_VERSION__ = "0.1.1";\n' + js_content + '\n</script>';

const bundled_html = raw_html
  .replace('<link rel="stylesheet" href="style.css">', '<style>\n' + css_content + '\n</style>')
  .replace('<script src="gsap.min.js"></script>', '<script>\n' + gsap_content + '\n</script>')
  .replace('<script src="app.js"></script>', full_js);

console.log('bundled_html length:', bundled_html.length);
console.log('Has style.css replaced:', !bundled_html.includes('href="style.css"'));
console.log('Has gsap replaced:', !bundled_html.includes('src="gsap.min.js"'));
console.log('Has app.js replaced:', !bundled_html.includes('src="app.js"'));
console.log('Has window.__PRYSM_SERVER_VERSION__:', bundled_html.includes('window.__PRYSM_SERVER_VERSION__'));

fs.writeFileSync('target/test_bundled.html', bundled_html);
console.log('Written target/test_bundled.html successfully');

