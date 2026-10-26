; Script Inno Setup 6 para o PrysmView Admin (Instalador Exclusivo do Tecnico / Suporte)
; Instala apenas o console administrativo PrysmAdmin.exe com Dashboard, Sidebar, Monitoramento e Conexao Rapida.
; NAO instala servico Windows e NAO instala o Host do PDV.

#define MyAppName "PrysmView Admin"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "PrysmView Team"
#define MyAppURL "https://github.com/DevMiguelOliveira/prismviewer"
#define MyAppExeName "PrysmAdmin.exe"

[Setup]
AppId={{E8F5A7B3-85D1-4A29-8732-5B4C2D1F9E3A}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
AppCopyright=Copyright (C) 2026 PrysmView Team. Todos os direitos reservados.
AppComments=Console administrativo de assistencia e suporte remoto seguro.
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\dist
OutputBaseFilename=PrysmAdmin-Setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\assets\icon.ico

; Metadados de Autenticidade para o Windows Defender e SmartScreen
VersionInfoVersion=0.1.0.0
VersionInfoCompany=PrysmView Security Solutions
VersionInfoDescription=Instalador Oficial do PrysmView Admin (Console de Suporte)
VersionInfoCopyright=Copyright (C) 2026 PrysmView Team. Todos os direitos reservados.
VersionInfoProductName=PrysmView Admin
VersionInfoProductVersion=0.1.0.0

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Dirs]
Name: "{app}\assets"; Permissions: users-readexec

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\logo.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\assets\icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\assets\PrysmView-Certificate.cer"; DestDir: "{app}\assets"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"; Tasks: desktopicon
Name: "{group}\Desinstalar {#MyAppName}"; Filename: "{uninstallexe}"; IconFilename: "{app}\assets\icon.ico"

[Run]
; Registra o certificado do desenvolvedor no armazenamento do usuario para eliminar alertas do SmartScreen (sem prompts bloqueantes)
Filename: "{sys}\certutil.exe"; Parameters: "-addstore -user -f CA ""{app}\assets\PrysmView-Certificate.cer"""; Flags: runhidden; Description: "Registrando autoridade intermediária"; StatusMsg: "Configurando integridade do sistema..."
Filename: "{sys}\certutil.exe"; Parameters: "-addstore -user -f TrustedPublisher ""{app}\assets\PrysmView-Certificate.cer"""; Flags: runhidden; Description: "Registrando como Editor Confiável"; StatusMsg: "Configurando editor confiável..."
Filename: "{app}\{#MyAppExeName}"; Description: "Iniciar {#MyAppName}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#MyAppExeName}"; Flags: runhidden
