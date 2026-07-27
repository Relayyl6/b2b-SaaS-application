# ============================================================
# test-env-setup.ps1
# Sets up the full test environment and runs cargo test
# Usage: pwsh .\scripts\test-env-setup.ps1
# ============================================================

$ErrorActionPreference = "Stop"
$WORKSPACE = $PSScriptRoot | Split-Path -Parent

Write-Host "=== CaaS Platform — Test Environment Setup ===" -ForegroundColor Cyan

# ──────────────────────────────────────────────────────────
# 1. Check Docker is running
# ──────────────────────────────────────────────────────────
Write-Host "`n[1/5] Checking Docker..." -ForegroundColor Yellow
try {
    docker ps | Out-Null
    Write-Host "    Docker is running." -ForegroundColor Green
} catch {
    Write-Error "Docker is not running. Start Docker Desktop first."
    exit 1
}

# ──────────────────────────────────────────────────────────
# 2. Start required services (postgres + redis + rabbitmq)
# ──────────────────────────────────────────────────────────
Write-Host "`n[2/5] Starting infrastructure services..." -ForegroundColor Yellow
Set-Location $WORKSPACE
docker compose up -d postgres postgres_control redis rabbitmq

Write-Host "    Waiting for Postgres to be healthy..." -ForegroundColor Gray
$retries = 0
do {
    Start-Sleep -Seconds 2
    $health = docker inspect --format='{{.State.Health.Status}}' caas_postgres 2>$null
    $retries++
    if ($retries -gt 30) { Write-Error "Postgres never became healthy"; exit 1 }
} while ($health -ne "healthy")
Write-Host "    Postgres is healthy." -ForegroundColor Green

$retries = 0
do {
    Start-Sleep -Seconds 2
    $health = docker inspect --format='{{.State.Health.Status}}' caas_redis 2>$null
    $retries++
    if ($retries -gt 30) { Write-Error "Redis never became healthy"; exit 1 }
} while ($health -ne "healthy")
Write-Host "    Redis is healthy." -ForegroundColor Green

# ──────────────────────────────────────────────────────────
# 3. Inject test environment variables
# ──────────────────────────────────────────────────────────
Write-Host "`n[3/5] Setting test environment variables..." -ForegroundColor Yellow

$env:DATABASE_URL                = "postgres://commerce_user:commerce_secret@localhost:5432/commerce_shared"
$env:CONTROL_PLANE_DATABASE_URL  = "postgres://control_user:control_secret@localhost:5433/commerce_control"
$env:ANALYTICS_DATABASE_URL      = "postgres://analytics_user:analytics_secret@localhost:5434/commerce_analytics"
$env:REDIS_URL                   = "redis://127.0.0.1:6379/"
$env:AMQP_ADDR                   = "amqp://guest:guest@127.0.0.1:5672/%2f"
$env:SECRET                      = "test_jwt_secret_key_min_32_chars_here"
$env:STRIPE_SECRET_KEY           = "sk_test_placeholder"
$env:STRIPE_WEBHOOK_SECRET       = "whsec_test_placeholder"
$env:SENDGRID_API_KEY            = "SG.test_placeholder"
$env:NOTIFICATION_DRY_RUN        = "true"
$env:RUST_LOG                    = "error"

Write-Host "    Environment variables set." -ForegroundColor Green

# ──────────────────────────────────────────────────────────
# 4. Run sqlx migrations (if sqlx-cli is installed)
# ──────────────────────────────────────────────────────────
Write-Host "`n[4/5] Running database migrations..." -ForegroundColor Yellow
if (Get-Command "sqlx" -ErrorAction SilentlyContinue) {
    # Run migrations for each service that has them
    $services = @("user-management", "order-service", "inventory-management", 
                  "product-catalog", "payments", "logistics", "notifications",
                  "analytics", "supplier-management")
    foreach ($svc in $services) {
        $migPath = Join-Path $WORKSPACE $svc "migrations"
        if (Test-Path $migPath) {
            Write-Host "    Migrating $svc..." -ForegroundColor Gray
            $env:DATABASE_URL = "postgres://commerce_user:commerce_secret@localhost:5432/commerce_shared"
            sqlx migrate run --source $migPath 2>&1 | Write-Host
        }
    }
    Write-Host "    Migrations complete." -ForegroundColor Green
} else {
    Write-Host "    sqlx-cli not found — skipping migrations." -ForegroundColor DarkYellow
    Write-Host "    Install with: cargo install sqlx-cli --no-default-features --features native-tls,postgres" -ForegroundColor Gray
}

# ──────────────────────────────────────────────────────────
# 5. Run the full test suite
# ──────────────────────────────────────────────────────────
Write-Host "`n[5/5] Running cargo test --workspace -j 1..." -ForegroundColor Yellow
Set-Location $WORKSPACE

cargo test --workspace -j 1 2>&1
$exitCode = $LASTEXITCODE

if ($exitCode -eq 0) {
    Write-Host "`n=== ALL TESTS PASSED ===" -ForegroundColor Green
} else {
    Write-Host "`n=== TESTS FAILED (exit code: $exitCode) ===" -ForegroundColor Red
}

exit $exitCode
