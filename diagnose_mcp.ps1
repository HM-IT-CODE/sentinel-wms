# diagnose_mcp.ps1
# Redirigir entrada JSON-RPC directa al binario y guardar salidas en archivos planos

$inputJson = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ObtenerEsquemaBaseDatos","arguments":{}}}'

Write-Host "Ejecutando binario con entrada JSON-RPC..."
$inputJson | .\target\debug\mcp-sql-sentinel.exe --transport stdio > stdout.log 2> stderr.log
Write-Host "Ejecucion finalizada."
