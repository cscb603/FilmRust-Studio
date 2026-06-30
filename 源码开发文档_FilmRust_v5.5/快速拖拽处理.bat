@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul
cd /d "%~dp0"

echo ================================================
echo   FilmRust Studio - 快速拖拽批量处理
echo   把照片或文件夹拖到这个窗口上即可
echo ================================================
echo.

if not exist "filmrust.exe" (
    echo [错误] 找不到 filmrust.exe
    echo 请把本 bat 和 filmrust.exe 放在一起
    pause
    exit /b 1
)

set "INPUT=%~1"
if "%INPUT%"=="" (
    echo [用法] 直接把照片或文件夹拖到本 bat 上
    pause
    exit /b 1
)

if not exist "output" mkdir output

:: 判断输入是文件还是文件夹
if exist "%INPUT%\*" (
    set "SRC_DIR=%INPUT%"
) else (
    for %%f in ("%INPUT%") do set "SRC_DIR=%%~dpf"
)
echo 扫描目录: %SRC_DIR%
echo.

set COUNT=0
set FAIL=0

for %%f in ("%SRC_DIR%*.jpg" "%SRC_DIR%*.jpeg" "%SRC_DIR%*.png" "%SRC_DIR%*.JPG" "%SRC_DIR%*.JPEG" "%SRC_DIR%*.PNG" "%SRC_DIR%*.tif" "%SRC_DIR%*.tiff" "%SRC_DIR%*.TIF" "%SRC_DIR%*.TIFF") do (
    if exist "%%f" (
        set "OUT=output\%%~nf_film%%~xf"
        echo   [%%~nxf] 处理中...
        "%~dp0filmrust.exe" --input "%%f" --output "!OUT!" --auto >nul 2>&1
        if !errorlevel! equ 0 (
            set /a COUNT+=1
        ) else (
            set /a FAIL+=1
        )
    )
)

echo.
echo ================================================
echo   完成: 成功 %COUNT% 张
if !FAIL! gtr 0 echo   失败: %FAIL% 张
echo   输出目录: %CD%\output\
echo ================================================
pause
exit /b 0
