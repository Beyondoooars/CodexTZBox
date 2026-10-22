# 在 GitHub 上创建 CodexTZBox 仓库并推送本地代码。
# 用法：在项目根目录右键“使用 PowerShell 运行”，或执行：
#   powershell -ExecutionPolicy Bypass -File .\publish.ps1

$ErrorActionPreference = 'Stop'
$owner = 'Beyondoooars'
$repo = 'CodexTZBox'

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Host '未检测到 GitHub CLI (gh)。' -ForegroundColor Yellow
    Write-Host '请任选一种方式：'
    Write-Host '  1) 安装 gh：winget install --id GitHub.cli -e'
    Write-Host '  2) 或在网页 https://github.com/new 创建仓库后，运行本目录下的 push-only.ps1'
    exit 1
}

$authed = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host 'gh 尚未登录，正在启动登录流程…' -ForegroundColor Yellow
    gh auth login
}

Write-Host "`n=== 创建仓库 $owner/$repo ===" -ForegroundColor Cyan
gh repo view "$owner/$repo" 2>$null
if ($LASTEXITCODE -ne 0) {
    gh repo create "$repo" `
        --public `
        --description 'Windows 桌面工具：让 OpenAI Codex 客户端以进程级独立时区运行，不修改系统时区' `
        --source . `
        --remote origin `
        --push
} else {
    Write-Host '仓库已存在，直接推送…' -ForegroundColor Yellow
    git push -u origin main
}

Write-Host "`n完成：https://github.com/$owner/$repo" -ForegroundColor Green
