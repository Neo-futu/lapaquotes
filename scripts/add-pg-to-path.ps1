# Add PostgreSQL 18 bin to the User PATH (idempotent).
$bin = 'D:\Program Files\PostgreSQL\18\bin'
$cur = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($cur -split ';') -contains $bin) {
    Write-Host 'already present, no change'
} else {
    $new = if ($cur) { $cur.TrimEnd(';') + ';' + $bin } else { $bin }
    [Environment]::SetEnvironmentVariable('Path', $new, 'User')
    Write-Host "appended to User PATH: $bin"
}
