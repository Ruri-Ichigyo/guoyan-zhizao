# Implementation Plan: Lightweight Fruit & Vegetable Recognition System

**Specification Reference**: `.specify/specs/fruit-vegetable-recognition/spec.md`
**Plan Version**: 1.0.0
**Created**: 2025-10-13
**Status**: Approved

## Planning Overview

This plan translates the feature specification into actionable implementation steps while ensuring compliance with the project constitution (`.specify/memory/constitution.md`). The system enables users to upload produce images for AI-powered classification, view historical results, and analyze usage statistics.

## Technical Architecture Summary

**Frontend**: Vue 3 + Vite + Pinia + Element Plus + Axios
**Backend**: Rust Axum + SQLx + Tokio + Tower (circuit breaker)
**AI Service**: FastAPI + Uvicorn + PyTorch MobileNetV3 + ONNX Runtime
**Infrastructure**: Docker Compose + Redis LRU + PostgreSQL 14+

**Team Assignments**:
- Frontend (Vue3): 王喜弟 - Iteration 1
- Backend (Rust/Axum): 杨正帅 - Iteration 1
- AI Service (FastAPI/PyTorch): 吴永杰 - Iteration 2
- Infrastructure (Docker): 杨正帅 - Continuous

## Constitution Check

Before implementation begins, verify this plan satisfies all constitutional principles:

### ✅ Principle 1: Code Quality Standards

- [x] **Rust tooling**: `cargo clippy`, `cargo fmt` configured in CI
- [x] **Python tooling**: `black`, `mypy` configured for AI service
- [x] **Vue ESLint**: Composition API enforcement via `eslint-plugin-vue`

**Implementation Notes**:
- Pre-commit hooks added to enforce linting (see `quickstart.md`)
- CI pipeline blocks merge on quality gate failures
- All new Vue components MUST use `<script setup>` syntax

### ✅ Principle 2: Testing Standards

- [x] **Unit test scaffolding**: Backend (tarpaulin), AI (pytest-cov), Frontend (vitest)
- [x] **Integration tests**: Backend ↔ AI service, Backend ↔ DB/Redis
- [x] **E2E tests**: Cypress scenarios for upload, history, statistics

**Coverage Targets**:
- Backend: ≥85% (route handlers, cache logic, validation)
- AI Service: ≥85% (inference, preprocessing, error handling)
- Frontend: ≥85% (composables, utilities)

### ✅ Principle 3: User Experience Consistency

- [x] **Upload flows**: `UploadPanel.vue` implements drag-and-drop with Element Plus
- [x] **Result layouts**: `ResultsHistory.vue` uses responsive grid (Element Plus Grid)
- [x] **Error states**: `useToast.ts` composable routes all errors through Element Plus Message

**Design Tokens**: Centralized in `frontend/src/styles/tokens.ts`

### ✅ Principle 4: Performance Requirements

- [x] **AI inference budget**: Target ≤1500ms P95 (ONNX FP16 quantization + warm-up)
- [x] **Redis caching**: Cache-aside pattern, 24h TTL, LRU eviction (target >95% hit rate)
- [x] **Frontend bundle**: Lazy-load statistics dashboard, WebP thumbnails (target <3s FCP)

**Monitoring**: Prometheus metrics for latency, cache hit rate, bundle size

---

## Architecture Design

### System Components

**Affected Services**:
- [x] **Backend (Rust/Axum)**:
  - `routes/uploads.rs`: POST /api/v1/uploads (multipart upload, AI orchestration)
  - `routes/results.rs`: GET /api/v1/results (pagination), GET /api/v1/results/{id} (detail)
  - `routes/statistics.rs`: GET /api/v1/statistics (aggregation queries)
  - `cache.rs`: Redis client wrapper with circuit breaker
  - `models.rs`: `Upload`, `ClassificationResult` structs

- [x] **AI Service (FastAPI)**:
  - `main.py`: FastAPI app with `/classify` POST endpoint
  - `model.py`: ONNX model loader and inference
  - `preprocessing.py`: Image resizing, normalization

- [x] **Frontend (Vue 3)**:
  - `components/UploadPanel.vue`: Drag-and-drop upload UI
  - `components/ResultsHistory.vue`: Paginated grid with thumbnails
  - `components/ResultDetail.vue`: Full-size image with classification overlay
  - `components/StatisticsDashboard.vue`: Charts (lazy-loaded)
  - `composables/useToast.ts`: Global error notifications
  - `stores/results.ts`: Pinia store for results state

