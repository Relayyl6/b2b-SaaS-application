#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8080}"
SUPPLIER_ID="${SUPPLIER_ID:-11111111-1111-1111-1111-111111111111}"
USER_ID="${USER_ID:-22222222-2222-2222-2222-222222222222}"
PRODUCT_ID="${PRODUCT_ID:-33333333-3333-3333-3333-333333333333}"

cat <<MSG
[simulation] This script assumes docker-compose stack is running.
[simulation] Steps:
  1) create order via gateway
  2) emit inventory.reserved (redis)
  3) logistics auto-creates shipment + emits logistics events to redis/rabbit
MSG

curl -sS -X POST "${BASE_URL}/api/orders/orders" \
  -H "content-type: application/json" \
  -d "{\"user_id\":\"${USER_ID}\",\"supplier_id\":\"${SUPPLIER_ID}\",\"product_id\":\"${PRODUCT_ID}\",\"qty\":1,\"items\":{\"sku\":\"demo\"}}" || true

docker compose exec -T redis redis-cli PUBLISH inventory.reserved "{\"event_type\":\"inventory.reserved\",\"order_id\":\"44444444-4444-4444-4444-444444444444\",\"user_id\":\"${USER_ID}\",\"supplier_id\":\"${SUPPLIER_ID}\",\"product_id\":\"${PRODUCT_ID}\"}"

echo "[simulation] Published inventory.reserved. Check logistics and analytics logs:"
echo "  docker compose logs -f logistics analytics"
