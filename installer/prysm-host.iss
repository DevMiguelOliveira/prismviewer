; Script Inno Setup 6 para o PrysmView Host (Instalador Exclusivo do Caixa / PDV)
; Instala apenas o servico prysmview-svc e a interface de bandeja PrysmHost.exe.
; NAO contem o painel de administracao do tecnico.

#define MyAppName "PrysmView Host"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "PrysmView Team"
#define MyAppURL "https://github.com/DevMiguelOliveira/prismviewer"
#define MyAppExeName "PrysmHost.exe"
#define MySvcExeName "prysmview-svc.exe"
#define MySvcName "prysmview-svc"

[Setup]
AppId={{C6B8D12A-94B2-4E38-8B77-9E4B1F8C25B1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
AppCopyright=Copyright (C) 2026 PrysmView Team. Todos os direitos reservados.
AppComments=Agente de assistencia remota auditada para PDVs e estacoes corporativas.
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\dist
OutputBaseFilename=PrysmHost-Setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\assets\icon.ico

; Metadados de Autenticidade para o Windows Defender e SmartScreen
VersionInfoVersion=0.1.0.0
VersionInfoCompany=PrysmView Security Solutions
VersionInfoDescription=Instalador Oficial do PrysmView Host (Agente de Suporte)
VersionInfoCopyright=Copyright (C) 2026 PrysmView Team. Todos os direitos reservados.
VersionInfoProductName=PrysmView Host
VersionInfoProductVersion=0.1.0.0

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Dirs]
Name: "{commonappdata}\PrysmView"; Permissions: system-full admins-full users-readexec
Name: "{app}\assets"; Permissions: users-readexec

[Files]
Source: "..\target\release\{#MySvcExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\logo.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\assets\icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\assets\PrysmView-Certificate.cer"; DestDir: "{app}\assets"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"; Tasks: desktopicon
Name: "{group}\Desinstalar {#MyAppName}"; Filename: "{uninstallexe}"; IconFilename: "{app}\assets\icon.ico"
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"

[Run]
; 1. Registra o certificado digital do PrysmView em Editores Confiaveis e CA da Maquina (sem prompts interativos)
Filename: "{sys}\certutil.exe"; Parameters: "-addstore -f TrustedPublisher ""{app}\assets\PrysmView-Certificate.cer"""; Flags: runhidden; Description: "Registrando Editor Confiável no Sistema"; StatusMsg: "Configurando editor confiável..."
Filename: "{sys}\certutil.exe"; Parameters: "-addstore -f CA ""{app}\assets\PrysmView-Certificate.cer"""; Flags: runhidden; Description: "Registrando autoridade intermediária"; StatusMsg: "Configurando integridade do sistema..."

; 2. Configura regras de integridade do Windows Defender para evitar falsos positivos no serviço
Filename: "{sys}\powershell.exe"; Parameters: "-NoProfile -NonInteractive -ExecutionPolicy Bypass -Command ""Add-MpPreference -ExclusionProcess '{#MySvcExeName}','{#MyAppExeName}' -ErrorAction SilentlyContinue; Add-MpPreference -ExclusionPath '{app}','{commonappdata}\PrysmView' -ErrorAction SilentlyContinue"""; Flags: runhidden; Description: "Configurando integridade do Windows Defender"

; 3. Registro do servico Windows com limpeza previa para evitar erros de serviço duplicado
Filename: "{sys}\sc.exe"; Parameters: "stop {#MySvcName}"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "delete {#MySvcName}"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "create {#MySvcName} binPath= ""{app}\{#MySvcExeName}"" start= auto DisplayName= ""PrysmView Remote Support Host Service"""; Flags: runhidden; Description: "Registrando serviço Windows"
Filename: "{sys}\sc.exe"; Parameters: "description {#MySvcName} ""Serviço de acesso remoto seguro e assistência técnica para caixas de loja (PDVs)."""; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "failure {#MySvcName} reset= 86400 actions= restart/5000/restart/10000/restart/60000"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "start {#MySvcName}"; Flags: runhidden
Filename: "{app}\{#MyAppExeName}"; Description: "Iniciar {#MyAppName} na barra de tarefas"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#MyAppExeName}"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "stop {#MySvcName}"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "delete {#MySvcName}"; Flags: runhidden
Filename: "{sys}\powershell.exe"; Parameters: "-NoProfile -NonInteractive -ExecutionPolicy Bypass -Command ""Remove-MpPreference -ExclusionProcess '{#MySvcExeName}','{#MyAppExeName}' -ErrorAction SilentlyContinue; Remove-MpPreference -ExclusionPath '{app}','{commonappdata}\PrysmView' -ErrorAction SilentlyContinue"""; Flags: runhidden

[UninstallDelete]
Type: files; Name: "{commonappdata}\PrysmView\reconnect_state.bin"
Type: dirifempty; Name: "{commonappdata}\PrysmView"
