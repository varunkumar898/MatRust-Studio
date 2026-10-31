$base = $PSScriptRoot
$logPath = Join-Path $base "verify-a312-final.log"
$log = Get-Content $logPath

$step = ''
$unit = ''
$ordered = New-Object System.Collections.Generic.List[string]
$a2Names = @{}
$crateUnits = @{}

$crateOfUnit = {
  param($sequence)
  $crateOrder = @()
  # Will be filled after discovering boundaries: use known header order mapping
  $crate
}

foreach ($line in $log) {
  if ($line -cmatch '^=== (\d+)\.') { $step = $Matches[1]; $crateIdx = 0; continue }
  if ($line -cmatch '^\s*Running unittests src\\lib\.rs\s*$' -or $line -cmatch '^\s*Running unittests src\\main\.rs\s*$') {
    $unit = $line.Trim()
    continue
  }
  if ($line -cmatch '^\s*(Running|Doc-tests)\s+(.*)$') { $unit = $line.Trim(); continue }
  if ($line -cmatch '^test result: ok\. (\d+) passed') {
    $ordered.Add(('STEP ' + $step + ' | ' + $unit + ' | ' + $Matches[1] + ' passed'))
    continue
  }
  if ($line -cmatch '^test (\S+) \.\.\. ok') {
    if ($unit -eq 'Running tests\stdlib_a2_tests.rs') {
      if (-not $a2Names.ContainsKey($step)) { $a2Names[$step] = New-Object System.Collections.Generic.List[string] }
      $a2Names[$step].Add($Matches[1])
    }
    continue
  }
}

$ordered | Set-Content ("$base\verify-ordered-units.txt")

$formatted = New-Object System.Collections.Generic.List[string]
foreach ($k in '4', '5', '6') {
  $formatted.Add('==== a2 (stdlib_a2_tests.rs) names, STEP ' + $k + ' (' + $a2Names[$k].Count + ') ====')
  $a2Names[$k] | Sort-Object | ForEach-Object { $formatted.Add($_) }
  $formatted.Add('')
}
# explicit set diff 4 vs 6 for a2
$s4 = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
foreach ($n in $a2Names['4']) { [void]$s4.Add($n) }
$s6 = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
foreach ($n in $a2Names['6']) { [void]$s6.Add($n) }
$formatted.Add('==== a2 set diff (in 4, not in 6) ==== ' + ($s4.Count - $s6.Count) + ' / dup-count-4: ' + $a2Names['4'].Count + ' distinct-4: ' + $s4.Count)
foreach ($n in $s4) { if (-not $s6.Contains($n)) { $formatted.Add('MISSING-IN-6: ' + $n) } }
$formatted.Add('dupes within step4 a2 raw list: ' + ($a2Names['4'].Count - $s4.Count))
$formatted | Set-Content ("$base\verify-a2-diff.txt")
Write-Output 'A2_DONE'
