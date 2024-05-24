$OutputEncoding = [console]::InputEncoding = [console]::OutputEncoding = New-Object System.Text.UTF8Encoding

Add-Type -AssemblyName System.Drawing

$programs = @()

function GetIconBytesByPath($path) {
    $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($path)
    $bitmap = $icon.ToBitmap()

    $stream = New-Object System.IO.MemoryStream
    $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    $bytes = $stream.ToArray()
    $stream.Close()

    return $bytes
}

$foregroundProcesses = Get-Process | Where-Object { $_.MainWindowHandle -ne 0 -and $_.MainWindowTitle -ne "" }

foreach ($process in $foregroundProcesses) {
    $processPath = $process.Path

    if (-not $processPath) {
        Continue
    }

    $fileVersionInfo = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($processPath)
    $description = $fileVersionInfo.FileDescription

    $name = if ($description) { $description } else { $process.ProcessName }

    $iconBuffer = GetIconBytesByPath $processPath

    $program = @{
        path = $processPath
        name = $name
        icon = $iconBuffer
    }

    $programs += $program

}

$programs | ConvertTo-Json