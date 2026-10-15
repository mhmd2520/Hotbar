; Hotbar's autostart task (scripts\install-autostart.ps1) runs it elevated. This per-user installer
; runs unelevated, so it can't close that process ("Failed to kill Hotbar"), but it may end and
; start the user's own scheduled task. Stop it before files are replaced; start it again after.

Var HotbarStopped

!macro HOTBAR_STOP_TASK
  nsExec::Exec 'schtasks /End /TN Hotbar'
  Pop $0
  ${If} $0 = 0
    StrCpy $HotbarStopped 1
    Sleep 1000 ; let it exit before the "is Hotbar running?" check
  ${EndIf}
!macroend

!macro HOTBAR_START_TASK
  nsExec::Exec 'schtasks /Query /TN Hotbar'
  Pop $0
  ${If} $0 = 0
    nsExec::Exec 'schtasks /Run /TN Hotbar'
    Pop $0
    Sleep 1500 ; up before the Finish page's "Run Hotbar", which then just opens its menu
  ${EndIf}
!macroend

; As soon as the installer window opens: the upgrade page may run the previous version's
; uninstaller before any hook below, and that one has to find Hotbar closed too.
!ifndef MUI_CUSTOMFUNCTION_GUIINIT
  !define MUI_CUSTOMFUNCTION_GUIINIT HotbarGuiInit
  Function HotbarGuiInit
    !insertmacro HOTBAR_STOP_TASK
  FunctionEnd
!endif

; Cancelled: put Hotbar back the way it was.
!ifndef MUI_CUSTOMFUNCTION_ABORT
  !define MUI_CUSTOMFUNCTION_ABORT HotbarAbort
  Function HotbarAbort
    ${If} $HotbarStopped = 1
      !insertmacro HOTBAR_START_TASK
    ${EndIf}
  FunctionEnd
!endif

!macro NSIS_HOOK_PREINSTALL
  !insertmacro HOTBAR_STOP_TASK ; silent installs (/S) have no window, so no GuiInit
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro HOTBAR_START_TASK
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec 'schtasks /End /TN Hotbar'
  Pop $0
  Sleep 1000
!macroend
