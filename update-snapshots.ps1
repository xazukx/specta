Get-ChildItem -Path . -Directory -Recurse -Filter "snapshots" | ForEach-Object {
    Get-ChildItem -Path $_.FullName -File -Filter "*.snap.new" | ForEach-Object {
        $newName = $_.FullName -replace '\.snap\.new$', '.snap'
        Move-Item -Path $_.FullName -Destination $newName -Force
        Write-Host "Renamed: $($_.FullName) -> $newName"
    }
}
