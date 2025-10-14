# Data Model Specification

**Feature**: Lightweight Fruit & Vegetable Recognition System
**Version**: 1.0
**Date**: 2025-10-13

## Overview

This document defines the data model for the fruit/vegetable recognition system, including database entities, relationships, validation rules, and state transitions.

## Entity Relationship Diagram

```
┌─────────────────┐         ┌──────────────────────────┐
│     uploads     │1      1 │  classification_results  │
│─────────────────│◄────────┤──────────────────────────│
│ id (PK)         │         │ id (PK)                  │
│ file_path       │         │ upload_id (FK)           │
│ original_name   │         │ produce_type             │
│ file_size_bytes │         │ confidence_score         │
│ image_hash      │         │ model_version            │
│ upload_timestamp│         │ inference_time_ms        │
│ content_type    │         │ created_at               │
└─────────────────┘         │ metadata (JSONB)         │
                            └──────────────────────────┘
```

**Relationship**: One upload has exactly one classification result (1:1)

## Entity Definitions

### Entity: `uploads`

**Purpose**: Stores metadata about uploaded images.

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier (prevents enumeration attacks) |
| `file_path` | VARCHAR(512) | NOT NULL | Relative path to stored image: `/uploads/YYYY/MM/DD/{uuid}.{ext}` |
| `original_filename` | VARCHAR(255) | NOT NULL | User's original filename (for display purposes) |
| `file_size_bytes` | INTEGER | NOT NULL, CHECK (file_size_bytes > 0 AND file_size_bytes <= 10485760) | File size in bytes (max 10MB = 10*1024*1024) |
| `image_hash` | VARCHAR(64) | UNIQUE, NOT NULL | SHA-256 hash of file contents (for deduplication and cache keys) |
| `upload_timestamp` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | When image was uploaded |
| `content_type` | VARCHAR(50) | NOT NULL, CHECK (content_type IN ('image/jpeg', 'image/png', 'image/webp')) | MIME type validation |

**Indexes**:
- `idx_uploads_timestamp`: `CREATE INDEX idx_uploads_timestamp ON uploads(upload_timestamp DESC);`
  - **Purpose**: Optimize history queries (most recent first)
- `idx_uploads_hash`: `CREATE UNIQUE INDEX idx_uploads_hash ON uploads(image_hash);`
  - **Purpose**: Fast lookup for duplicate uploads, cache key generation

**Validation Rules**:
- File size: 1 byte ≤ size ≤ 10,485,760 bytes (10MB)
- Content type: Must be JPEG, PNG, or WebP
- Image hash: Must be unique (prevents duplicate storage)

**State Transitions**: N/A (uploads are immutable after creation)

---

### Entity: `classification_results`

**Purpose**: Stores AI classification results for each upload.

**Fields**:

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `upload_id` | UUID | FOREIGN KEY REFERENCES uploads(id) ON DELETE CASCADE, NOT NULL, UNIQUE | One-to-one relationship with uploads |
| `produce_type` | VARCHAR(100) | NOT NULL | Classified produce name (e.g., "apple", "banana", "tomato") |
| `confidence_score` | REAL | NOT NULL, CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0) | Model confidence (0.0-1.0 range) |
| `model_version` | VARCHAR(50) | NOT NULL | Model identifier (e.g., "mobilenetv3-v1.2.0") for tracking/A-B testing |
| `inference_time_ms` | INTEGER | NOT NULL, CHECK (inference_time_ms > 0) | Inference duration in milliseconds (for performance monitoring) |
| `created_at` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | When classification was performed |
| `metadata` | JSONB | NULL | Extensible field for additional model output (e.g., top-5 predictions, bounding boxes in future) |

**Indexes**:
- `idx_results_created_at`: `CREATE INDEX idx_results_created_at ON classification_results(created_at DESC);`
  - **Purpose**: Optimize statistics aggregation queries
- `idx_results_upload_id`: `CREATE UNIQUE INDEX idx_results_upload_id ON classification_results(upload_id);`
  - **Purpose**: Enforce 1:1 relationship, fast join to uploads

**Validation Rules**:
- Confidence score: Must be between 0.0 and 1.0 (inclusive)
- Inference time: Must be positive (>0ms)
- Upload ID: Must reference existing upload (foreign key constraint)

**State Transitions**: N/A (results are immutable after creation)

