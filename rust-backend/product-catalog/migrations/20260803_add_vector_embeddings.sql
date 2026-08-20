-- Add pgvector embeddings for semantic search
CREATE EXTENSION IF NOT EXISTS vector;
ALTER TABLE products ADD COLUMN IF NOT EXISTS embedding vector(1536);
