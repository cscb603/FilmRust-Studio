@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul
cd /d "%~dp0"

:: ================================================
::   FilmRust Studio - Photoshop 一键安装脚本
::   星TAP 软件 2026  csb603@qq.com
::   用法: 右键 → 以管理员身份运行
::         或: install_to_ps.bat "C:\Program Files\Adobe\Adobe Photoshop 2026"
:: ================================================

echo ================================================
echo   FilmRust Studio - Photoshop 一键安装
echo   星TAP 软件 2026  csb603@qq.com
echo ================================================
echo.

:: 如果命令行传入路径，直接使用
if not "%~1"=="" (
    if exist "%~1\Photoshop.exe" (
        set "PS_DIR=%~1"
        set "PS_VER=%~nx1"
        goto :install
    ) else (
        echo [失败] 指定路径未找到 Photoshop.exe: %~1
        echo.
    )
)

call :auto_detect
if errorlevel 1 (
    call :manual_input
    if errorlevel 1 (
        echo.
        echo 自动检测失败，请手动安装:
        echo   1. 把本目录下的 FilmRust_Studio.jsx、filmrust.exe、guitubiao.png
        echo   2. 复制到 Photoshop 安装目录的 Presets\Scripts\ 文件夹
        echo   3. 打开 Photoshop，菜单: 文件 ^> 脚本 ^> FilmRust_Studio
        echo.
        pause
        exit /b 1
    )
)

:install
echo [1/3] 检测到: !PS_VER!
echo         目录: !PS_DIR!
echo.

echo [2/3] 正在安装到 Photoshop 脚本菜单...
set "SCRIPT_DIR=!PS_DIR!\Presets\Scripts"
if not exist "!SCRIPT_DIR!" (
    mkdir "!SCRIPT_DIR!" 2>nul
)
if not exist "!SCRIPT_DIR!" (
    echo [失败] 无法创建脚本目录: !SCRIPT_DIR!
    echo 请右键 → 以管理员身份运行
    pause
    exit /b 1
)

copy /Y "%~dp0FilmRust_Studio.jsx" "!SCRIPT_DIR!\FilmRust_Studio.jsx" >nul 2>&1
if errorlevel 1 (
    echo [失败] 复制文件失败，权限不足
    echo 请右键 → 以管理员身份运行
    pause
    exit /b 1
)

copy /Y "%~dp0filmrust.exe" "!SCRIPT_DIR!\filmrust.exe" >nul 2>&1
copy /Y "%~dp0guitubiao.png" "!SCRIPT_DIR!\guitubiao.png" >nul 2>&1

echo   已安装到: !SCRIPT_DIR!
echo.

echo [3/3] 安装完成！
echo.
echo ================================================
echo   FilmRust Studio 已安装到 Photoshop！
echo ================================================
echo.
echo   ■ 使用方法:
echo.
echo     打开 Photoshop，点击:
echo       文件 → 脚本 → FilmRust_Studio
echo.
echo   ■ 推荐设置快捷键（一次设置，永久使用）:
echo.
echo     1. 菜单: 编辑 → 键盘快捷键 (Alt+Shift+Ctrl+K)
echo     2. 快捷键用于: 选择「面板菜单」
echo     3. 找到: 文件 → 脚本 → FilmRust_Studio
echo     4. 按 Ctrl+Shift+F → 接受 → 确定
echo.
echo   ■ 之后每张图按 Ctrl+Shift+F 一键出片！
echo.
echo ================================================
pause
exit /b 0

:: ========== 子程序：自动检测 PS 版本 ==========
:auto_detect
set PS_VER=
set PS_DIR=

echo [1/3] 正在检测 Photoshop 版本...

:: 方法A: 注册表查询（版本号 27.0=2026, 26.0=2025, ...）
for %%v in (27.0 26.0 25.0 24.0 23.0 22.0 21.0 20.0) do (
    if "!PS_VER!"=="" (
        for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\Adobe\Photoshop\%%v" /v ApplicationPath 2^>nul ^| findstr /i "REG_SZ"') do (
            if exist "%%b\Photoshop.exe" (
                set "PS_VER=%%v"
                set "PS_DIR=%%b"
            )
        )
    )
)

:: 方法B: 检查常见安装目录
if "!PS_VER!"=="" (
    for %%p in (
        "C:\Program Files\Adobe\Adobe Photoshop 2026"
        "C:\Program Files\Adobe\Adobe Photoshop 2025"
        "C:\Program Files\Adobe\Adobe Photoshop 2024"
        "C:\Program Files\Adobe\Adobe Photoshop 2023"
        "C:\Program Files\Adobe\Adobe Photoshop 2022"
        "C:\Program Files\Adobe\Adobe Photoshop 2021"
        "C:\Program Files\Adobe\Adobe Photoshop 2020"
        "C:\Program Files\Adobe\Adobe Photoshop CC 2019"
        "C:\Program Files\Adobe\Adobe Photoshop CC 2018"
    ) do (
        if exist "%%~p\Photoshop.exe" (
            set "PS_VER=%%~nxp"
            set "PS_DIR=%%~p"
        )
    )
)

if "!PS_VER!"=="" (
    echo [失败] 未检测到 Photoshop
    exit /b 1
)
exit /b 0

:: ========== 子程序：手动输入路径 ==========
:manual_input
echo.
echo 请手动输入 Photoshop 安装路径:
echo 例如: C:\Program Files\Adobe\Adobe Photoshop 2026
echo.
set /p "PS_DIR=路径: "
if "!PS_DIR!"=="" exit /b 1

:: 去掉可能输入的引号
set "PS_DIR=!PS_DIR:"=!"

if not exist "!PS_DIR!\Photoshop.exe" (
    echo [失败] 该目录下未找到 Photoshop.exe
    echo   路径: !PS_DIR!
    exit /b 1
)

set "PS_VER=手动输入"
exit /b 0
