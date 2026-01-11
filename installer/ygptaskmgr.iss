#define MyVersion "0.2.1"
#define MyArch "x86_64"


[Setup]
AppName=ygptaskmgr
AppVersion={#MyVersion}
DefaultDirName={autopf}\ygptaskmgr
DisableProgramGroupPage=yes
OutputBaseFilename=ygptaskmgr-{#MyVersion}-setup-windows-{#MyArch}
Compression=lzma
SolidCompression=yes
ChangesEnvironment=yes
OutputDir=output

[Files]
Source: "..\target\release\ygptaskmgr.exe"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Добавляем {app} в PATH при установке
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; \
    ValueType: expandsz; ValueName: "Path"; \
    ValueData: "{olddata};{app}"; \
    Check: NeedsAddPath

[Code]

var
  RemoveUserData: Boolean;

function NeedsAddPath(): Boolean;
var
  Path: string;
begin
  if not RegQueryStringValue(
    HKLM,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path',
    Path
  ) then
  begin
    Result := True;
    exit;
  end;

  Result := Pos(
    Uppercase(ExpandConstant('{app}')),
    Uppercase(Path)
  ) = 0;
end;

procedure RemovePathEntry();
var
  Path: string;
  AppDir: string;
begin
  AppDir := ExpandConstant('{app}');

  if not RegQueryStringValue(
    HKLM,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path',
    Path
  ) then
    exit;

  StringChangeEx(Path, ';' + AppDir, '', True);
  StringChangeEx(Path, AppDir + ';', '', True);
  StringChangeEx(Path, AppDir, '', True);

  RegWriteExpandStringValue(
    HKLM,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path',
    Path
  );
end;

procedure InitializeUninstallProgressForm();
begin
  RemoveUserData :=
    MsgBox(
      'Удалить данные программы (tasks.json и настройки)?',
      mbConfirmation,
      MB_YESNO
    ) = IDYES;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  DataDir: string;
begin
  if CurUninstallStep = usUninstall then
  begin
    { 1. Чистим PATH }
    RemovePathEntry();

    { 2. Опционально удаляем данные пользователя }
    if RemoveUserData then
    begin
      DataDir := ExpandConstant('{userappdata}\yagopop\ygptaskmgr');
      DelTree(DataDir, True, True, True);
    end;
  end;
end;
