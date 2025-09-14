#!/usr/bin/env pwsh

Write-Host "🧪 Testing ConHub Settings API Endpoints..." -ForegroundColor Cyan

$API_BASE = "http://localhost:3001"
$USER_ID = "test-user"

# Test Settings Endpoints
Write-Host "`n📋 Testing Settings Endpoints..." -ForegroundColor Yellow

# Get Settings
Write-Host "  → GET /api/settings/$USER_ID"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID" -Method GET
    if ($response.success) {
        Write-Host "    ✅ Settings retrieved successfully" -ForegroundColor Green
    } else {
        Write-Host "    ❌ Failed to retrieve settings: $($response.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "    ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Update Settings
Write-Host "  → PUT /api/settings/$USER_ID"
$updateData = @{
    profile = @{
        first_name = "John"
        last_name = "Doe"
        email = "john.doe@example.com"
        bio = "Updated bio"
        location = "San Francisco, CA"
        website = "https://johndoe.com"
        social_links = @{
            github = "https://github.com/johndoe"
            twitter = "https://twitter.com/johndoe"
        }
    }
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID" -Method PUT -Body $updateData -ContentType "application/json"
    if ($response.success) {
        Write-Host "    ✅ Settings updated successfully" -ForegroundColor Green
    } else {
        Write-Host "    ❌ Failed to update settings: $($response.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "    ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Test API Token Endpoints
Write-Host "`n🔑 Testing API Token Endpoints..." -ForegroundColor Yellow

# Get API Tokens
Write-Host "  → GET /api/settings/$USER_ID/api-tokens"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/api-tokens" -Method GET
    if ($response.success) {
        Write-Host "    ✅ API tokens retrieved successfully" -ForegroundColor Green
    } else {
        Write-Host "    ❌ Failed to retrieve API tokens: $($response.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "    ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Create API Token
Write-Host "  → POST /api/settings/$USER_ID/api-tokens"
$tokenData = @{
    name = "Test API Token"
    permissions = @("read", "write")
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/api-tokens" -Method POST -Body $tokenData -ContentType "application/json"
    if ($response.success) {
        Write-Host "    ✅ API token created successfully" -ForegroundColor Green
        $tokenId = $response.data.id
        
        # Delete the created token
        Write-Host "  → DELETE /api/settings/$USER_ID/api-tokens/$tokenId"
        $deleteResponse = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/api-tokens/$tokenId" -Method DELETE
        if ($deleteResponse.success) {
            Write-Host "    ✅ API token deleted successfully" -ForegroundColor Green
        } else {
            Write-Host "    ❌ Failed to delete API token: $($deleteResponse.error)" -ForegroundColor Red
        }
    } else {
        Write-Host "    ❌ Failed to create API token: $($response.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "    ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Test Webhook Endpoints
Write-Host "`n🪝 Testing Webhook Endpoints..." -ForegroundColor Yellow

# Get Webhooks
Write-Host "  → GET /api/settings/$USER_ID/webhooks"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/webhooks" -Method GET
    if ($response.success) {
        Write-Host "    ✅ Webhooks retrieved successfully" -ForegroundColor Green
    } else {
        Write-Host "    ❌ Failed to retrieve webhooks: $($response.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "    ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Create Webhook
Write-Host "  → POST /api/settings/$USER_ID/webhooks"
$webhookData = @{
    name = "Test Webhook"
    url = "https://api.example.com/webhook"
    events = @("repository.sync", "ai.request")
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/webhooks" -Method POST -Body $webhookData -ContentType "application/json"
    if ($response.success) {
        Write-Host "    ✅ Webhook created successfully" -ForegroundColor Green
        $webhookId = $response.data.id
        
        # Delete the created webhook
        Write-Host "  → DELETE /api/settings/$USER_ID/webhooks/$webhookId"
        $deleteResponse = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/webhooks/$webhookId" -Method DELETE
        if ($deleteResponse.success) {
            Write-Host "    ✅ Webhook deleted successfully" -ForegroundColor Green
        } else {
            Write-Host "    ❌ Failed to delete webhook: $($deleteResponse.error)" -ForegroundColor Red
        }
    } else {
        Write-Host "    ❌ Failed to create webhook: $($response.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "    ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Test Team Endpoints
Write-Host "`n👥 Testing Team Endpoints..." -ForegroundColor Yellow

# Get Team Members
Write-Host "  → GET /api/settings/$USER_ID/team"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/team" -Method GET
    if ($response.success) {
        Write-Host "    ✅ Team members retrieved successfully" -ForegroundColor Green
    } else {
        Write-Host "    ❌ Failed to retrieve team members: $($response.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "    ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Invite Team Member
Write-Host "  → POST /api/settings/$USER_ID/team"
$memberData = @{
    email = "newmember@example.com"
    role = "member"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/team" -Method POST -Body $memberData -ContentType "application/json"
    if ($response.success) {
        Write-Host "    ✅ Team member invited successfully" -ForegroundColor Green
        $memberId = $response.data.id
        
        # Remove the invited member
        Write-Host "  → DELETE /api/settings/$USER_ID/team/$memberId"
        $deleteResponse = Invoke-RestMethod -Uri "$API_BASE/api/settings/$USER_ID/team/$memberId" -Method DELETE
        if ($deleteResponse.success) {
            Write-Host "    ✅ Team member removed successfully" -ForegroundColor Green
        } else {
            Write-Host "    ❌ Failed to remove team member: $($deleteResponse.error)" -ForegroundColor Red
        }
    } else {
        Write-Host "    ❌ Failed to invite team member: $($response.error)" -ForegroundColor Red
    }
} catch {
    Write-Host "    ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n✅ Settings API testing completed!" -ForegroundColor Green
Write-Host "📝 Check the results above for any failed tests." -ForegroundColor Cyan