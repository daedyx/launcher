# Fotografiert das Hauptfenster eines Prozesses (auch wenn es verdeckt ist, über PrintWindow).
param([string]$Process = "daedyx-launcher", [string]$Out = "$env:TEMP\daedyx-shot.png")
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public class W {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint flags);
  public struct RECT { public int L,T,R,B; }
}
"@
[W]::SetProcessDPIAware() | Out-Null
$p = Get-Process $Process | Select-Object -First 1
$r = New-Object W+RECT; [W]::GetWindowRect($p.MainWindowHandle, [ref]$r) | Out-Null
$bmp = New-Object System.Drawing.Bitmap ($r.R-$r.L), ($r.B-$r.T)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $g.GetHdc()
[W]::PrintWindow($p.MainWindowHandle, $dc, 2) | Out-Null   # 2 = PW_RENDERFULLCONTENT
$g.ReleaseHdc($dc)
$bmp.Save($Out); Write-Output $Out
