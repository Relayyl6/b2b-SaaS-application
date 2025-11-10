Redis Streams to OpenSearch Indexer

A high-performance Rust service that consumes events from multiple Redis streams and indexes them into OpenSearch for analytics and monitoring purposes.

Overview

This service acts as a bridge between Redis streams and OpenSearch, continuously listening for new events in Redis and automatically indexing them in OpenSearch. It's designed for real-time analytics processing in a microservices architecture.

Features

· Multi-stream Consumption: Simultaneously listens to multiple Redis streams
· Real-time Indexing: Automatically indexes events into OpenSearch as they arrive
· Fault Tolerant: Handles connection errors and continues processing
· Structured Logging: Provides clear console output for monitoring
· Configurable: Easy configuration through environment variables

Supported Streams

The service listens to the following Redis streams by default:

· orders_stream
· inventory_stream
· suppliers_stream
· restaurants_stream
· payments_stream

Prerequisites

· Rust 1.70+ and Cargo
· Redis server with streams enabled
· OpenSearch cluster
· Tokio runtime

Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd <project-directory>
```

1. Build the project:

```bash
cargo build --release
```

Configuration

Create a .env file in the project root or set the following environment variables:

Required Environment Variables

```env
# Redis connection URL
REDIS_URL=redis://localhost:6379

# OpenSearch connection URL
OPENSEARCH_URL=http://localhost:9200
```

Optional Environment Variables

```env
# OpenSearch index name (default: "platform_analytics")
OPENSEARCH_INDEX=platform_analytics
```

Usage

Running the Service

```bash
# Development mode
cargo run

# Production mode
cargo run --release
```

Expected Output

When running successfully, you'll see:

```
🎧 Listening to stream: orders_stream
🎧 Listening to stream: inventory_stream
🎧 Listening to stream: suppliers_stream
🎧 Listening to stream: restaurants_stream
🎧 Listening to stream: payments_stream
✅ Indexed event: orders_stream
✅ Indexed event: inventory_stream
```

Data Flow

1. Consumption: Service connects to Redis and listens for new messages in configured streams
2. Transformation: Redis stream messages are converted to structured AnalyticsEvent objects
3. Indexing: Events are sent to OpenSearch as JSON documents
4. Acknowledgment: Success/failure messages are logged to console

Event Structure

Each event indexed to OpenSearch follows this schema:

```json
{
  "event_type": "stream_name",
  "payload": {
    "field1": "value1",
    "field2": "value2"
  },
  "timestamp": "2023-01-01T00:00:00Z"
}
```

Error Handling

· Redis Connection Errors: Service will retry connection automatically
· OpenSearch Indexing Failures: Failed events are logged but don't stop the service
· Malformed Messages: Messages are converted with fallback string representation

Monitoring

Check the console output for:

· ✅ Indexed event: <event_type> - Successful indexing
· ⚠️ Failed to index event: <error> - Indexing failures
· ❌ Redis read error: <error> - Redis connection issues

Development

Adding New Streams

Modify the streams vector in the main function:

```rust
let streams = vec![
    "orders_stream",
    "inventory_stream", 
    "suppliers_stream",
    "restaurants_stream",
    "payments_stream",
    "your_new_stream",  // Add new streams here
];
```

Building for Production

```bash
cargo build --release
./target/release/<binary-name>
```

Dependencies

· redis - Redis client for Rust
· reqwest - HTTP client for OpenSearch requests
· tokio - Async runtime
· serde - Serialization/deserialization
· serde_json - JSON handling
· chrono - Timestamp handling
· dotenv - Environment variable management

