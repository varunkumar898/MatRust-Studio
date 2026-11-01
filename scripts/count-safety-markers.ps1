# Counts panic/unsafe markers in production source (src/), excluding inline
# #[cfg(test)] modules. Integration tests under tests/ and benches/ are excluded.
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$crates = @()
if (Test-Path (Join-Path $root "crates")) {
    $crates += Get-ChildItem (Join-Path $root "crates") -Directory | ForEach-Object { $_.FullName }
}
$crates += (Join-Path $root "apps/desktop")

$patterns = [ordered]@{
    "unwrap()"       = '\.unwrap\(\)'
    "expect("        = '\.expect\('
    "panic!"         = '\bpanic!\s*\('
    "todo!"          = '\btodo!\s*\('
    "unimplemented!" = '\bunimplemented!\s*\('
    "unsafe"         = '\bunsafe\b'
}

"crate,production_" + (($patterns.Keys | ForEach-Object { $_ -replace '[()!]', '' }) -join ',production_')
foreach ($crate in $crates) {
    $src = Join-Path $crate "src"
    if (-not (Test-Path $src)) { continue }
    $counts = @{}
    foreach ($key in $patterns.Keys) { $counts[$key] = 0 }
    Get-ChildItem $src -Recurse -Filter *.rs | ForEach-Object {
        $text = Get-Content $_.FullName -Raw
        if ($null -eq $text) { return }
        $cut = $text.IndexOf("#[cfg(test)]")
        if ($cut -ge 0) { $text = $text.Substring(0, $cut) }
        foreach ($key in $patterns.Keys) {
            $counts[$key] += ([regex]::Matches($text, $patterns[$key])).Count
        }
    }
    $values = $patterns.Keys | ForEach-Object { $counts[$_] }
    "$(Split-Path $crate -Leaf)," + ($values -join ',')
}
