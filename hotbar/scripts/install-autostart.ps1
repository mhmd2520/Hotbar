#Requires -RunAsAdministrator
<#
  Starts Hotbar at logon with highest privileges, so the Ctrl double-tap also works
  while an elevated window is focused. Claude sessions are still opened unelevated.
  Run once from an admin PowerShell:  .\install-autostart.ps1
#>
param([string]$Exe)

if (-not $Exe) {
    $candidates = @(
        "$env:LOCALAPPDATA\Hotbar\hotbar.exe",
        "$env:ProgramFiles\Hotbar\hotbar.exe",
        (Join-Path $PSScriptRoot '..\src-tauri\target\release\hotbar.exe')
    )
    $Exe = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $Exe -or -not (Test-Path $Exe)) { throw "hotbar.exe not found. Pass -Exe <path>." }
$Exe = (Resolve-Path $Exe).Path
$user = "$env:USERDOMAIN\$env:USERNAME"

# --background: start silently in the tray (a start from the Start menu opens the menu instead)
$action    = New-ScheduledTaskAction -Execute $Exe -Argument '--background' -WorkingDirectory (Split-Path $Exe)
$trigger   = New-ScheduledTaskTrigger -AtLogOn -User $user
$principal = New-ScheduledTaskPrincipal -UserId $user -LogonType Interactive -RunLevel Highest
$settings  = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
               -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew

Register-ScheduledTask -TaskName 'Hotbar' -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
Write-Host "Hotbar will start elevated at logon: $Exe"
if (Get-Process hotbar -ErrorAction SilentlyContinue) {
    Write-Host "Hotbar is already running (not elevated). Quit it from the tray first, then run:  Start-ScheduledTask -TaskName Hotbar"
} else {
    Write-Host "Start it now with:  Start-ScheduledTask -TaskName Hotbar"
}
