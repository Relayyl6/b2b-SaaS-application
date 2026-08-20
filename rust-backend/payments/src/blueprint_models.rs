// Auto-generated foundational structs from blueprints
// These must be integrated into models.rs manually

use serde::{Serialize, Deserialize};

/* Blueprint API Payload 0:
// POST /api/v1/billing/ledger/transactions
  // Request
  {"account_id": "acc_123", "amount": 1000, "currency": "USD", "reference": "inv_456"}
  // Response
  {"transaction_id": "txn_789", "status": "committed"}
*/

/* Blueprint API Payload 1:
// TypeScript SDK
  const result = await client.billing.createTransaction({ ... });
*/

/* Blueprint API Payload 2:
// POST /api/v1/billing/charges/idempotent
  // Request
  {"amount": 5000, "currency": "USD"}
  // Response
  {"transaction_id": "txn_abc", "status": "committed"}
*/

/* Blueprint API Payload 3:
// TypeScript SDK
  const result = await client.billing.createChargeIdempotent({ ... });
*/

/* Blueprint API Payload 4:
// POST /api/v1/billing/metering/events
  // Request
  {"tenant_id": "t_123", "event_type": "api_call", "usage_value": 1}
  // Response
  {"event_id": "evt_123", "status": "accepted"}
*/

/* Blueprint API Payload 5:
// TypeScript SDK
  const result = await client.billing.trackUsageEvent({ ... });
*/

/* Blueprint API Payload 6:
// POST /api/v1/billing/settlements/split
  // Request
  {"payment_id": "pay_123", "routing_rules": [{"recipient": "platform", "percentage": 5}]}
  // Response
  {"split_id": "splt_123", "status": "routed"}
*/

/* Blueprint API Payload 7:
// TypeScript SDK
  const result = await client.billing.splitPayment({ ... });
*/

/* Blueprint API Payload 8:
// POST /api/v1/billing/calculations/fixed
  // Request
  {"amount": "100.0050", "currency": "USD"}
  // Response
  {"calculated_total": "100.01", "status": "success"}
*/

/* Blueprint API Payload 9:
// TypeScript SDK
  const result = await client.billing.calculateTotal({ ... });
*/

/* Blueprint API Payload 10:
// POST /api/v1/billing/events/append
  // Request
  {"command": "DepositFunds", "payload": {"amount": "50.00"}}
  // Response
  {"event_id": "ev_999", "status": "appended"}
*/

/* Blueprint API Payload 11:
// TypeScript SDK
  const result = await client.billing.appendFinancialEvent({ ... });
*/

/* Blueprint API Payload 12:
// POST /api/v1/billing/webhooks/dispatch
  // Request
  {"event_id": "evt_123", "type": "invoice.paid", "data": {}}
  // Response
  {"dispatch_id": "dsp_123", "status": "queued"}
*/

/* Blueprint API Payload 13:
// TypeScript SDK
  const result = await client.billing.dispatchWebhook({ ... });
*/

/* Blueprint API Payload 14:
// POST /api/v1/billing/tax/calculate
  // Request
  {"line_items": [], "shipping_address": {"zip": "90210", "country": "US"}}
  // Response
  {"tax_amount": 850, "jurisdiction": "CA_LA"}
*/

/* Blueprint API Payload 15:
// TypeScript SDK
  const result = await client.billing.calculateTax({ ... });
*/

/* Blueprint API Payload 16:
// POST /api/v1/billing/wallets/withdraw
  // Request
  {"wallet_id": "wal_123", "withdraw_amount": 50000}
  // Response
  {"status": "approved", "remaining_balance": 10000}
*/

/* Blueprint API Payload 17:
// TypeScript SDK
  const result = await client.billing.withdrawFunds({ ... });
*/

/* Blueprint API Payload 18:
// POST /api/v1/billing/fx/convert
  // Request
  {"from_currency": "USD", "to_currency": "EUR", "amount": 10000}
  // Response
  {"converted_amount": 9200, "rate": "0.92"}
*/

/* Blueprint API Payload 19:
// TypeScript SDK
  const result = await client.billing.convertCurrency({ ... });
*/

/* Blueprint API Payload 20:
// POST /api/v1/billing/ibans/issue
  // Request
  {"customer_id": "cus_123"}
  // Response
  {"virtual_iban": "GB00MODL12345678", "status": "issued"}
*/

/* Blueprint API Payload 21:
// TypeScript SDK
  const result = await client.billing.issueVirtualIban({ ... });
*/

