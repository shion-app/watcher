$OutputEncoding = [console]::InputEncoding = [console]::OutputEncoding = New-Object System.Text.UTF8Encoding

Add-Type -AssemblyName System.Drawing

# 创建空数组，用于存储数据
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

# 获取前台运行的应用列表
$foregroundProcesses = Get-Process | Where-Object { $_.MainWindowHandle -ne 0 -and $_.MainWindowTitle -ne "" }

# 输出应用列表信息
foreach ($process in $foregroundProcesses) {
    # $processId = $process.Id
    $processPath = $process.Path

    # 检查是否为空值
    if (-not $processPath) {
        # 如果 $processPath 为空值，跳过当前循环
        Continue
    }

    # 使用 FileVersionInfo 获取更全面的信息
    $fileVersionInfo = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($processPath)
    $description = $fileVersionInfo.FileDescription

    # 通过条件运算符创建 programName
    $name = if ($description) { $description } else { $process.ProcessName }

    # 获取图标的缓冲区数据
    $iconBuffer = GetIconBytesByPath $processPath

    $program = @{
        path = $processPath
        name = $name
        icon = $iconBuffer
    }

    # 将数据添加到数组
    $programs += $program

    # Write-Host "可执行文件路径: $processPath, name: $name, iconBuffer: $iconBuffer"
}

# 将数组转换为 JSON 格式并输出
$programs | ConvertTo-Json