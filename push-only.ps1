# 已在网页手动创建 Beyondoooars/CodexTZBox 后，用此脚本推送。
# 用法：powershell -ExecutionPolicy Bypass -File .\push-only.ps1

$ErrorActionPreference = 'Stop'
$owner = 'Beyondoooars'
$repo = 'CodexTZBox'

Set-Location $PSScriptRoot

Write-Host '=== 远端配置 ===' -ForegroundColor Cyan
git remote -v

Write-Host "`n=== 推送到 $owner/$repo ===" -ForegroundColor Cyan
git push -u origin main

Write-Host "`n完成：https://github.com/$owner/$repo" -ForegroundColor Green
Write-Host '若提示凭据，请使用 GitHub 用户名 + Personal Access Token（不要用账号密码）。'
