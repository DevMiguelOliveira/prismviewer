; Script Inno Setup 6 para o PrysmView Host (Instalador Exclusivo do Caixa / PDV)
; Instala apenas o servico prysmview-svc e a interface de bandeja PrysmHost.exe.
; NAO contem o painel de administracao do tecnico.

#define MyAppName "PrysmView Host"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "PrysmView Team"
#define MyAppURL "https://github.com/DevMiguelOliveira/prismviewer"
#define MyAppExeName "PrysmHost.exe"
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

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"; Tasks: desktopicon
Name: "{group}\Desinstalar {#MyAppName}"; Filename: "{uninstallexe}"; IconFilename: "{app}\assets\icon.ico"
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"

[Run]
Filename: "{sys}\sc.exe"; Parameters: "create {#MySvcExeName} binPath= ""{app}\{#MySvcExeName}"" start= auto DisplayName= ""PrysmView Remote Support Host Service"""; Flags: runhidden; Description: "Registrando serviço Windows"
Filename: "{sys}\sc.exe"; Parameters: "description {#MySvcExeName} ""Serviço de acesso remoto seguro e assistência técnica para caixas de loja (PDVs)."""; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "failure {#MySvcExeName} reset= 86400 actions= restart/5000/restart/10000/restart/60000"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "start {#MySvcExeName}"; Flags: runhidden
Filename: "{app}\{#MyAppExeName}"; Description: "Iniciar {#MyAppName} na barra de tarefas"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#MyAppExeName}"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "stop {#MySvcExeName}"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "delete {#MySvcExeName}"; Flags: runhidden

[UninstallDelete]
Type: files; Name: "{commonappdata}\PrysmView\reconnect_state.bin"
Type: dirifempty; Name: "{commonappdata}\PrysmView"