**Metadata JSONB Examples**:
```json
{
  "top_predictions": [
    {"class": "apple", "confidence": 0.96},
    {"class": "pear", "confidence": 0.03},
    {"class": "peach", "confidence": 0.01}
  ],
  "preprocessing_time_ms": 45,
  "postprocessing_time_ms": 12
}
```

---

## Database Migrations

### Migration 001: Initial Schema

**File**: `backend/migrations/001_create_uploads_and_results.sql`

```sql
-- Create uploads table
CREATE TABLE IF NOT EXISTS uploads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_path VARCHAR(512) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    image_hash VARCHAR(64) NOT NULL UNIQUE,
    upload_timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    content_type VARCHAR(50) NOT NULL,

    CONSTRAINT chk_file_size CHECK (file_size_bytes > 0 AND file_size_bytes <= 10485760),
    CONSTRAINT chk_content_type CHECK (content_type IN ('image/jpeg', 'image/png', 'image/webp'))
);

-- Create classification_results table
CREATE TABLE IF NOT EXISTS classification_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    upload_id UUID NOT NULL UNIQUE REFERENCES uploads(id) ON DELETE CASCADE,
    produce_type VARCHAR(100) NOT NULL,
    confidence_score REAL NOT NULL,
    model_version VARCHAR(50) NOT NULL,
    inference_time_ms INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NULL,

    CONSTRAINT chk_confidence CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    CONSTRAINT chk_inference_time CHECK (inference_time_ms > 0)
);

-- Create indexes for performance
CREATE INDEX idx_uploads_timestamp ON uploads(upload_timestamp DESC);
CREATE UNIQUE INDEX idx_uploads_hash ON uploads(image_hash);
CREATE INDEX idx_results_created_at ON classification_results(created_at DESC);
CREATE UNIQUE INDEX idx_results_upload_id ON classification_results(upload_id);

-- Add comments for documentation
COMMENT ON TABLE uploads IS 'Stores metadata about user-uploaded produce images';
COMMENT ON TABLE classification_results IS 'Stores AI classification results for each upload';
COMMENT ON COLUMN uploads.image_hash IS 'SHA-256 hash for deduplication and Redis cache keys';
COMMENT ON COLUMN classification_results.metadata IS 'Extensible JSONB field for model output (top-k predictions, etc.)';
```

### Migration Rollback

```sql
DROP TABLE IF EXISTS classification_results CASCADE;
DROP TABLE IF EXISTS uploads CASCADE;
```

---

## Redis Cache Schema

**Purpose**: Cache classification results to reduce AI service load and improve response times.

**Key Pattern**: `classification:{image_hash}`

**Value Format**: JSON

```json
{
  "result_id": "uuid",
  "upload_id": "uuid",
  "produce_type": "apple",
  "confidence_score": 0.96,
  "model_version": "mobilenetv3-v1.2.0",
  "inference_time_ms": 1234,
  "timestamp": "2025-10-13T10:00:00Z"
}
```

**TTL**: 86400 seconds (24 hours)

**Eviction Policy**: LRU (Least Recently Used)

**Example Operations**:
```redis
# Store result
SET classification:sha256hash '{"result_id":"...","produce_type":"apple",...}' EX 86400

# Retrieve result
GET classification:sha256hash

# Check if result exists
EXISTS classification:sha256hash

# Monitor cache hit rate
INFO stats
# Look for: keyspace_hits / (keyspace_hits + keyspace_misses) > 0.95
```

---

## Query Patterns

### Query: Get paginated history (most recent first)

```sql
SELECT
    u.id AS upload_id,
    u.original_filename,
    u.file_path,
    u.upload_timestamp,
    cr.produce_type,
    cr.confidence_score,
    cr.created_at
FROM uploads u
INNER JOIN classification_results cr ON u.id = cr.upload_id
ORDER BY u.upload_timestamp DESC
LIMIT 20 OFFSET 0;
```

**Index Used**: `idx_uploads_timestamp`

**Performance**: O(1) for first page due to index scan

---

### Query: Get single result detail

```sql
SELECT
    u.id,
    u.original_filename,
    u.file_path,
    u.file_size_bytes,
    u.upload_timestamp,
    cr.produce_type,
    cr.confidence_score,
    cr.model_version,
    cr.inference_time_ms,
    cr.metadata
FROM uploads u
INNER JOIN classification_results cr ON u.id = cr.upload_id
WHERE u.id = $1;
```

