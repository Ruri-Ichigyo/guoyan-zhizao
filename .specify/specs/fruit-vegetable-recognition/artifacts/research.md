# Technical Research & Decisions

**Feature**: Lightweight Fruit & Vegetable Recognition System
**Date**: 2025-10-13
**Status**: Approved

## Overview

This document consolidates technical research and architecture decisions for implementing the fruit/vegetable recognition system. All decisions align with the project constitution and the technical architecture specified in the plan arguments.

## Technology Stack Decisions

### Frontend Architecture

**Decision**: Vue 3 + Vite + Pinia + Element Plus

**Rationale**:
- **Vue 3**: Already adopted in project (see `frontend/src/App.vue`). Composition API aligns with Constitutional Principle 1.
- **Vite**: Fast HMR, optimized bundling for <3s FCP target (Principle 4). Already configured in project.
- **Pinia**: Modern Vue state management, lighter than Vuex. TypeScript-first design improves maintainability.
- **Element Plus**: Comprehensive UI component library. Provides drag-drop upload, grid layouts, toast notifications out-of-box (Principle 3).
- **Axios**: HTTP client already integrated in project (`frontend/src/api/http.ts`). Supports interceptors for global error handling.

**Alternatives Considered**:
- Vuex (legacy state management - rejected for Pinia's better DX)
- Ant Design Vue (more components but heavier bundle - Element Plus lighter)
- Fetch API (lacks interceptor patterns - Axios better for centralized error handling)

**Implementation Notes**:
- Use `<script setup>` syntax exclusively (Constitutional requirement)
- Centralize design tokens in `frontend/src/styles/tokens.ts`
- Lazy-load statistics dashboard to meet bundle size budget

### Backend Architecture

**Decision**: Rust Axum + SQLx + Tokio async runtime

**Rationale**:
- **Axum**: Already adopted (`backend/src/main.rs`). Ergonomic async handler, good performance for 50+ concurrent uploads.
- **SQLx**: Compile-time SQL verification, async PostgreSQL driver. Already in use (`backend/Cargo.toml`).
- **Tokio**: Industry-standard async runtime. Handles concurrent uploads efficiently without blocking.
- **Async Task Queue**: Use `tokio::spawn` for AI service calls to prevent blocking main request thread.

**Alternatives Considered**:
- Actix-web (more mature but Axum has cleaner API and tower middleware ecosystem)
- Diesel ORM (blocking I/O - SQLx async better for performance requirements)
- Synchronous architecture (rejected - cannot meet 50+ concurrent upload requirement)

**Implementation Notes**:
- Use connection pooling for PostgreSQL (already configured in `backend/src/db.rs`)
- Implement circuit breaker for AI service calls (use `tower` crate)
- Add request ID middleware for distributed tracing

### AI Service Architecture

**Decision**: FastAPI + Uvicorn + PyTorch + MobileNetV3 + ONNX Runtime

**Rationale**:
- **FastAPI**: Already adopted (`ai/main.py`). Auto-generates OpenAPI docs (Principle NFR6: 100% documentation).
- **Uvicorn**: ASGI server, handles async I/O efficiently.
- **PyTorch**: Dominant ML framework. Large model ecosystem.
- **MobileNetV3**: Lightweight CNN optimized for inference speed. Achieves ≤1500ms P95 latency requirement.
- **ONNX Runtime**: Cross-platform inference engine. Optimizes model execution (quantization, graph optimization).

**Alternatives Considered**:
- TensorFlow Serving (heavier, harder to deploy - FastAPI + ONNX simpler)
- ResNet-50 (more accurate but slower - MobileNetV3 balances accuracy/speed)
- TorchServe (complex setup - FastAPI more flexible for custom logic)

**Implementation Notes**:
- Quantize model to FP16 for faster inference
- Warm up model on service startup (pre-allocate CUDA memory if GPU available)
- Use batching if multiple concurrent requests arrive (future optimization)

**Model Selection**:
- **Primary**: MobileNetV3-Large pretrained on ImageNet, fine-tuned on produce dataset
- **Accuracy Target**: ≥92% on validation set (1000+ images)
- **Input Size**: 224x224 RGB (standard for MobileNet)
- **Classes**: 50+ common fruits/vegetables (apple, banana, tomato, cucumber, etc.)

### Infrastructure

**Decision**: Docker Compose + Redis LRU + PostgreSQL 14+

**Rationale**:
- **Docker Compose**: Already adopted (`docker-compose.yml`). Orchestrates 5 services (frontend, backend, AI, Redis, PostgreSQL).
- **Redis LRU**: Automatic eviction when cache full. Prioritizes frequently accessed results.
- **PostgreSQL 14+**: Already in use. Supports JSONB for flexible metadata storage.

**Alternatives Considered**:
- Kubernetes (overkill for initial deployment - Docker Compose sufficient)
- Memcached (no data persistence - Redis better for cache + pub/sub if needed)
- MySQL (less robust JSONB support - PostgreSQL superior for semi-structured data)

**Implementation Notes**:
- Redis key pattern: `classification:{sha256_hash}` → JSON result
- TTL: 86400 seconds (24 hours)
- PostgreSQL indexes: `uploads.upload_timestamp DESC`, `classification_results.created_at DESC`

## Architecture Patterns

### Caching Strategy: Cache-Aside Pattern

**Decision**: Backend checks Redis before calling AI service

**Flow**:
1. User uploads image
2. Backend computes SHA-256 hash
3. Check Redis: `GET classification:{hash}`
   - **Cache Hit**: Return cached result (target >95% hit rate)
   - **Cache Miss**: Call AI service → Store in Redis → Return result
4. Set TTL: 86400 seconds

**Rationale**: Reduces AI service load, improves response time for duplicate uploads.

**Monitoring**: Track cache hit rate via Redis `INFO stats` command.

### Error Handling: Circuit Breaker Pattern

**Decision**: Wrap AI service calls with circuit breaker (use `tower` crate)

**States**:
- **Closed**: Normal operation, requests pass through
- **Open**: AI service unhealthy, fail fast with cached results or user-friendly error
- **Half-Open**: Test if service recovered

**Rationale**: Prevents cascading failures when AI service down (Spec Risk #1).

**Configuration**:
- Failure threshold: 5 consecutive errors
- Timeout: 30 seconds
- Recovery interval: 60 seconds

### File Storage

**Decision**: Local filesystem with configurable upload directory

**Path**: `/uploads/{year}/{month}/{day}/{uuid}.{ext}`

**Rationale**:
- Simple for initial deployment
- Organized by date for easy cleanup
- UUID prevents collisions

**Future**: Migrate to S3/Minio for production scalability (documented in Spec Assumptions).

## Performance Optimizations

### AI Inference (<1500ms P95)

1. **Model Quantization**: FP32 → FP16 (2x speedup, minimal accuracy loss)
2. **ONNX Optimization**: Graph fusion, operator fusion
3. **Warm Start**: Preload model weights on service startup
4. **Input Preprocessing**: Resize/normalize on client if possible (reduce server CPU)

### Redis Cache (>95% Hit Rate)

1. **TTL Strategy**: 24 hours balances freshness vs. hit rate
2. **Key Design**: SHA-256 hash deduplicates identical images
3. **Eviction**: LRU ensures most popular results stay cached
4. **Monitoring**: Alert if hit rate <95% for >5 minutes

### Frontend Bundle (<3s FCP)

1. **Code Splitting**: Lazy-load `/statistics` route (Chart.js heavy)
2. **Image Optimization**: WebP thumbnails, lazy-load images
3. **Tree Shaking**: Vite automatically removes unused Element Plus components
4. **CDN**: Serve static assets from CDN in production

## Data Model Design Principles

### Database Schema

**Entity: `uploads`**
- Primary key: UUID (avoids sequential ID enumeration attacks)
- `image_hash`: SHA-256 for deduplication and cache lookups
- `file_path`: Relative path for portability
- Index: `upload_timestamp DESC` (history queries sorted newest-first)

**Entity: `classification_results`**
- Foreign key: `upload_id` (one-to-one relationship)
- `confidence_score`: FLOAT CHECK (0.0-1.0) enforces valid range
- `model_version`: Track which model produced result (important for A/B testing)
- `inference_time_ms`: Performance monitoring
- Index: `created_at DESC` (statistics aggregation)

### API Design Principles

**RESTful Conventions**:
- POST `/api/v1/uploads` - Create new classification
- GET `/api/v1/results` - List historical results (pagination)
- GET `/api/v1/results/{id}` - Get single result details
- GET `/api/v1/statistics` - Aggregate analytics
- GET `/health` - Health check

**Response Format** (consistent across all endpoints):
```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "metadata": {
    "timestamp": "2025-10-13T10:00:00Z",
    "request_id": "uuid"
  }
}
```

**Error Format**:
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "INVALID_FILE_FORMAT",
    "message": "Please upload JPEG, PNG, or WebP",
    "details": {}
  }
}
```

## Testing Strategy

### Unit Tests (≥85% Coverage)

**Backend** (`backend/tests/`):
- Route handlers: Mock database/Redis/AI service
- Image validation logic: Test size limits, format checks
- Cache layer: Mock Redis, verify hit/miss logic

**AI Service** (`ai/tests/`):
- Inference wrapper: Mock model, test preprocessing
- Input validation: Test image format handling
- Error handling: Test malformed requests

**Frontend** (`frontend/tests/unit/`):
- Composables: `useToast`, `useImageUpload`, `useResultsHistory`
- Utilities: Date formatters, confidence percentage display
- API client: Mock Axios responses

### Integration Tests

**Backend ↔ AI Service**:
- Contract test: Verify `/classify` request/response schema
- Error handling: Test AI service timeout, 503 errors
- Performance: Measure actual P95 latency

**Backend ↔ Database**:
- CRUD operations: Create/read uploads and results
- Transaction handling: Ensure atomicity
- Index usage: Verify query plans use indexes

**Backend ↔ Redis**:
- Cache hit/miss scenarios
- TTL expiration
- Connection failure handling

### E2E Tests (Cypress)

**Test Scenarios**:
1. **Upload Workflow**: Drag-drop image → see result → verify in history
2. **History Navigation**: Paginate → click detail → back to list
3. **Statistics Dashboard**: Load page → verify charts render
4. **Error Handling**: Upload 11MB file → see Toast error

## Security Considerations

### Input Validation

- File size limit: 10MB (prevent DoS)
- File format whitelist: JPEG, PNG, WebP only (prevent arbitrary file upload)
- Image dimension validation: Max 4096x4096 (prevent memory exhaustion)

### Rate Limiting

- Per-IP limit: 100 uploads/hour (prevent abuse)
- Global limit: 500 uploads/minute (protect backend)

### Sanitization

- Filenames: Strip path traversal characters (`../`, `..\\`)
- SQL injection: Use parameterized queries (SQLx default)
- XSS: Vue auto-escapes templates, but sanitize user-generated content if displayed

## Monitoring & Observability

### Metrics (Prometheus format)

**Backend**:
- `upload_requests_total` (counter)
- `inference_latency_seconds` (histogram)
- `cache_hit_rate` (gauge)
- `ai_service_errors_total` (counter)

**AI Service**:
- `inference_requests_total` (counter)
- `inference_duration_seconds` (histogram)
- `model_load_time_seconds` (gauge)

**Alerts**:
- AI inference P95 > 1500ms for >5 minutes
- Cache hit rate < 95% for >10 minutes
- AI service error rate > 5% for >2 minutes

### Logging

**Structured JSON logs**:
```json
{
  "timestamp": "2025-10-13T10:00:00Z",
  "level": "INFO",
  "request_id": "uuid",
  "service": "backend",
  "message": "Image uploaded",
  "metadata": {
    "upload_id": "uuid",
    "file_size": 2048000,
    "duration_ms": 234
  }
}
```

## Team Assignments

Based on provided resource allocation:

| Module | Owner | Iteration | Notes |
|--------|-------|-----------|-------|
| Frontend (Vue3) | 王喜弟 | Iteration 1 | Upload UI, history, statistics dashboard |
| Backend (Rust/Axum) | 杨正帅 | Iteration 1 | API routes, database, caching |
| AI Service (FastAPI/PyTorch) | 吴永杰 | Iteration 2 | Model integration, ONNX optimization |
| Infrastructure (Docker) | 杨正帅 | Continuous | CI/CD, deployment scripts |

**Iteration 1 Deliverables** (Frontend + Backend):
- Upload endpoint (mock AI service initially)
- History listing and detail views
- Database schema and migrations
- Unit tests + integration tests

**Iteration 2 Deliverables** (AI Service):
- Model training/fine-tuning on produce dataset
- FastAPI inference endpoint
- ONNX optimization
- Performance validation (≤1500ms P95)

## Open Questions (Resolved)

All technical questions resolved through research and alignment with provided architecture:

1. ~~Model selection?~~ → MobileNetV3 (balance accuracy/speed)
2. ~~Caching strategy?~~ → Cache-aside with Redis LRU
3. ~~File storage?~~ → Local filesystem (S3 in future)
4. ~~State management?~~ → Pinia (modern, TypeScript-first)
5. ~~AI service reliability?~~ → Circuit breaker pattern

## Next Steps

1. Generate `data-model.md` with detailed database schema
2. Create OpenAPI contracts in `/contracts/api-spec.yaml`
3. Update agent context with technology stack
4. Begin Iteration 1 implementation (Backend + Frontend)

---

**Approved By**: System Architect
**Date**: 2025-10-13
