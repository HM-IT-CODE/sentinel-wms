# test_mcp_sse.ps1
# Script en ASCII plano para probar el transporte SSE de MCP

$ErrorActionPreference = "Stop"

$exePath = "target/debug/mcp-sql-sentinel.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "No se encontro el ejecutable en $exePath. Ejecuta primero test_mcp.ps1 para compilar."
    exit 1
}

Write-Host "[START] Iniciando el proceso MCP en modo SSE (puerto 3000)..." -ForegroundColor Cyan
$proc = Start-Process -FilePath $exePath -ArgumentList "--transport", "sse" -NoNewWindow -PassThru -RedirectStandardError "stderr_sse.log"

# Esperar a que el servidor levante
Start-Sleep -Seconds 3

# Cargar assembly HTTP explícitamente para evitar fallos en PowerShell 5.1
Add-Type -AssemblyName System.Net.Http
$client = New-Object System.Net.Http.HttpClient
$client.Timeout = [System.TimeSpan]::FromSeconds(30)

try {
    Write-Host "[GET] Conectando a /sse..." -ForegroundColor Cyan
    $sseUri = [System.Uri]"http://localhost:3000/sse"
    $responseStreamTask = $client.GetStreamAsync($sseUri)
    Start-Sleep -Seconds 1

    if (-not $responseStreamTask.IsCompleted) {
        Write-Host "[OK] Conectado a SSE. Leyendo stream..." -ForegroundColor Green
    } else {
        Write-Error "La conexion SSE se cerro inmediatamente"
        exit 1
    }

    $stream = $responseStreamTask.Result
    $reader = New-Object System.IO.StreamReader($stream)

    # Leer evento 'endpoint'
    Write-Host "[READ] Esperando evento 'endpoint'..." -ForegroundColor Cyan
    $line1 = $reader.ReadLine() # event: endpoint
    $line2 = $reader.ReadLine() # data: /message?session_id=session-X
    $line3 = $reader.ReadLine() # linea vacia separadora

    Write-Host "Evento recibido:" -ForegroundColor Yellow
    Write-Host $line1 -ForegroundColor Gray
    Write-Host $line2 -ForegroundColor Gray

    # Parsear session_id
    $sessionId = ""
    if ($line2 -match "session_id=(session-\d+)") {
        $sessionId = $Matches[1]
        Write-Host "Session ID detectado: $sessionId" -ForegroundColor Green
    } else {
        Write-Error "No se pudo extraer session_id de: $line2"
        exit 1
    }

    # Enviar Initialize por POST a /message?session_id=XXX
    Write-Host "[POST] Enviando initialize a /message?session_id=$sessionId..." -ForegroundColor Cyan
    $postUri = "http://localhost:3000/message?session_id=$sessionId"
    $initializeJson = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
    $content = New-Object System.Net.Http.StringContent($initializeJson, [System.Text.Encoding]::UTF8, "application/json")

    $postTask = $client.PostAsync($postUri, $content)
    $postResult = $postTask.Result
    Write-Host "HTTP Status Post: $($postResult.StatusCode)" -ForegroundColor Green

    # Leer respuesta SSE ('message')
    Write-Host "[READ] Esperando respuesta en stream SSE..." -ForegroundColor Cyan
    $resLine1 = $reader.ReadLine() # event: message
    $resLine2 = $reader.ReadLine() # data: {jsonrpc...}
    $resLine3 = $reader.ReadLine() # linea vacia

    Write-Host "Evento 'message' recibido desde SSE:" -ForegroundColor Yellow
    Write-Host $resLine1 -ForegroundColor Gray
    Write-Host $resLine2 -ForegroundColor Gray

} finally {
    Write-Host "[STOP] Deteniendo el servidor..." -ForegroundColor Cyan
    $proc.Kill()
    Write-Host "[EXIT] Prueba completada con exito." -ForegroundColor Green
}
