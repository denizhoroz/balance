; Balance installer (Inno Setup 6). Build with installer\build.ps1, which
; passes /DAppVersion from app/Cargo.toml.
;
; Installed layout (db\ and .cache\ live next to the exe):
;   {app}\balance.exe
;   {app}\db\links.json      shipped; settings.toml / balance.xlsx made by the app
;   {app}\.cache\            made by the app

#define AppName "Balance"
#define AppExe "balance.exe"
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{19599519-E2ED-469F-927E-2C1A9FB48717}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=denizhoroz
; All users (Program Files) preselected; the dialog also offers "just me"
; (%LOCALAPPDATA%\Programs).
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
DefaultDirName={autopf}\{#AppName}
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\assets\icons\balance.ico
UninstallDisplayIcon={app}\{#AppExe}
OutputDir=..\dist
OutputBaseFilename=Balance-{#AppVersion}-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes

[Tasks]
Name: desktopicon; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Dirs]
; Program Files is read-only for normal users: in all-users mode every user
; may write the shared data and cache. Per-user installs are writable anyway.
Name: "{app}\db"; Permissions: users-modify; Flags: uninsneveruninstall; Check: IsAdminInstallMode
Name: "{app}\.cache"; Permissions: users-modify; Check: IsAdminInstallMode

[Files]
Source: "..\app\target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
; Overwritten on every install so upgrades bring new price links. Removed
; on uninstall only together with the rest of db\ (see [Code]).
Source: "links.json"; DestDir: "{app}\db"; Flags: ignoreversion uninsneveruninstall

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; The price cache is disposable.
Type: filesandordirs; Name: "{app}\.cache"

[Code]
// After uninstalling, ask whether to delete the user's data too. The
// default (Enter, or a silent uninstall) keeps it.
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if (CurUninstallStep = usPostUninstall) and DirExists(ExpandConstant('{app}\db')) then
  begin
    if SuppressibleMsgBox('Also delete your data (workbook, settings, price links)?' + #13#10#13#10 +
        ExpandConstant('{app}\db'), mbConfirmation, MB_YESNO or MB_DEFBUTTON2, IDNO) = IDYES then
    begin
      DelTree(ExpandConstant('{app}\db'), True, True, True);
      RemoveDir(ExpandConstant('{app}'));
    end;
  end;
end;