**Index Used**: Primary key lookup (O(1))

---

### Query: Statistics - top produce types

```sql
SELECT
    cr.produce_type,
    COUNT(*) AS count,
    ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER (), 2) AS percentage
FROM classification_results cr
WHERE cr.created_at >= NOW() - INTERVAL '30 days'
GROUP BY cr.produce_type
ORDER BY count DESC
LIMIT 10;
```

**Index Used**: `idx_results_created_at` (filter by date range)

**Performance**: Aggregation on filtered subset (30 days of data)

---

### Query: Statistics - upload trends over time

```sql
SELECT
    DATE(u.upload_timestamp) AS date,
    COUNT(*) AS upload_count
FROM uploads u
WHERE u.upload_timestamp >= NOW() - INTERVAL '30 days'
GROUP BY DATE(u.upload_timestamp)
ORDER BY date DESC;
```

**Index Used**: `idx_uploads_timestamp`

**Performance**: Date-based aggregation on indexed column

---

## Validation Rules Summary

### Upload Validation

1. **File Size**: 1 byte ≤ size ≤ 10MB
2. **Content Type**: MIME type must be `image/jpeg`, `image/png`, or `image/webp`
3. **File Format**: Verify actual file format matches content type (magic bytes check)
4. **Image Dimensions**: Max 4096x4096 pixels (prevent memory exhaustion)
5. **Duplicate Check**: SHA-256 hash must be unique (reject duplicates or return cached result)

### Classification Result Validation

1. **Confidence Score**: Must be in range [0.0, 1.0]
2. **Inference Time**: Must be positive integer (>0ms)
3. **Model Version**: Must be non-empty string
4. **Produce Type**: Must be non-empty string (max 100 chars)

---

## Data Lifecycle

### Upload Flow

1. **User uploads image** → Backend receives file
2. **Validation**: Check file size, format, dimensions
3. **Hash computation**: SHA-256 of file contents
4. **Duplicate check**: Query `uploads.image_hash`
   - If exists: Return cached classification result (skip steps 5-7)
   - If new: Continue
5. **File storage**: Save to `/uploads/YYYY/MM/DD/{uuid}.{ext}`
6. **Database insert**: Insert into `uploads` table
7. **AI classification**: Call AI service → Insert into `classification_results`
8. **Cache population**: Store result in Redis with 24h TTL

### History Query Flow

1. **User navigates to history** → Frontend requests `GET /api/v1/results?page=1&limit=20`
2. **Backend query**: Paginated query with `ORDER BY upload_timestamp DESC`
3. **Response**: JSON array of results with pagination metadata

### Statistics Aggregation Flow

1. **User opens statistics dashboard** → Frontend requests `GET /api/v1/statistics`
2. **Backend aggregation**: Run queries for top produce, upload trends, average confidence
3. **Caching**: Cache statistics for 5 minutes (reduce database load)
4. **Response**: JSON with aggregated data

---

## Performance Considerations

### Database Optimization

- **Indexes**: All frequently queried columns indexed (timestamps, foreign keys)
- **Connection Pooling**: SQLx pool with max 20 connections (already configured)
- **Query Optimization**: Use EXPLAIN ANALYZE to verify index usage

### Cache Optimization

- **Hit Rate Target**: >95% (monitor via Redis INFO stats)
- **Key Expiration**: 24h TTL balances freshness vs. hit rate
- **Memory**: LRU eviction prevents unbounded growth

### Scalability

- **Partitioning** (future): Partition `uploads` and `classification_results` by month if dataset grows >10M rows
- **Read Replicas** (future): Add PostgreSQL read replicas for statistics queries
- **CDN** (future): Serve images from CDN to reduce backend load

---

## Testing Checklist

- [ ] Verify foreign key cascade delete (delete upload → result deleted)
- [ ] Test constraint violations (file size >10MB, confidence >1.0)
- [ ] Validate index usage with EXPLAIN ANALYZE
- [ ] Load test: Insert 10k records, measure query performance
- [ ] Test duplicate upload handling (same image_hash)
- [ ] Verify JSONB metadata queries (e.g., extract top_predictions)

---

**Reviewed By**: System Architect
**Date**: 2025-10-13
