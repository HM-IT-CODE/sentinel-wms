# test_mcp.ps1
# Script en ASCII plano optimizado para evitar interbloqueos de buffers en Windows

$ErrorActionPreference = "Stop"

Write-Host "[BUILD] Compilando el proyecto..." -ForegroundColor Cyan
cargo build

$exePath = "target/debug/mcp-sql-sentinel.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "No se encontro el ejecutable en $exePath"
    exit 1
}

Write-Host "[START] Iniciando el proceso MCP en modo Stdio..." -ForegroundColor Cyan

# Configurar informacion de inicio del proceso
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $exePath
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
# Dejar que stderr fluya directo a la consola del host para evitar deadlocks de buffer
$psi.RedirectStandardError = $false
$psi.CreateNoWindow = $true

# Iniciar proceso
$proc = New-Object System.Diagnostics.Process
$proc.StartInfo = $psi
$proc.Start() | Out-Null

$stdin = $proc.StandardInput
$stdout = $proc.StandardOutput

# Funcion auxiliar para enviar datos a stdin y leer de stdout
function Send-McpRequest($jsonRequest) {
    Write-Host ""
    Write-Host "[ENVIANDO AL SERVIDOR]:" -ForegroundColor Yellow
    Write-Host $jsonRequest -ForegroundColor Gray
    
    # Escribir la peticion
    $stdin.WriteLine($jsonRequest)
    
    # Leer la linea de respuesta
    $response = $stdout.ReadLine()
    Write-Host "[RESPUESTA RECIBIDA (stdout)]:" -ForegroundColor Green
    Write-Host $response -ForegroundColor Gray
}

# 1. Enviar solicitud de inicializacion (initialize)
$initializeReq = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
Send-McpRequest $initializeReq

# 2. Enviar notificacion de inicializacion completada (initialized)
Write-Host ""
Write-Host "[NOTIFICACION] Enviando inicializado..." -ForegroundColor Yellow
$initializedNotif = '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
$stdin.WriteLine($initializedNotif)
Start-Sleep -Milliseconds 200

# 3. Enviar consulta de listado de herramientas (tools/list)
$toolsListReq = '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
Send-McpRequest $toolsListReq

# 4. Enviar llamada para ejecutar la herramienta ObtenerEsquemaBaseDatos (tools/call)
$getSchemaReq = '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ObtenerEsquemaBaseDatos","arguments":{}}}'
Send-McpRequest $getSchemaReq

Write-Host ""
Write-Host "[STOP] Cerrando la entrada estandar (stdin)..." -ForegroundColor Cyan
$stdin.Close()

# Esperar a que salga
$proc.WaitForExit() | Out-Null
Write-Host "[EXIT] Servidor MCP finalizado." -ForegroundColor Cyan
