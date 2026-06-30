@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul
cd /d "%~dp0"

title FilmRust Studio - PS 安装向导

:: ===== 检查必要文件 =====
if not exist "%~dp0胶片调色.jsx" call :missing "胶片调色.jsx"
if not exist "%~dp0filmrust.exe" call :missing "filmrust.exe"

:: ===== 主逻辑 =====
call :run_main "%~1"
exit /b %ERRORLEVEL%

:missing
echo.
echo [错误] 找不到文件: %~1
echo 请确保本 bat 和 胶片调色.jsx、filmrust.exe 在同一文件夹
pause
exit /b 1

:run_main
call :show_banner

:: 命令行传入了路径且有效 → 直接安装
if not "%~1"=="" if exist "%~1\Photoshop.exe" (
    set "PS_DIR=%~1"
    set "PS_VER=%~nx1"
    goto :do_install
)

call :search_ps
if !PS_FOUND!==1 call :confirm_ps
if !PS_FOUND!==1 goto :do_install

call :ask_path

:do_install
call :install_files
exit /b 0

:: ===== 显示横幅 =====
:show_banner
echo.
echo ================================================
echo    FilmRust Studio v5.9
echo    Photoshop 一键安装向导
echo    星TAP 软件 2026
echo ================================================
echo.
exit /b 0

:: ===== 自动搜索 Photoshop =====
:search_ps
echo 正在搜索已安装的 Photoshop...
echo.

set PS_FOUND=0

for %%p in (
    "C:\Program Files\Adobe\Adobe Photoshop 2026"
    "C:\Program Files\Adobe\Adobe Photoshop 2025"
    "C:\Program Files\Adobe\Adobe Photoshop 2024"
    "C:\Program Files\Adobe\Adobe Photoshop 2023"
    "C:\Program Files\Adobe\Adobe Photoshop 2022"
    "C:\Program Files\Adobe\Adobe Photoshop 2021"
    "C:\Program Files\Adobe\Adobe Photoshop 2020"
    "C:\Program Files\Adobe\Adobe Photoshop CC"
) do (
    if !PS_FOUND!==0 if exist "%%~p\Photoshop.exe" (
        set "PS_DIR=%%~p"
        set "PS_VER=%%~nxp"
        set PS_FOUND=1
        echo   找到: %%~p
    )
)

if !PS_FOUND!==0 call :not_found
exit /b 0

:not_found
echo   未在常见位置找到 Photoshop，将手动输入路径。
exit /b 0

:: ===== 确认检测到的 PS =====
:confirm_ps
echo.
echo 即将安装到上述位置。
echo   Y = 确认安装     N = 手动输入路径
echo.
set /p "CONFIRM=请选择 (Y/N): "
if /i "!CONFIRM!"=="N" set PS_FOUND=0
exit /b 0

:: ===== 手动输入 PS 路径 =====
:ask_path
call :show_ask_prompt

set "PS_DIR="
set /p "PS_DIR=路径: "

if "!PS_DIR!"=="" call :cancelled && exit /b 1

set "PS_DIR=!PS_DIR:"=!"

if not exist "!PS_DIR!\Photoshop.exe" call :invalid_path "!PS_DIR!"
if not exist "!PS_DIR!\Photoshop.exe" goto :ask_path

set "PS_VER=手动选择"
exit /b 0

:show_ask_prompt
echo.
echo --------------------------------------------------
echo   请输入 Photoshop 安装路径
echo   例如: C:\Program Files\Adobe\Adobe Photoshop 2026
echo   小技巧: 在资源管理器地址栏复制路径，右键粘贴到此处
echo --------------------------------------------------
echo.
exit /b 0

:invalid_path
echo.
echo [错误] 找不到 Photoshop.exe
echo   路径: %~1
echo 请检查路径是否正确，重新输入。
exit /b 0

:cancelled
echo 未输入路径，已取消安装。
pause
exit /b 1

:: ===== 复制文件到 PS 脚本目录 =====
:install_files
call :show_installing
call :do_copy
if !COPY_OK!==0 exit /b 1
call :show_done
exit /b 0

:show_installing
echo.
echo ================================================
echo   Photoshop 位置: !PS_DIR!
echo ================================================
echo.
exit /b 0

:do_copy
set "SCRIPT_DIR=!PS_DIR!\Presets\Scripts"

if not exist "!SCRIPT_DIR!" (
    echo 创建脚本目录...
    mkdir "!SCRIPT_DIR!" 2>nul
)

if not exist "!SCRIPT_DIR!" call :no_permission && exit /b 1

echo 正在复制文件...

copy /Y "%~dp0胶片调色.jsx" "!SCRIPT_DIR!\胶片调色.jsx" >nul
if errorlevel 1 call :no_permission && exit /b 1

copy /Y "%~dp0filmrust.exe" "!SCRIPT_DIR!\filmrust.exe" >nul

set COPY_OK=1
exit /b 0

:no_permission
echo [错误] 无法写入脚本目录，权限不足。
echo 请右键以管理员身份运行本脚本。
pause
set COPY_OK=0
exit /b 0

:show_done
echo.
echo ================================================
echo   安装完成！
echo ================================================
echo.
echo   已安装文件至:
echo     !SCRIPT_DIR!
echo.
echo   * 使用方法:
echo     Photoshop - 文件 - 脚本 - 胶片调色
echo.
echo   * 推荐设置快捷键 Ctrl+Shift+F:
echo     编辑 - 键盘快捷键 - 面板菜单
echo     - 找到 胶片调色 - 设置快捷键
echo.
echo   之后每张图按 Ctrl+Shift+F 一键出片！
echo.
echo ================================================
pause
exit /b 0