/* Blueprint API Payload 22:
// POST /api/v1/billing/plans/configure
  // Request
  {"plan_id": "plan_enterprise", "tiers": [{"up_to": 100, "price": 100}]}
  // Response
  {"plan_id": "plan_enterprise", "status": "configured"}
*/

/* Blueprint API Payload 23:
// TypeScript SDK
  const result = await client.billing.configurePricingPlan({ ... });
*/

/* Blueprint API Payload 24:
// POST /api/v1/billing/risk/evaluate
  // Request
  {"card_hash": "hash_123", "ip_address": "192.168.1.1", "amount": 500000}
  // Response
  {"risk_score": 85, "action": "block"}
*/

/* Blueprint API Payload 25:
// TypeScript SDK
  const result = await client.billing.evaluateRisk({ ... });
*/

/* Blueprint API Payload 26:
// POST /api/v1/billing/audit/log
  // Request
  {"actor_id": "usr_123", "action": "UPDATE_BILLING"}
  // Response
  {"log_id": "log_123", "status": "recorded"}
*/

/* Blueprint API Payload 27:
// TypeScript SDK
  const result = await client.billing.logAuditAction({ ... });
*/

/* Blueprint API Payload 28:
// POST /api/v1/billing/payments/route
  // Request
  {"amount": 10000, "currency": "USD", "payment_method": "card_tok_123"}
  // Response
  {"gateway": "stripe", "status": "routed"}
*/

/* Blueprint API Payload 29:
// TypeScript SDK
  const result = await client.billing.routePayment({ ... });
*/

/* Blueprint API Payload 30:
// POST /api/v1/billing/disputes/handle
  // Request
  {"dispute_id": "dp_123", "evidence_text": "Service provided"}
  // Response
  {"status": "evidence_submitted", "resolution": "pending"}
*/

/* Blueprint API Payload 31:
// TypeScript SDK
  const result = await client.billing.submitDisputeEvidence({ ... });
*/

/* Blueprint API Payload 32:
// POST /api/v1/billing/dunning/retry
  // Request
  {"invoice_id": "inv_123", "attempt": 2}
  // Response
  {"status": "failed", "next_retry_at": "2026-08-21T09:00:00Z"}
*/

/* Blueprint API Payload 33:
// TypeScript SDK
  const result = await client.billing.retryPayment({ ... });
*/

/* Blueprint API Payload 34:
// POST /api/v1/billing/escrow/release
  // Request
  {"escrow_id": "esc_123", "amount": 10000}
  // Response
  {"status": "released", "transaction_id": "txn_999"}
*/

/* Blueprint API Payload 35:
// TypeScript SDK
  const result = await client.billing.releaseEscrow({ ... });
*/

/* Blueprint API Payload 36:
// POST /api/v1/billing/rules/execute
  // Request
  {"tenant_id": "t_123", "wasm_payload": "<base64>"}
  // Response
  {"status": "executed", "result_value": 1500}
*/

/* Blueprint API Payload 37:
// TypeScript SDK
  const result = await client.billing.executeCustomRule({ ... });
*/

/* Blueprint API Payload 38:
// POST /api/v1/billing/reconciliation/trigger
  // Request
  {"report_url": "s3://reports/bank.csv"}
  // Response
  {"status": "processing", "job_id": "job_123"}
*/

/* Blueprint API Payload 39:
// TypeScript SDK
  const result = await client.billing.triggerReconciliation({ ... });
*/

/* Blueprint API Payload 40:
// POST /api/v1/billing/subscriptions/upgrade
  // Request
  {"subscription_id": "sub_123", "new_plan": "enterprise"}
  // Response
  {"prorated_charge": 4500, "status": "upgraded"}
*/

/* Blueprint API Payload 41:
// TypeScript SDK
  const result = await client.billing.upgradeSubscription({ ... });
*/

/* Blueprint API Payload 42:
// POST /api/v1/billing/capital/advance
  // Request
  {"invoice_id": "inv_123", "advance_amount": 80000}
  // Response
  {"status": "funded", "fee": 2000}
*/

/* Blueprint API Payload 43:
// TypeScript SDK
  const result = await client.billing.requestCapitalAdvance({ ... });
*/

/* Blueprint API Payload 44:
// POST /api/v1/billing/cards/issue
  // Request
  {"supplier_id": "sup_123", "limit": 500000}
  // Response
  {"card_id": "card_123", "status": "issued"}
*/

