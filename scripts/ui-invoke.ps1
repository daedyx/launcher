# Drückt einen Knopf des Fensters über Windows-UI-Automation (zugleich Beweis, dass die Oberfläche für Hilfsprogramme lesbar ist).
param([string]$Process = "daedyx-launcher", [string]$Name, [int]$Index = 0)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$p = Get-Process $Process | Select-Object -First 1
$root = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
$cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $Name)
$found = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
if ($found.Count -le $Index) { Write-Error "Element '$Name' nicht gefunden"; exit 1 }
$found[$Index].GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
