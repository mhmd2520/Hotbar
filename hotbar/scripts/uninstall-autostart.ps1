#Requires -RunAsAdministrator
# Removes the logon task created by install-autostart.ps1 and stops the copy it started.
$task = Get-ScheduledTask -TaskName 'Hotbar' -ErrorAction SilentlyContinue
if (-not $task) {
    Write-Host 'No Hotbar autostart task found; nothing to remove.'
    return
}
Stop-ScheduledTask -TaskName 'Hotbar' -ErrorAction SilentlyContinue
Unregister-ScheduledTask -TaskName 'Hotbar' -Confirm:$false -ErrorAction Stop
Write-Host 'Hotbar autostart removed. Hotbar is no longer running; start it again from the Start menu if you want it.'
