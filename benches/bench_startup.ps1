param (
    [int]$Iterations = 5,
    [string]$Config = "release",
    [string]$BinaryPath = ""
)

$ErrorActionPreference = "Stop"

$workspaceRoot = Split-Path -Parent $PSScriptRoot
Set-Location $workspaceRoot

if (-not $BinaryPath) {
    $BinaryPath = Join-Path $workspaceRoot "target\$Config\mdpad.exe"
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " MdPad Cold-Start Benchmark Runner" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Target Configuration: $Config"
Write-Host "Target Binary:        $BinaryPath"
Write-Host "Iteration Count:      $Iterations"
Write-Host ""

if (-not (Test-Path $BinaryPath)) {
    Write-Host "Binary not found. Compiling with 'cargo build --$Config'..." -ForegroundColor Yellow
    if ($Config -eq "release") {
        cargo build --release
    } else {
        cargo build
    }
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Cargo build failed with exit code $LASTEXITCODE."
        exit 1
    }
}

$latencies = [System.Collections.Generic.List[double]]::new()

for ($i = 1; $i -le $Iterations; $i++) {
    Write-Host "Run $i of $Iterations... " -NoNewline

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $BinaryPath
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $false

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $proc = [System.Diagnostics.Process]::Start($psi)

    if ($null -eq $proc) {
        Write-Error "Failed to start process: $BinaryPath"
        exit 1
    }

    try {
        # Wait until GUI process enters idle state or exits
        $exitedOrIdle = $false
        while (-not $exitedOrIdle -and $stopwatch.ElapsedMilliseconds -lt 5000) {
            $proc.Refresh()
            if ($proc.HasExited) {
                $exitedOrIdle = $true
                break
            }
            if ($proc.MainWindowHandle -ne [IntPtr]::Zero) {
                $exitedOrIdle = $true
                break
            }
            Start-Sleep -Milliseconds 5
        }
        $stopwatch.Stop()

        $elapsedMs = $stopwatch.Elapsed.TotalMilliseconds
        $latencies.Add($elapsedMs)
        Write-Host ("{0:F2} ms" -f $elapsedMs) -ForegroundColor Green
    }
    finally {
        if (-not $proc.HasExited) {
            $proc.Kill()
            [void]$proc.WaitForExit(1000)
        }
        $proc.Dispose()
    }

    Start-Sleep -Milliseconds 200
}

Write-Host ""
Write-Host "Benchmark Results:" -ForegroundColor Cyan
Write-Host "----------------------------------------"

$latenciesList = $latencies.ToArray()
[Array]::Sort($latenciesList)

$min = $latenciesList[0]
$max = $latenciesList[$latenciesList.Length - 1]
$avg = ($latenciesList | Measure-Object -Average).Average
$mid = [int]($latenciesList.Length / 2)
$median = if ($latenciesList.Length % 2 -eq 0) {
    ($latenciesList[$mid - 1] + $latenciesList[$mid]) / 2.0
} else {
    $latenciesList[$mid]
}

Write-Host ("  Min Latency:    {0:F2} ms" -f $min)
Write-Host ("  Max Latency:    {0:F2} ms" -f $max)
Write-Host ("  Avg Latency:    {0:F2} ms" -f $avg)
Write-Host ("  Median Latency: {0:F2} ms" -f $median)
Write-Host "----------------------------------------"
