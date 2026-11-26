; Setup des Daedyx Launchers (NSIS, pro Benutzer, kein Administratorrecht).
; Bauen: scripts\build-setup.ps1 (setzt VERSION, BINDIR, OUTFILE).
;
; Schalter für das stille Selbst-Update (vom Launcher gestartet):
;   /S          still (NSIS)
;   /UPDATE     wartet, bis der laufende Launcher beendet ist, und startet ihn danach neu
;   /NOLAUNCH   nach /UPDATE nicht neu starten (Skripte, Tests)
Unicode true
RequestExecutionLevel user
SetCompressor /SOLID lzma

!ifndef VERSION
  !error "VERSION fehlt (makensis /DVERSION=0.1.0)"
!endif
!ifndef BINDIR
  !error "BINDIR fehlt"
!endif
!ifndef OUTFILE
  !define OUTFILE "Daedyx-Launcher_${VERSION}_x64-setup.exe"
!endif

; Installationsprobe (scripts\build-setup.ps1 -Probe, Review U3 F-U3-2): eigene Kennung, damit eine echte Installation
; „Daedyx Launcher“ nie berührt wird.
!ifdef PROBE
  !define APPNAME "Daedyx Launcher Probe"
  !define DIRKEY "Software\Daedyx\LauncherProbe"
  !define RUNVALUE "DaedyxLauncherProbe"
!else
  !define APPNAME "Daedyx Launcher"
  !define DIRKEY "Software\Daedyx\Launcher"
  !define RUNVALUE "DaedyxLauncher"
!endif
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
!define RUNKEY "Software\Microsoft\Windows\CurrentVersion\Run"

Name "${APPNAME}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\${APPNAME}"
; Bei einem Update derselbe Ordner wie bei der ersten Installation
InstallDirRegKey HKCU "${DIRKEY}" "InstallDir"
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${APPNAME}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "${APPNAME} Setup"

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"

!define MUI_ABORTWARNING
!define MUI_ICON "daedyx.ico"
!define MUI_UNICON "daedyx.ico"
!define MUI_FINISHPAGE_RUN "$INSTDIR\daedyx-launcher.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Daedyx Launcher starten"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "German"

Var IsUpdate
Var NoLaunch

Function .onInit
  ${GetParameters} $R0
  StrCpy $IsUpdate 0
  StrCpy $NoLaunch 0
  ClearErrors
  ${GetOptions} $R0 "/UPDATE" $R1
  ${IfNot} ${Errors}
    StrCpy $IsUpdate 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $R0 "/NOLAUNCH" $R1
  ${IfNot} ${Errors}
    StrCpy $NoLaunch 1
  ${EndIf}
FunctionEnd

; Der Launcher (Fenster oder Kommandozeile), der das Setup gestartet hat, beendet sich gleich danach. Solange seine
; Datei noch in Benutzung ist, lässt sie sich nicht löschen: bis zu 20 Sekunden auf beide Programme warten (U3: auch
; `daedyx.exe`, sonst scheitert ein Selbst-Update aus der Kommandozeile am Überschreiben).
Function WaitForLauncher
  StrCpy $0 0
  loop:
    ClearErrors
    Delete "$INSTDIR\daedyx-launcher.exe"
    Delete "$INSTDIR\daedyx.exe"
    ${IfNot} ${FileExists} "$INSTDIR\daedyx-launcher.exe"
    ${AndIfNot} ${FileExists} "$INSTDIR\daedyx.exe"
      Return
    ${EndIf}
    Sleep 500
    IntOp $0 $0 + 1
    ${If} $0 < 40
      Goto loop
    ${EndIf}
FunctionEnd

Section "Install"
  ${If} $IsUpdate == 1
    Call WaitForLauncher
  ${EndIf}
  SetOutPath "$INSTDIR"
  File "${BINDIR}\daedyx-launcher.exe"
  File "${BINDIR}\daedyx.exe"
  File "daedyx.ico"
  ; Seit 0.2.0 (U3) ohne Katalog-Werkzeug: das Programm einer älteren Fassung beim Update entfernen (F-U3-6).
  Delete "$INSTDIR\daedyx-catalog.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateShortcut "$SMPROGRAMS\${APPNAME}.lnk" "$INSTDIR\daedyx-launcher.exe" "" "$INSTDIR\daedyx.ico"

  WriteRegStr HKCU "${DIRKEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayName" "${APPNAME}"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTKEY}" "Publisher" "daedyx"
  WriteRegStr HKCU "${UNINSTKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayIcon" "$INSTDIR\daedyx.ico"
  WriteRegStr HKCU "${UNINSTKEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${UNINSTKEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINSTKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTKEY}" "NoRepair" 1

  ; Nach einem stillen Selbst-Update den Launcher wieder öffnen
  ${If} $IsUpdate == 1
  ${AndIf} $NoLaunch == 0
    Exec '"$INSTDIR\daedyx-launcher.exe"'
  ${EndIf}
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\daedyx-launcher.exe"
  Delete "$INSTDIR\daedyx.exe"
  Delete "$INSTDIR\daedyx-catalog.exe"
  Delete "$INSTDIR\daedyx.ico"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\${APPNAME}.lnk"
  DeleteRegValue HKCU "${RUNKEY}" "${RUNVALUE}"
  DeleteRegKey HKCU "${UNINSTKEY}"
  DeleteRegKey HKCU "${DIRKEY}"
  DeleteRegKey /ifempty HKCU "Software\Daedyx"
  ; Die mit dem Launcher installierten Apps und die Einstellungen bleiben bestehen.
SectionEnd