/* Blueprint API Payload 45:
// TypeScript SDK
  const result = await client.billing.issueVirtualCard({ ... });
*/

/* Blueprint API Payload 46:
// POST /api/v1/billing/tax/nexus/check
  // Request
  {"region_code": "EU", "volume": 15000000}
  // Response
  {"nexus_triggered": true, "rate": "0.20"}
*/

/* Blueprint API Payload 47:
// TypeScript SDK
  const result = await client.billing.checkTaxNexus({ ... });
*/

/* Blueprint API Payload 48:
// POST /api/v1/billing/refunds/saga/start
  // Request
  {"transaction_id": "txn_123", "refund_amount": 10000}
  // Response
  {"saga_id": "saga_123", "status": "running"}
*/

/* Blueprint API Payload 49:
// TypeScript SDK
  const result = await client.billing.startRefundSaga({ ... });
*/

/* Blueprint API Payload 50:
// POST /api/v1/billing/invoices/generate_pdf
  // Request
  {"invoice_id": "inv_123"}
  // Response
  {"pdf_url": "s3://.../inv.pdf", "status": "generated"}
*/

/* Blueprint API Payload 51:
// TypeScript SDK
  const result = await client.billing.generateInvoicePdf({ ... });
*/

/* Blueprint API Payload 52:
// POST /api/v1/billing/revenue/recognize
  // Request
  {"month": "2026-08"}
  // Response
  {"recognized_amount": 10000, "status": "calculated"}
*/

/* Blueprint API Payload 53:
// TypeScript SDK
  const result = await client.billing.recognizeRevenue({ ... });
*/

/* Blueprint API Payload 54:
// POST /api/v1/billing/treasury/report
  // Request
  {"account_ids": ["acc_1", "acc_2"]}
  // Response
  {"total_balance": 5000000, "status": "generated"}
*/

/* Blueprint API Payload 55:
// TypeScript SDK
  const result = await client.billing.generateTreasuryReport({ ... });
*/

/* Blueprint API Payload 56:
// POST /api/v1/billing/ach/initiate
  // Request
  {"account_id": "acc_123", "amount": 5000000}
  // Response
  {"status": "pending", "expected_clear_date": "2026-08-25"}
*/

/* Blueprint API Payload 57:
// TypeScript SDK
  const result = await client.billing.initiateAchTransfer({ ... });
*/

/* Blueprint API Payload 58:
// POST /api/v1/billing/orders/split
  // Request
  {"order_id": "ord_123", "milestones": [30, 30, 40]}
  // Response
  {"installments_created": 3, "status": "success"}
*/

/* Blueprint API Payload 59:
// TypeScript SDK
  const result = await client.billing.splitOrderInvoices({ ... });
*/

/* Blueprint API Payload 60:
// POST /api/v1/billing/pricing/hybrid
  // Request
  {"base_fee": 5000, "percentage": "0.02", "volume": 100000}
  // Response
  {"total_charge": 7000, "status": "calculated"}
*/

/* Blueprint API Payload 61:
// TypeScript SDK
  const result = await client.billing.calculateHybridPricing({ ... });
*/

/* Blueprint API Payload 62:
// POST /api/v1/billing/payouts/crypto
  // Request
  {"wallet_address": "0x123...", "amount_usdc": 5000}
  // Response
  {"tx_hash": "0xabc...", "status": "processing"}
*/

/* Blueprint API Payload 63:
// TypeScript SDK
  const result = await client.billing.processCryptoPayout({ ... });
*/

/* Blueprint API Payload 64:
// POST /api/v1/billing/audit/chain
  // Request
  {"entry_id": "ent_123", "previous_hash": "0xabc..."}
  // Response
  {"current_hash": "0xdef...", "status": "chained"}
*/

/* Blueprint API Payload 65:
// TypeScript SDK
  const result = await client.billing.verifyAuditChain({ ... });
*/

/* Blueprint API Payload 66:
// POST /api/v1/billing/rwa/tokenize
  // Request
  {"invoice_id": "inv_123", "amount_to_fractionalize": 5000000}
  // Response
  {"token_id": "rwa_789", "tx_hash": "0xabc...", "status": "minted"}
*/

/* Blueprint API Payload 67:
// TypeScript SDK
  const result = await client.billing.tokenizeInvoice({ ... });
*/

