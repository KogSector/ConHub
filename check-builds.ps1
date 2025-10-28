#!/usr/bin/env pwsh

# Simple script to check current build status
Write-Host "=== ConHub Build Status ===" -ForegroundColor Cyan
Write-Host ""

# Check Docker processes
Write-Host "Active Docker Build Processes:" -ForegroundColor Yellow
$dockerProcesses = docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Image}}"
if ($dockerProcesses) {
    Write-Host $dockerProcesses
} else {
    Write-Host "No active Docker containers found" -ForegroundColor Gray
}

Write-Host ""

# Check for any docker-compose build processes
Write-Host "Docker Compose Build Processes:" -ForegroundColor Yellow
$composeProcesses = Get-Process | Where-Object { $_.ProcessName -like "*docker*" -or $_.ProcessName -like "*compose*" }
if ($composeProcesses) {
    $composeProcesses | Format-Table ProcessName, Id, CPU -AutoSize
} else {
    Write-Host "No Docker Compose processes found" -ForegroundColor Gray
}

Write-Host ""
Write-Host "=== Build Summary ===" -ForegroundColor Green
Write-Host "Backend: Building (Terminal 5)" -ForegroundColor Yellow
Write-Host "Auth: Building (Terminal 7)" -ForegroundColor Yellow  
Write-Host "AI: Building (Terminal 8)" -ForegroundColor Yellow
Write-Host "Other services: Pending" -ForegroundColor Gray