- [x] **Database (PostgreSQL)**:
  - Table `uploads`: Stores image metadata (file_path, hash, timestamp)
  - Table `classification_results`: Stores AI output (produce_type, confidence, model_version)
  - Indexes: `uploads.upload_timestamp DESC`, `classification_results.created_at DESC`

- [x] **Cache (Redis)**:
  - Key pattern: `classification:{sha256_hash}` → JSON result
  - TTL: 86400 seconds (24 hours)
  - Eviction: LRU policy

### Data Flow Diagram

```
[User] → [Frontend Upload Component]
         ↓ (multipart/form-data)
    [Backend: POST /api/v1/uploads]
         ↓
    [Validate file] → [Compute SHA-256 hash]
         ↓
    [Check Redis cache]
         ├─ HIT: Return cached result (skip AI call)
         └─ MISS: Continue ↓
    [Store file to disk] → [Insert into uploads table]
         ↓
    [Call AI Service: POST /classify]
         ↓
    [AI Service: ONNX inference]
         ↓
    [Backend receives result] → [Insert into classification_results table]
         ↓
    [Store in Redis (24h TTL)]
         ↓
    [Return JSON response to frontend]
         ↓
    [Frontend displays result]
```

### API Contracts

See `contracts/api-spec.yaml` for full OpenAPI 3.0 specification.

**Key Endpoints**:
- `POST /api/v1/uploads`: Upload image, trigger classification
- `GET /api/v1/results?page=1&limit=20`: Paginated history
- `GET /api/v1/results/{id}`: Single result detail
- `GET /api/v1/statistics`: Aggregate analytics
- `GET /health`: Component health check

**Response Format** (all endpoints):
```json
{
  "success": true/false,
  "data": { ... },
  "error": { "code": "...", "message": "...", "details": {} },
  "metadata": { "timestamp": "...", "request_id": "..." }
}
```

### Database Schema

See `artifacts/data-model.md` for detailed schema.

**Migration**: `backend/migrations/001_create_uploads_and_results.sql`

```sql
CREATE TABLE uploads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_path VARCHAR(512) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_size_bytes INTEGER NOT NULL CHECK (file_size_bytes > 0 AND file_size_bytes <= 10485760),
    image_hash VARCHAR(64) NOT NULL UNIQUE,
    upload_timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    content_type VARCHAR(50) NOT NULL CHECK (content_type IN ('image/jpeg', 'image/png', 'image/webp'))
);

CREATE TABLE classification_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    upload_id UUID NOT NULL UNIQUE REFERENCES uploads(id) ON DELETE CASCADE,
    produce_type VARCHAR(100) NOT NULL,
    confidence_score REAL NOT NULL CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    model_version VARCHAR(50) NOT NULL,
    inference_time_ms INTEGER NOT NULL CHECK (inference_time_ms > 0),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NULL
);

CREATE INDEX idx_uploads_timestamp ON uploads(upload_timestamp DESC);
CREATE INDEX idx_results_created_at ON classification_results(created_at DESC);
```

---

## Implementation Phases

### Phase 0: Research & Design (Completed)

**Artifacts Generated**:
- ✅ `artifacts/research.md`: Technology decisions and architecture patterns
- ✅ `artifacts/data-model.md`: Database schema and validation rules
- ✅ `contracts/api-spec.yaml`: OpenAPI 3.0 API contracts
- ✅ `artifacts/quickstart.md`: Developer setup guide

**Status**: Complete

---

### Phase 1: Backend Foundation (Week 1-2, 杨正帅)

**Goal**: Implement core backend API with database, caching, and AI service integration.

**Tasks**:

1. **Database Setup**
   - [ ] Create `backend/migrations/001_create_uploads_and_results.sql`
   - [ ] Run migrations: `sqlx migrate run`
   - [ ] Verify schema: `psql $DATABASE_URL -c "\dt"`

2. **Data Models** (`backend/src/models.rs`)
   - [ ] Define `Upload` struct with validation
   - [ ] Define `ClassificationResult` struct
   - [ ] Implement `From` traits for database row conversions

3. **Cache Layer** (`backend/src/cache.rs`)
   - [ ] Initialize Redis connection pool
   - [ ] Implement `get_result(hash: &str) -> Option<CachedResult>`
   - [ ] Implement `set_result(hash: &str, result: &ClassificationResult, ttl: u64)`
   - [ ] Add circuit breaker for Redis failures (degrade gracefully)

