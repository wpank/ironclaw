-- Add HDC (Hyperdimensional Computing) fingerprint column to memory_documents.
-- Stores a 1280-byte binary fingerprint vector for near-duplicate detection.
ALTER TABLE memory_documents ADD COLUMN hdc_fingerprint BYTEA;
