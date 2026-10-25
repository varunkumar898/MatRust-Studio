$logPath = 'C:\Users\varun\.antigravity\rustlab\bench-results\a312\verify-a312-final.log'
$base = 'C:\Users\varun\.antigravity\rustlab\bench-results\a312'
$log = Get-Content $logPath

$step = ''
$unit = ''
$out = New-Object System.Collections.Generic.List[string]
$names = @{}
$counts = @{}

function UnitIdentity($unit) {
  # Normalize "     Running unittests src\lib.rs (C:\...\rustlab_math-abcdef0123456789.exe)"
  if ($unit -match 'Running\s+(unittests\s*)?\s*(.*?)\s*\(([^)]+)\)') {
    $exe = Split-Path -Leaf $Matches[3]
    $crate = ''
    if ($exe -match '^([a-zA-Z0-9_]+)-[0-9a-f]{8,}\.exe') { $crate = $Matches[1] }
    return 'RUN:' + $crate + ':' + $Matches[2]
  }
  if ($unit -match 'Doc-tests\s+(.*)') { return 'DOC:' + $Matches[1] }
  if ($unit -match '^Running\s+(.*)$') { return 'RUN:' + $Matches[1] }
  return 'UNKNOWN:' + $unit
}

foreach ($line in $log) {
  if ($line -cmatch '^=== .* ===') {
    $step = $line.Trim()
    $out.Add('')
    $out.Add($step)
  } elseif ($line -cmatch '^\s*(Running\s+|Doc-tests\s+)') {
    $unit = $line.Trim()
    $ident = UnitIdentity $unit
    $out.Add('  [' + $unit + '  =>  ' + $ident + ']')
  } elseif ($line -match '^test result: (.*)$') {
    $counts[$step + '|' + $unit] = $Matches[1]
    $out.Add('    test result: ' + $Matches[1])
  } elseif ($line -match '^test (\S+) \.\.\. (ok|FAILED)') {
    $out.Add('      ' + $line.Trim())
    if ($Matches[2] -eq 'ok') {
      $ident = UnitIdentity $unit
      $key = $step + '|' + $ident
      if (-not $names.ContainsKey($key)) { $names[$key] = New-Object System.Collections.Generic.List[string] }
      $names[$key].Add($Matches[1])
    }
  }
}
$out | Set-Content ("$base\verify-extract.txt")

# Step totals: sum of "N passed" over all result lines within each step
$totals = New-Object System.Collections.Generic.List[string]
$byStepName = @{}
foreach ($e in $counts.GetEnumerator()) {
  $s = ''
  if ($e.Key -match '^=== (\d+)\.') { $s = $Matches[1] }
  if ($e.Value -cmatch 'ok\. (\d+) passed') {
    $n = [int]$Matches[1]
    if ($n -gt 0 -or $e.Value -cmatch 'running') {
      if (-not $byStepName.ContainsKey($s)) { $byStepName[$s] = New-Object System.Collections.Generic.List[string] }
      $byStepName[$s].Add(($e.Key.Trim() + ' | ' + $e.Value))
    }
  }
}
foreach ($s in '4', '5', '6') {
  $total = 0
  $units = 0
  if ($byStepName.ContainsKey($s)) {
    foreach ($e in $byStepName[$s]) {
      if ($e -cmatch '\| ok\. (\d+) passed') { $total += [int]$Matches[1]; $units++ }
    }
  }
  $totals.Add(('STEP ' + $s + ': ' + $units + ' units, TOTAL PASSED = ' + $total))
}
$totals | Set-Content ("$base\verify-totals.txt")

# Per-unit detail for steps 4/5/6
$detail = New-Object System.Collections.Generic.List[string]
foreach ($s in '4', '5', '6') {
  $detail.Add('')
  $detail.Add('########## STEP ' + $s + ' ##########')
  foreach ($k in $names.Keys) {
    $parts = $k -split '\|', 2
    $s2 = ''
    if ($parts[0] -match '^=== (\d+)\.') { $s2 = $Matches[1] }
    if ($s2 -eq $s) { $detail.Add('[ ' + $parts[1] + ' ]  named=' + $names[$k].Count) }
  }
}
$detail | Set-Content ("$base\verify-units.txt")

# Name-set diff step4 vs step6 by unit identity
$diff = New-Object System.Collections.Generic.List[string]
$s4u = @{}; $s6u = @{}; $s5u = @{}
foreach ($k in $names.Keys) {
  $parts = $k -split '\|', 2
  $s = ''
  if ($parts[0] -match '^=== (\d+)\.') { $s = $Matches[1] }
  if ($s -eq '4') { $s4u[$parts[1]] = $names[$k] }
  if ($s -eq '5') { $s5u[$parts[1]] = $names[$k] }
  if ($s -eq '6') { $s6u[$parts[1]] = $names[$k] }
}
$diff.Add('== Units with names: step4=' + $s4u.Keys.Count + ' step5=' + $s5u.Keys.Count + ' step6=' + $s6u.Keys.Count)
$diff.Add('')
$diff.Add('== Per-unit name-count comparisons (4 vs 6), showing units where 6 has fewer names: ==')
foreach ($u in $s4u.Keys) {
  if ($s6u.ContainsKey($u)) {
    if ($s6u[$u].Count -lt $s4u[$u].Count) {
      $diff.Add('UNIT ' + $u + ' : ' + $s4u[$u].Count + ' vs ' + $s6u[$u].Count)
      foreach ($n in $s4u[$u]) { if (-not ($s6u[$u] -contains $n)) { $diff.Add('   MISSING-IN-6: ' + $n) } }
    }
  } else {
    $diff.Add('UNIT ABSENT-IN-6: ' + $u + ' (had ' + $s4u[$u].Count + ' tests)')
  }
}
$diff.Add('')
$diff.Add('== Sanity: units in 6 not in 4 (should be none): ==')
foreach ($u in $s6u.Keys) { if (-not $s4u.ContainsKey($u)) { $diff.Add('ONLY-IN-6: ' + $u) } }
$diff | Set-Content ("$base\verify-stepdiff.txt")
Write-Output 'EXTRACT_DONE'