4. **Upload Route** (`backend/src/routes/uploads.rs`)
   - [ ] Parse multipart file upload (use `axum::extract::Multipart`)
   - [ ] Validate file size (≤10MB), format (JPEG/PNG/WebP)
   - [ ] Compute SHA-256 hash of file contents
   - [ ] Check Redis cache first (key: `classification:{hash}`)
   - [ ] Save file to `/uploads/YYYY/MM/DD/{uuid}.{ext}`
   - [ ] Insert into `uploads` table
   - [ ] Call AI service: `POST http://localhost:8002/classify` (with circuit breaker)
   - [ ] Insert into `classification_results` table
   - [ ] Populate Redis cache (TTL: 86400s)
   - [ ] Return JSON response

5. **Results Routes** (`backend/src/routes/results.rs`)
   - [ ] Implement `GET /api/v1/results` with pagination (page, limit query params)
   - [ ] Implement `GET /api/v1/results/{id}` detail view
   - [ ] Join `uploads` and `classification_results` tables

6. **Statistics Route** (`backend/src/routes/statistics.rs`)
   - [ ] Aggregate top produce types (GROUP BY produce_type)
   - [ ] Calculate average confidence
   - [ ] Generate upload trends (GROUP BY DATE)
   - [ ] Cache statistics for 5 minutes (reduce DB load)

7. **Health Check** (`backend/src/routes/mod.rs`)
   - [ ] Extend `GET /health` to check PostgreSQL, Redis, AI service
   - [ ] Return component-level status and response times

**Testing**:
- [ ] Unit tests for route handlers (mock DB, Redis, AI service)
- [ ] Integration tests for database operations
- [ ] Integration tests for Redis caching (hit/miss scenarios)
- [ ] Contract test for AI service `/classify` endpoint
- [ ] Coverage target: ≥85%

**Quality Gates**:
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] `cargo test` shows ≥85% coverage
- [ ] Integration tests pass against local Docker Compose

**Estimated Effort**: 10 person-days

---

### Phase 2: AI Service Integration (Week 3-4, 吴永杰)

**Goal**: Implement FastAPI inference service with MobileNetV3 + ONNX optimization.

**Tasks**:

1. **Model Preparation**
   - [ ] Fine-tune MobileNetV3 on produce dataset (1000+ images, ≥92% accuracy)
   - [ ] Export model to ONNX format: `torch.onnx.export(...)`
   - [ ] Quantize to FP16 for faster inference
   - [ ] Place `mobilenetv3.onnx` in `ai/models/`

2. **Model Wrapper** (`ai/model.py`)
   - [ ] Load ONNX model: `onnxruntime.InferenceSession("models/mobilenetv3.onnx")`
   - [ ] Implement preprocessing: resize to 224x224, normalize
   - [ ] Implement `predict(image: np.ndarray) -> Tuple[str, float]`
   - [ ] Add warm-up inference on startup (preload weights)

3. **FastAPI Server** (`ai/main.py`)
   - [ ] Create FastAPI app
   - [ ] Define `/classify` POST endpoint accepting multipart image
   - [ ] Type-annotate request/response models (Pydantic)
   - [ ] Validate image format, dimensions
   - [ ] Call model.predict()
   - [ ] Return JSON: `{produce_type: str, confidence: float, inference_time_ms: int}`

4. **Error Handling**
   - [ ] Handle invalid image format → 400 Bad Request
   - [ ] Handle model inference failure → 500 Internal Server Error
   - [ ] Handle timeout (>5s) → 504 Gateway Timeout

5. **Performance Optimization**
   - [ ] Enable ONNX Runtime optimizations (graph fusion, etc.)
   - [ ] Measure P95 latency with load testing (target ≤1500ms)
   - [ ] Profile bottlenecks with `cProfile`

**Testing**:
- [ ] Unit tests for preprocessing (mock ONNX model)
- [ ] Unit tests for inference wrapper
- [ ] Integration test: Send sample image → verify response schema
- [ ] Performance test: 50+ concurrent requests → measure P95 latency
- [ ] Coverage target: ≥85%

**Quality Gates**:
- [ ] `mypy ai/` passes without errors
- [ ] `black ai/` formatting applied
- [ ] P95 inference latency ≤1500ms (load test with k6 or locust)
- [ ] Model accuracy ≥92% on validation set

**Estimated Effort**: 12 person-days

---

