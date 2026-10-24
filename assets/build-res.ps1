# 重新编译 assets/app.res（图标 + 文件版本信息）。
#
# 什么时候需要跑:
#   - 改了 app.rc（比如版本号、文件描述）
#   - 换了图标，或重新跑过 make-icons.py
#
# 为什么把编译好的 .res 提交进仓库、而不在 build.rs 里现编译:
# 这样 `cargo build` 不需要任何资源编译器，克隆下来就能直接构建。
# 代价是版本号得手工同步，CI 会比对 exe 的版本资源与 Cargo.toml，忘记时会失败。

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$rcFile = 'assets\app.rc'
$outFile = 'assets\app.res'

# rc.exe 来自 Windows SDK，windres 来自 MSYS2 / MinGW，两者都能产出
# MSVC 链接器接受的 RES 格式。
$rc = Get-Command rc.exe -ErrorAction SilentlyContinue
$windres = Get-Command windres.exe -ErrorAction SilentlyContinue

if ($rc) {
    Write-Output "用 rc.exe: $($rc.Source)"
    & $rc.Source /nologo /fo $outFile $rcFile
} elseif ($windres) {
    Write-Output "用 windres: $($windres.Source)"
    # --codepage=65001 让 app.rc 里的中文按 UTF-8 解析
    & $windres.Source --codepage=65001 -O res -i $rcFile -o $outFile
} else {
    throw "找不到资源编译器。需要 rc.exe (Windows SDK) 或 windres (MSYS2: pacman -S mingw-w64-ucrt-x86_64-binutils)"
}

if ($LASTEXITCODE -ne 0) { throw "资源编译失败，退出码 $LASTEXITCODE" }

$size = (Get-Item $outFile).Length
Write-Output "已生成 $outFile ($size 字节)"
Write-Output "记得重新 cargo build 以把新资源编进 exe。"
