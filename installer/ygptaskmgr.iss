[Setup]
AppName=ygptaskmgr
AppVersion=0.1.1
DefaultDirName={pf}\ygptaskmgr
DisableProgramGroupPage=yes
OutputBaseFilename=ygptaskmgr-setup
Compression=lzma
SolidCompression=yes
ChangesEnvironment=yes

[Tasks]
Name: removedata; Description: "Удалить данные программы (tasks.json)"; \
    Flags: unchecked

[Files]
Source: "C:\Users\ttata\rustesting\ygptaskmgr\target\release\ygptaskmgr.exe"; \
    DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Добавляем {app} в PATH при установке
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; \
    ValueType: expandsz; ValueName: "Path"; \
    ValueData: "{olddata};{app}"; \
    Check: NeedsAddPath

[Code]

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

  Result := Pos(Uppercase(ExpandConstant('{app}')), Uppercase(Path)) = 0;
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

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  DataDir: string;
begin
  if CurUninstallStep = usUninstall then
  begin
    { 1. Чистим PATH }
    RemovePathEntry();

    { 2. Опционально удаляем данные }
    if IsTaskSelected('removedata') then
    begin
      DataDir := ExpandConstant('{userappdata}\yagopop\ygptaskmgr');
      DelTree(DataDir, True, True, True);
    end;
  end;
end;

