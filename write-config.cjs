const fs = require('fs');
const path = require('path');

const config = {
  "$schema": "../node_modules/@tauri-apps/cli/schema.json",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://127.0.0.1:3000",
    "distDir": "../dist"
  },
  "package": {
    "productName": "PrysmView",
    "version": "2.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "dialog": {
        "all": true
      },
      "notification": {
        "all": true
      },
      "window": {
        "all": true
      }
    },
    "bundle": {
      "active": true,
      "category": "DeveloperTool",
      "copyright": "Copyright 2026 PrysmView Team",
      "icon": [
        "icons/icon.ico",
        "icons/icon.png"
      ],
      "identifier": "com.prysmview.app"
    },
    "security": {
      "csp": null
    },
    "updater": {
      "active": false
    },
    "windows": [
      {
        "fullscreen": false,
        "height": 860,
        "resizable": true,
        "title": "PrysmView - Acesso Remoto & Gestão de PDVs",
        "width": 1360,
        "minWidth": 1024,
        "minHeight": 700,
        "center": true,
        "decorations": true
      }
    ]
  }
};

const dir = path.join(__dirname, 'src-tauri');
if (!fs.existsSync(dir)) {
  fs.mkdirSync(dir, { recursive: true });
}

const target = path.join(dir, 'tauri.conf.json');
fs.writeFileSync(target, JSON.stringify(config, null, 2), { encoding: 'utf8' });
console.log('✅ tauri.conf.json gravado com sucesso em UTF-8 puro (sem BOM)!');