/* Blueprint API Payload 68:
// POST /api/v1/billing/amm/swap
  // Request
  {"source_currency": "USD", "target_currency": "EUR", "amount": 1000000}
  // Response
  {"exchange_rate": "0.92", "settled_amount": 920000}
*/

/* Blueprint API Payload 69:
// TypeScript SDK
  const result = await client.billing.swapLiquidity({ ... });
*/

/* Blueprint API Payload 70:
// POST /api/v1/billing/trading/order
  // Request
  {"commodity": "STEEL_A", "order_type": "LIMIT", "price": 85000, "qty": 100}
  // Response
  {"order_id": "ord_555", "status": "PLACED"}
*/

/* Blueprint API Payload 71:
// TypeScript SDK
  const result = await client.billing.placeTradingOrder({ ... });
*/

/* Blueprint API Payload 72:
// POST /api/v1/billing/treasury/optimize
  // Request
  {"account_id": "treasury_main"}
  // Response
  {"allocated_to": ["Aave", "Compound"], "expected_apy": "4.5%"}
*/

/* Blueprint API Payload 73:
// TypeScript SDK
  const result = await client.billing.optimizeTreasuryYield({ ... });
*/

/* Blueprint API Payload 74:
// POST /api/v1/billing/insurance/quote
  // Request
  {"shipment_id": "ship_999", "risk_factors": ["weather", "port_congestion"]}
  // Response
  {"premium": 150000, "payout": 10000000}
*/

/* Blueprint API Payload 75:
// TypeScript SDK
  const result = await client.billing.quoteInsurance({ ... });
*/

/* Blueprint API Payload 76:
// POST /api/v1/billing/zkp/verify
  // Request
  {"proof": "0xabc...", "public_inputs": ["score > 800"]}
  // Response
  {"verified": true, "financing_approved": true}
*/

/* Blueprint API Payload 77:
// TypeScript SDK
  const result = await client.billing.verifyCreditProof({ ... });
*/

/* Blueprint API Payload 78:
// POST /api/v1/billing/mpc/sign
  // Request
  {"party_id": "corp_a", "partial_signature": "0x123..."}
  // Response
  {"status": "WAITING_ON_OTHERS", "threshold": "2/3"}
*/

/* Blueprint API Payload 79:
// TypeScript SDK
  const result = await client.billing.signPayrollBatch({ ... });
*/

/* Blueprint API Payload 80:
// POST /api/v1/billing/ledger/entries
  // Request
  {
    "account_id": "acc_123",
    "amount": "15000.50",
    "currency": "USD",
    "entry_type": "credit",
    "idempotency_key": "idk_999"
  }
  // Response
  {
    "entry_id": "ent_uuid",
    "balance_after": "45000.75",
    "status": "committed"
  }
*/

/* Blueprint API Payload 81:
const entry = await client.ledger.createEntry({ accountId: "acc_123", amount: 15000.5, currency: "USD" });
*/

/* Blueprint API Payload 82:
// POST /api/v1/billing/metering/events
  // Request
  {
    "subscription_id": "sub_456",
    "metric_id": "api_requests",
    "value": 500,
    "timestamp": "2026-08-19T22:00:00Z"
  }
  // Response
  {
    "status": "accepted",
    "batch_id": "batch_888"
  }
*/

/* Blueprint API Payload 83:
await client.metering.reportUsage({ subscriptionId: "sub_456", metricId: "api_requests", value: 500 });
*/

/* Blueprint API Payload 84:
// POST /api/v1/billing/dunning/schedule
  // Request
  {
    "invoice_id": "inv_777"
  }
  // Response
  {
    "dunning_id": "dun_uuid",
    "next_retry_at": "2026-08-20T14:30:00Z",
    "ml_confidence_score": 0.89
  }
*/

/* Blueprint API Payload 85:
const schedule = await client.dunning.getSchedule("inv_777");
*/

/* Blueprint API Payload 86:
// POST /api/v1/billing/taxes/estimate
  // Request
  {
    "amount": "1000.00",
    "origin_country": "US",
    "dest_country": "DE",
    "buyer_vat_id": "DE123456789"
  }
  // Response
  {
    "tax_amount": "0.00",
    "reason": "eu_reverse_charge",
    "effective_rate": "0.0"
  }
*/

/* Blueprint API Payload 87:
const tax = await client.taxes.estimate({ amount: 1000, destCountry: "DE", buyerVatId: "DE123456789" });
*/

/* Blueprint API Payload 88:
// GET /api/v1/billing/subscriptions/sub_123/churn-risk
  // Response
  {
    "risk_score": 0.85,
    "primary_factor": "api_usage_drop_30d",
    "recommended_action": "schedule_qbr"
  }
*/