### Phase 3: Frontend Development (Week 2-4, 王喜弟)

**Goal**: Build Vue 3 UI with upload, history, detail, and statistics views.

**Tasks**:

1. **Component: UploadPanel.vue**
   - [ ] Use Element Plus `<el-upload>` with drag-and-drop
   - [ ] Validate file size client-side (≤10MB)
   - [ ] Show progress bar during upload
   - [ ] Call backend: `POST /api/v1/uploads` (use Axios)
   - [ ] Display result: produce type + confidence percentage
   - [ ] Show Toast error if upload fails

2. **Component: ResultsHistory.vue**
   - [ ] Fetch paginated results: `GET /api/v1/results?page=1&limit=20`
   - [ ] Render responsive grid (Element Plus `<el-row>`, `<el-col>`)
   - [ ] Display thumbnail, produce name, confidence, timestamp
   - [ ] Implement pagination controls (Element Plus `<el-pagination>`)
   - [ ] Click card → navigate to `/results/{id}`

3. **Component: ResultDetail.vue**
   - [ ] Fetch single result: `GET /api/v1/results/{id}`
   - [ ] Display full-size image
   - [ ] Show classification details (confidence, timestamp, model version)
   - [ ] Back button → return to history

4. **Component: StatisticsDashboard.vue**
   - [ ] Lazy-load chart library (Chart.js or ECharts)
   - [ ] Fetch statistics: `GET /api/v1/statistics`
   - [ ] Render pie chart (produce distribution)
   - [ ] Render line chart (uploads over time)
   - [ ] Display key metrics (total uploads, average confidence)
   - [ ] Auto-refresh every 60 seconds

5. **Composable: useToast.ts**
   - [ ] Wrap Element Plus Message API
   - [ ] Provide `error()`, `success()`, `info()` methods

6. **Composable: useImageUpload.ts**
   - [ ] Encapsulate upload logic
   - [ ] Provide reactive state: `isUploading`, `result`, `error`

7. **Store: results.ts (Pinia)**
   - [ ] Store results list, pagination state
   - [ ] Actions: `fetchResults(page)`, `fetchResultById(id)`

**Testing**:
- [ ] Unit tests for composables (vitest)
- [ ] E2E tests (Cypress):
   - Upload workflow: Drag-drop → see result → verify in history
   - History navigation: Paginate → click detail → back to list
   - Statistics rendering: Verify charts display
   - Error handling: Upload 11MB file → see Toast error
- [ ] Coverage target: ≥85%

**Quality Gates**:
- [ ] `npm run lint` passes (ESLint Composition API rules)
- [ ] Cypress E2E tests pass
- [ ] Lighthouse audit: Performance ≥90, FCP <3s
- [ ] Bundle size: Main chunk <500KB (lazy-load statistics)

**Estimated Effort**: 10 person-days

---

### Phase 4: Caching & Optimization (Week 5, 杨正帅 + 王喜弟)

**Goal**: Optimize performance to meet constitutional budgets.

**Tasks**:

1. **Backend: Cache Hit Rate Optimization**
   - [ ] Monitor Redis `INFO stats` (keyspace_hits / keyspace_misses)
   - [ ] Adjust TTL if hit rate <95%
   - [ ] Implement cache warming on deploy (pre-populate popular images)

2. **Backend: AI Service Circuit Breaker**
   - [ ] Configure `tower` circuit breaker (5 failures → open, 60s recovery)
   - [ ] Return cached result on circuit open (if available)
   - [ ] Log circuit state transitions

3. **Frontend: Bundle Size Optimization**
   - [ ] Analyze bundle with `npm run build && npm run analyze`
   - [ ] Lazy-load `StatisticsDashboard.vue` route
   - [ ] Use WebP format for thumbnails (reduce payload)
   - [ ] Implement virtual scrolling for history (if >100 items)

4. **Frontend: Image Optimization**
   - [ ] Generate thumbnails on backend (resize to 300x300)
   - [ ] Serve thumbnails from `/uploads/.../thumb_{uuid}.webp`

**Testing**:
- [ ] Load test: 50+ concurrent uploads → measure latency, cache hit rate
- [ ] Lighthouse audit: Verify FCP <3s, Performance ≥90
- [ ] Redis monitoring: Confirm hit rate >95% after 1 hour of traffic

**Quality Gates**:
- [ ] Cache hit rate >95% in load testing
- [ ] AI inference P95 ≤1500ms
- [ ] Frontend FCP <3s (Lighthouse)

