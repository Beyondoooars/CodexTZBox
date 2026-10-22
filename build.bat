@echo off
chcp 65001 >nul
setlocal
title 构建 CodexTZBox

cd /d "%~dp0resource"

echo ==== 开始构建 CodexTZBox（依赖已有缓存，首次约 1~3 分钟）====
echo.

call npm run build

if errorlevel 1 (
  echo.
  echo [失败] 构建没有成功，请把上面的报错内容发给我。
) else (
  echo.
  echo [完成] 产物已生成：
  echo   1^) %~dp0CodexTZBox.exe
  echo   2^) 工具箱目录中的同名文件（默认 D:\Toolkits\CodexTZBox）
)

echo.
pause
endlocal