/* Blueprint API Payload 89:
const risk = await client.subscriptions.getChurnRisk("sub_123");
*/

/* Blueprint API Payload 90:
// POST /api/v1/billing/checkout/composite
  // Request
  {
    "order_id": "ord_555",
    "splits": [
      { "method": "card_tok_1", "amount": "40000" },
      { "method": "wire_transfer", "amount": "60000" }
    ]
  }
  // Response
  {
    "status": "awaiting_wire",
    "payment_intent_id": "pi_777"
  }
*/

/* Blueprint API Payload 91:
const intent = await client.checkout.processComposite(orderId, splits);
*/

/* Blueprint API Payload 92:
// POST /api/v1/billing/reconciliation/run
  // Request
  {
    "bank_statement_id": "stmt_001"
  }
  // Response
  {
    "matched_count": 450,
    "unmatched_count": 12,
    "confidence_threshold": 0.95
  }
*/

/* Blueprint API Payload 93:
const results = await client.reconciliation.run("stmt_001");
*/

/* Blueprint API Payload 94:
// POST /api/v1/billing/escrow/release
  // Request
  {
    "escrow_id": "esc_888",
    "delivery_proof_hash": "sha256_hash_here"
  }
  // Response
  {
    "status": "funds_released",
    "payout_id": "po_123"
  }
*/

/* Blueprint API Payload 95:
await client.escrow.releaseFunds("esc_888", { deliveryProof: "hash" });
*/

/* Blueprint API Payload 96:
// POST /api/v1/billing/invoices/inv_1/discount
  // Request
  {
    "payment_date": "2026-08-25T00:00:00Z"
  }
  // Response
  {
    "original_amount": "10000.00",
    "discounted_amount": "9800.00",
    "apr_equivalent": "12.5"
  }
*/

/* Blueprint API Payload 97:
const offer = await client.invoices.getDiscountOffer("inv_1", "2026-08-25");
*/

/* Blueprint API Payload 98:
// POST /api/v1/billing/credit/check
  // Request
  {
    "subsidiary_id": "sub_444",
    "requested_amount": "50000.00"
  }
  // Response
  {
    "approved": true,
    "remaining_global_credit": "150000.00"
  }
*/

/* Blueprint API Payload 99:
const result = await client.credit.checkLimit("sub_444", 50000);
*/

/* Blueprint API Payload 100:
// POST /api/v1/billing/fraud/evaluate
  // Request
  {
    "ip_address": "192.168.1.1",
    "domain_age_days": 14,
    "order_volume": "250000"
  }
  // Response
  {
    "action": "manual_review",
    "risk_score": 0.92,
    "flags": ["high_volume_new_domain"]
  }
*/

/* Blueprint API Payload 101:
const eval = await client.fraud.evaluate({ ipAddress: "...", orderVolume: 250000 });
*/

/* Blueprint API Payload 102:
// GET /api/v1/billing/revrec/schedules/inv_99
  // Response
  {
    "total_revenue": "12000.00",
    "recognized_revenue": "2000.00",
    "deferred_revenue": "10000.00",
    "schedule": [
      { "month": "2026-09", "amount": "1000.00" }
    ]
  }
*/

/* Blueprint API Payload 103:
const schedule = await client.revrec.getSchedule("inv_99");
*/

/* Blueprint API Payload 104:
// POST /api/v1/billing/wallets/charge
  // Request
  {
    "wallet_id": "wal_111",
    "amount": "150.00"
  }
  // Response
  {
    "status": "success",
    "remaining_balance": "49850.00"
  }
*/

/* Blueprint API Payload 105:
const result = await client.wallets.charge("wal_111", 150.00);
*/

/* Blueprint API Payload 106:
// POST /api/v1/billing/payouts/split
  // Request
  {
    "charge_id": "ch_555"
  }
  // Response
  {
    "platform_fee": "500.00",
    "destinations": [
      { "vendor_id": "v_1", "amount": "4500.00" },
      { "vendor_id": "v_2", "amount": "5000.00" }
    ]
  }
*/

/* Blueprint API Payload 107:
await client.payouts.routeSplits("ch_555");
*/