**Estimated Effort**: 4 person-days

---

### Phase 5: Testing & Documentation (Week 5, All)

**Goal**: Achieve ≥85% coverage, finalize documentation.

**Tasks**:

1. **Coverage Validation**
   - [ ] Backend: `cargo tarpaulin` → verify ≥85%
   - [ ] AI Service: `pytest --cov` → verify ≥85%
   - [ ] Frontend: `npm run test:coverage` → verify ≥85%

2. **Integration Testing**
   - [ ] End-to-end upload flow: File → Backend → AI → DB → Cache → Response
   - [ ] Database integrity: Foreign key cascade, constraint violations
   - [ ] Redis failover: Test cache miss fallback

3. **E2E Testing**
   - [ ] Finalize Cypress specs for all user workflows
   - [ ] Run in CI: `npm run test:e2e:ci`

4. **Documentation Updates**
   - [ ] Update `docs/API.md` from `contracts/api-spec.yaml`
   - [ ] Update architecture diagrams in `backend/uml-output/`
   - [ ] Document deployment process in `docs/DEPLOY.md`

5. **Load Testing**
   - [ ] Run k6 load test: 50+ concurrent uploads
   - [ ] Measure: AI latency (≤1500ms P95), cache hit rate (>95%), error rate (<1%)

**Quality Gates**:
- [ ] All tests pass in CI
- [ ] Documentation reviewed and approved
- [ ] Load test confirms performance budgets met

**Estimated Effort**: 5 person-days

---

## Risk Management

| Risk | Impact | Mitigation | Owner |
|------|--------|------------|-------|
| AI inference latency >1500ms | High | Use ONNX FP16 quantization, optimize preprocessing, warm-up on startup | 吴永杰 |
| Cache hit rate <95% | Medium | Increase TTL to 48h, implement cache warming on deploy | 杨正帅 |
| Model accuracy <92% | High | Validate on 1000+ image dataset before launch, establish feedback loop | 吴永杰 |
| Frontend bundle >1MB | Medium | Lazy-load statistics, use WebP thumbnails, analyze with visualizer | 王喜弟 |
| AI service downtime | High | Circuit breaker pattern, return cached results, degrade gracefully | 杨正帅 |

---

## Dependencies

**External**:
- PyTorch 2.0+ (AI service)
- ONNX Runtime 1.15+ (AI service)
- Element Plus 2.3+ (frontend)
- Chart.js or ECharts (frontend, lazy-loaded)

**Internal**:
- Existing Docker Compose setup (5 services)
- PostgreSQL database schema (migrations)
- Redis instance (LRU eviction)

**Blockers**:
- ⚠️ **Pre-trained MobileNetV3 model**: Must obtain or train on produce dataset before AI service implementation (吴永杰 responsible)

---

## Rollout Plan

1. **Development**: Feature branch `feature/fruit-vegetable-recognition`
2. **Iteration 1** (Week 1-2):
   - Backend API foundation (杨正帅)
   - Frontend UI with mocked AI service (王喜弟)
   - Merge to staging branch
3. **Iteration 2** (Week 3-4):
   - AI service integration (吴永杰)
   - Frontend connects to real AI service (王喜弟)
   - Merge to staging branch
4. **Testing** (Week 5):
   - Load testing, performance validation
   - Bug fixes
5. **Production Deploy**:
   - Gradual rollout: 10% → 50% → 100% traffic
   - Monitor: AI latency, cache hit rate, error rate
6. **Monitoring**:
   - Prometheus metrics: `upload_requests_total`, `inference_latency_seconds`, `cache_hit_rate`
   - Alerts: AI P95 >1500ms, cache <95%, error rate >1%

---

## Success Metrics

Post-deployment, track:
- **AI inference P95 ≤1500ms** (Constitutional Principle 4)
- **Redis cache hit rate >95%** (Principle 4)
- **Frontend FCP <3s** (Principle 4)
- **Unit test coverage ≥85%** (Principle 2)
- **Zero critical bugs** in first 7 days post-launch

---

## Open Issues

None. All design decisions resolved in Phase 0 (Research).

---

## Approval

- [x] Technical Lead: System Architect - 2025-10-13
- [ ] Feature Owner: Product Manager
- [ ] Team Members: 王喜弟, 杨正帅, 吴永杰

---

**Revision History**:

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-13 | System Architect | Initial plan with team assignments and iteration breakdown |
