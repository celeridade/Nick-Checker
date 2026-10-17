#Requires -Version 5.1
# Avalia JS no Brave via CDP (perfil debug do usuario; nao toca login/cookies).
# Uso: powershell -File tools\cdp.ps1 -TargetId <id> -Expression 'document.title'
# Saida: JSON { ok, type, value } no stdout.
param(
    [Parameter(Mandatory = $true)][string]$TargetId,
    [Parameter(Mandatory = $true)][string]$Expression,
    [int]$TimeoutSec = 20,
    [switch]$Await
)
$ErrorActionPreference = 'Stop'
$wsUrl = "ws://127.0.0.1:9222/devtools/page/$TargetId"
$payload = @{ id = 1; method = 'Runtime.evaluate'; params = @{ expression = $Expression; returnByValue = $true; awaitPromise = [bool]$Await } } | ConvertTo-Json -Depth 6 -Compress
$ws = [System.Net.WebSockets.ClientWebSocket]::new()
$cts = [System.Threading.CancellationTokenSource]::new($TimeoutSec * 1000)
try {
    $ws.ConnectAsync([Uri]$wsUrl, $cts.Token).GetAwaiter().GetResult() | Out-Null
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($payload)
    $ws.SendAsync([System.ArraySegment[byte]]$bytes, [System.Net.WebSockets.WebSocketMessageType]::Text, $true, $cts.Token).GetAwaiter().GetResult() | Out-Null
    $buf = New-Object byte[] 1048576
    $sb = New-Object System.Text.StringBuilder
    do {
        $r = $ws.ReceiveAsync([System.ArraySegment[byte]]$buf, $cts.Token).GetAwaiter().GetResult()
        $sb.Append([System.Text.Encoding]::UTF8.GetString($buf, 0, $r.Count)) | Out-Null
    } while (-not $r.EndOfMessage)
    $resp = $sb.ToString() | ConvertFrom-Json
    $res = $resp.result.result
    if ($null -ne $res.subtype -and $res.subtype -eq 'error') {
        @{ ok = $false; error = $res.description } | ConvertTo-Json -Compress
    } else {
        @{ ok = $true; type = $res.type; value = $res.value } | ConvertTo-Json -Depth 8 -Compress
    }
} finally {
    $cts.Cancel()
    $ws.Dispose()
    $cts.Dispose()
}
