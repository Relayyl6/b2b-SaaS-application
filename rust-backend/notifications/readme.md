# Notifications Service

The notifications service is the communication and alerting boundary for the B2B backend. It turns domain events into durable notification records and can deliver them through provider adapters.

## Responsibilities

- Persist notification outbox records in Postgres.
- Consume Redis domain events from orders, inventory, logistics, payments, and users.
- Generate user/supplier-facing messages from business events.
- Retry pending delivery attempts in a background worker.
- Expose APIs for manual notifications, listing, lookup, and marking in-app notifications as read.

## Runtime

```env
DATABASE_URL=postgres://postgres:postgres@postgres:5432/notifications
REDIS_URL=redis://redis:6379
SERVICE_PORT=3009
NOTIFICATION_DRY_RUN=true
EMAIL_WEBHOOK_URL=
SMS_WEBHOOK_URL=
EXPO_PUSH_URL=https://exp.host/--/api/v2/push/send
EXPO_ACCESS_TOKEN=
```

`NOTIFICATION_DRY_RUN=true` records and marks delivery as successful without calling an external email/SMS/push provider. Set it to `false` when you want actual provider calls.

Email and SMS are webhook-backed. The service posts JSON to `EMAIL_WEBHOOK_URL` and `SMS_WEBHOOK_URL`, so you can plug in Resend, SendGrid, Mailgun, Twilio, Termii, or a small internal provider service without changing the notification database or API.

Push notifications use Expo's push API by default, which fits React Native apps well. If you later move to direct FCM/APNs, replace only the push branch in `src/provider.rs`.

## HTTP API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Health check |
| `POST` | `/notifications` | Create and attempt to send a notification |
| `GET` | `/notifications` | List notifications with filters |
| `GET` | `/notifications/{id}` | Fetch one notification |
| `PUT` | `/notifications/{id}/read` | Mark an in-app notification as read |
| `POST` | `/notification-devices` | Register/update a React Native push token |
| `GET` | `/notification-devices/user/{user_id}` | List active push devices for a user |
| `DELETE` | `/notification-devices/{id}` | Disable a push device |

### Create Notification

```json
{
  "user_id": "11111111-1111-1111-1111-111111111111",
  "order_id": "22222222-2222-2222-2222-222222222222",
  "channel": "in_app",
  "priority": "normal",
  "recipient": "user:11111111-1111-1111-1111-111111111111",
  "subject": "Order received",
  "body": "Your order is pending inventory reservation.",
  "payload": {
    "source": "support"
  }
}
```

### React Native Push Registration

Your React Native app should ask the OS for permission, obtain an Expo push token, then register it:

```json
POST /notification-devices

{
  "user_id": "11111111-1111-1111-1111-111111111111",
  "platform": "android",
  "push_token": "ExponentPushToken[xxxxxxxxxxxxxxxxxxxxxx]",
  "provider": "expo",
  "device_id": "optional-device-id",
  "app_version": "1.0.0"
}
```

To send push to all active devices for a user, omit `recipient` and pass `channel: "push"`:

```json
POST /notifications

{
  "user_id": "11111111-1111-1111-1111-111111111111",
  "channel": "push",
  "priority": "high",
  "subject": "Shipment update",
  "body": "Your order is now in transit.",
  "payload": {
    "screen": "OrderDetails",
    "order_id": "22222222-2222-2222-2222-222222222222"
  }
}
```

The response is an array of notification records, one per active device. If there are no registered devices, the service returns `202 Accepted` with a message.

You can also send to one raw Expo token by setting `recipient` to that token.

### Email and SMS

Email:

```json
{
  "channel": "email",
  "recipient": "buyer@example.com",
  "subject": "Order received",
  "body": "Your order is pending inventory reservation."
}
```

SMS:

```json
{
  "channel": "sms",
  "recipient": "+2348012345678",
  "body": "Your shipment is now in transit."
}
```

### List Notifications

```text
GET /notifications?user_id=<uuid>&status=sent&limit=50&offset=0
```

## Event Subscriptions

The service listens to:

- `order.created`
- `order.cancelled`
- `inventory.lowstock`
- `inventory.rejected`
- `logistics.shipment_created`
- `logistics.shipment_updated`
- `logistics.shipment_cancelled`
- `payment.failed`
- `payment.success`
- `user.created`

## Scaling Notes

The HTTP API can scale horizontally behind the API gateway. The Redis Pub/Sub listener should usually run as a single replica unless events are idempotent or the platform moves to Redis Streams/Kafka consumer groups. The delivery worker is safe to scale only after adding row-level claiming, for example `FOR UPDATE SKIP LOCKED`, to avoid two replicas sending the same pending notification.
