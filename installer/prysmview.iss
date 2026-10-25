; Script do Instalador Inno Setup 6 para o PrysmView
; Conforme ADR 0002 e Fase 10 da especificação AGENTS.md

#define MyAppName "PrysmView"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "PrysmView Team"
#define MyAppURL "https://github.com/DevMiguelOliveira/prismviewer"
#define MyAppExeName "prysmview-tray.exe"
#define MyViewerExeName "prysmview-viewer.exe"
#define MySvcExeName "prysmview-svc.exe"

[Setup]
AppId={{C6B8D12A-94B2-4E38-8B77-9E4B1F8C25B1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\dist
OutputBaseFilename=PrysmView-Setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Dirs]
; Diretório de dados com permissões restritas (apenas SYSTEM e Administradores têm controle total)
Name: "{commonappdata}\PrysmView"; Permissions: system-full admins-full users-readexec
Name: "{app}\assets"; Permissions: users-readexec

[Files]
; Binários do PrysmView compilados em release
Source: "..\target\release\{#MySvcExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\{#MyViewerExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\logo.png"; DestDir: "{app}\assets"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName} Host (PDV)"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{#MyAppName} Viewer (Técnico)"; Filename: "{app}\{#MyViewerExeName}"
Name: "{autodesktop}\{#MyAppName} Viewer"; Filename: "{app}\{#MyViewerExeName}"; Tasks: desktopicon
Name: "{group}\Desinstalar {#MyAppName}"; Filename: "{uninstallexe}"
; Inicialização automática do aplicativo de bandeja no logon de qualquer usuário
Name: "{commonstartup}\{#MyAppName} Tray"; Filename: "{app}\{#MyAppExeName}"

[Run]
; 1. Registra o serviço Windows prysmview-svc em LocalSystem com início automático
Filename: "{sys}\sc.exe"; Parameters: "create {#MySvcExeName} binPath= ""{app}\{#MySvcExeName}"" start= auto DisplayName= ""PrysmView Remote Support Host Service"""; Flags: runhidden; Description: "Registrando serviço Windows"
; 2. Define a descrição do serviço no SCM
Filename: "{sys}\sc.exe"; Parameters: "description {#MySvcExeName} ""Serviço de acesso remoto seguro e assistência técnica para caixas de loja (PDVs)."""; Flags: runhidden
; 3. Configura política de recuperação automática em caso de falha (Restart=on-failure)
Filename: "{sys}\sc.exe"; Parameters: "failure {#MySvcExeName} reset= 86400 actions= restart/5000/restart/10000/restart/60000"; Flags: runhidden
; 4. Inicia o serviço Windows imediatamente após a instalação
Filename: "{sys}\sc.exe"; Parameters: "start {#MySvcExeName}"; Flags: runhidden
; 5. Inicia o aplicativo da bandeja na sessão do operador
Filename: "{app}\{#MyAppExeName}"; Description: "Iniciar {#MyAppName} na barra de tarefas"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; 1. Finaliza a bandeja se estiver rodando
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#MyAppExeName}"; Flags: runhidden
; 2. Para o serviço Windows prysmview-svc
Filename: "{sys}\sc.exe"; Parameters: "stop {#MySvcExeName}"; Flags: runhidden
; 3. Aguarda parada e remove o registro do serviço no SCM
Filename: "{sys}\sc.exe"; Parameters: "delete {#MySvcExeName}"; Flags: runhidden

[UninstallDelete]
; Limpeza de arquivos de estado temporários e logs na desinstalação
Type: files; Name: "{commonappdata}\PrysmView\reconnect_state.bin"
Type: dirifempty; Name: "{commonappdata}\PrysmView"
