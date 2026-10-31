# Use Continue so informational compiler output on stderr does not trigger NativeCommandError in PowerShell 5.1
$ErrorActionPreference = "Continue"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $repoRoot

# 1. Resolve CARGO_HOME and Cargo registry path
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE ".cargo" }
$cargoRegistry = Join-Path $cargoHome "registry"

# 2. Resolve rustup sysroot
$sysroot = (rustc --print sysroot).Trim()

# 3. Resolve syslibs directory from global config if possible
$globalConfig = Join-Path $cargoHome "config.toml"
$syslibsDir = $null
if (Test-Path $globalConfig) {
    $content = Get-Content $globalConfig -Raw
    if ($content -match 'syslibs-aarch64-gnullvm') {
        $m = [regex]::Match($content, '["'']([^"'']*syslibs-aarch64-gnullvm[^"'']*)["'']')
        if ($m.Success) {
            $syslibsDir = $m.Groups[1].Value.Replace('\\', '\')
        }
    }
}
if (-not $syslibsDir -or -not (Test-Path $syslibsDir)) {
    $syslibsDir = Join-Path $cargoHome "syslibs-aarch64-gnullvm"
}

# 4. Construct RUSTFLAGS
# Note: RUSTFLAGS env var overrides target config rustflags in cargo.
# We must include:
# - syslibs directory (-L <path>)
# - PE timestamp neutralization (-C link-arg=--no-insert-timestamp)
# - Path remappings (--remap-path-prefix=<FROM>=<TO>)
$flags = @(
    "-L $syslibsDir",
    "-C link-arg=--no-insert-timestamp",
    "--remap-path-prefix=$repoRoot=/rustlab",
    "--remap-path-prefix=$cargoRegistry=/cargo-registry",
    "--remap-path-prefix=$sysroot=/rustup"
)

$repoRootSlash = $repoRoot.Replace('\', '/')
$cargoRegistrySlash = $cargoRegistry.Replace('\', '/')
$sysrootSlash = $sysroot.Replace('\', '/')

if ($repoRootSlash -ne $repoRoot) {
    $flags += "--remap-path-prefix=$repoRootSlash=/rustlab"
}
if ($cargoRegistrySlash -ne $cargoRegistry) {
    $flags += "--remap-path-prefix=$cargoRegistrySlash=/cargo-registry"
}
if ($sysrootSlash -ne $sysroot) {
    $flags += "--remap-path-prefix=$sysrootSlash=/rustup"
}

$env:RUSTFLAGS = $flags -join " "
Write-Host "Building release with RUSTFLAGS: $env:RUSTFLAGS"

cargo build --release --locked --bin rustlab
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