/* Blueprint API Payload 108:
// POST /api/v1/billing/subscriptions/sub_1/upgrade
  // Request
  {
    "new_plan_id": "plan_gold",
    "effective_date": "2026-08-19T12:00:00Z"
  }
  // Response
  {
    "prorated_credit": "150.00",
    "prorated_charge": "600.00",
    "net_due": "450.00"
  }
*/

/* Blueprint API Payload 109:
const preview = await client.subscriptions.previewUpgrade("sub_1", "plan_gold");
*/

/* Blueprint API Payload 110:
// GET /api/v1/billing/audit/verify
  // Request
  {
    "record_id": "inv_123"
  }
  // Response
  {
    "verified": true,
    "hash_chain": "a1b2c3d4..."
  }
*/

/* Blueprint API Payload 111:
const isValid = await client.audit.verifyRecord("inv_123");
*/

/* Blueprint API Payload 112:
// POST /api/v1/billing/fx/lock
  // Request
  {
    "source_currency": "EUR",
    "target_currency": "USD",
    "amount": "100000.00",
    "lock_duration_days": 30
  }
  // Response
  {
    "locked_rate": "1.0950",
    "expires_at": "2026-09-18T00:00:00Z",
    "hedge_fee": "150.00"
  }
*/

/* Blueprint API Payload 113:
const fxLock = await client.fx.createLock("EUR", "USD", 100000, 30);
*/

/* Blueprint API Payload 114:
// POST /api/v1/billing/invoices/inv_1/apply-late-fees
  // Response
  {
    "days_overdue": 45,
    "fee_applied": "250.00",
    "new_total": "10250.00"
  }
*/

/* Blueprint API Payload 115:
await client.invoices.applyLateFees("inv_1");
*/

/* Blueprint API Payload 116:
// POST /api/v1/billing/treasury/sweep
  // Request
  {
    "vendor_id": "v_123"
  }
  // Response
  {
    "swept_amount": "14500.00",
    "status": "processing_ach"
  }
*/

/* Blueprint API Payload 117:
await client.treasury.triggerSweep("v_123");
*/

/* Blueprint API Payload 118:
// POST /api/v1/billing/pricing/resolve
  // Request
  {
    "customer_id": "cust_1",
    "sku": "SKU-123",
    "quantity": 1200
  }
  // Response
  {
    "unit_price": "49.16", // Blended
    "total": "59000.00",
    "rule_applied": "tier_contract_v2"
  }
*/

/* Blueprint API Payload 119:
const price = await client.pricing.resolve("cust_1", "SKU-123", 1200);
*/

/* Blueprint API Payload 120:
// GET /api/v1/billing/metering/anomalies
  // Response
  {
    "anomalies": [
      {
        "subscription_id": "sub_4",
        "metric": "bandwidth_tb",
        "deviation_sigma": 4.5,
        "auto_paused": true
      }
    ]
  }
*/

/* Blueprint API Payload 121:
const anomalies = await client.metering.getAnomalies();
*/

/* Blueprint API Payload 122:
// POST /api/v1/billing/payouts/instant
  // Request
  {
    "vendor_id": "v_7",
    "amount": "2500.00",
    "network": "fednow"
  }
  // Response
  {
    "payout_id": "po_88",
    "status": "cleared",
    "network_ref": "rtp_msg_123"
  }
*/

/* Blueprint API Payload 123:
const payout = await client.payouts.triggerInstant("v_7", 2500, "fednow");
*/

/* Blueprint API Payload 124:
// POST /api/v1/billing/invoices/consolidate
  // Request
  {
    "parent_company_id": "comp_hq",
    "billing_period": "2026-08"
  }
  // Response
  {
    "consolidated_invoice_id": "inv_master_1",
    "total_amount": "145000.00",
    "child_invoices_rolled_up": 42
  }
*/

/* Blueprint API Payload 125:
const masterInvoice = await client.invoices.consolidate("comp_hq", "2026-08");
*/

/* Blueprint API Payload 126:
// POST /api/v1/billing/disputes/disp_1/defend
  // Response
  {
    "status": "evidence_submitted",
    "win_probability": 0.82
  }
*/

/* Blueprint API Payload 127:
const defense = await client.disputes.autoDefend("disp_1");
*/

/* Blueprint API Payload 128:
// POST /api/v1/billing/spend/request
  // Request
  {
    "employee_id": "emp_1",
    "cart_total": "1200.00"
  }
  // Response
  {
    "status": "pending_approval",
    "approver_id": "mgr_1"
  }
*/

/* Blueprint API Payload 129:
const req = await client.spend.requestApproval("emp_1", 1200);
*/

