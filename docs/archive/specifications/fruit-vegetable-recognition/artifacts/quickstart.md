# Implementation Quickstart

**Feature**: Lightweight Fruit & Vegetable Recognition System
**Branch**: `feature/fruit-vegetable-recognition`
**For**: 王喜弟 (Frontend), 杨正帅 (Backend), 吴永杰 (AI Service)

## Prerequisites

- Docker Desktop / Docker Engine 20.10+
- Docker Compose v2
- Rust 1.70+ (backend development)
- Python 3.10+ (AI service development)
- Node.js 18+ (frontend development)

## Project Structure

```
.
├── backend/              # Rust Axum backend (杨正帅)
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/
│   │   │   ├── uploads.rs    # POST /api/v1/uploads
│   │   │   ├── results.rs    # GET /api/v1/results, /api/v1/results/{id}
│   │   │   └── statistics.rs # GET /api/v1/statistics
│   │   ├── models.rs          # Data models (Upload, ClassificationResult)
│   │   ├── db.rs              # Database connection pool
│   │   └── cache.rs           # Redis client
│   ├── migrations/
│   │   └── 001_create_uploads_and_results.sql
│   └── Cargo.toml
├── ai/                   # FastAPI + PyTorch (吴永杰)
│   ├── main.py           # FastAPI server
│   ├── model.py          # MobileNetV3 inference
│   ├── preprocessing.py  # Image preprocessing
│   └── requirements.txt
├── frontend/             # Vue 3 + Vite (王喜弟)
│   ├── src/
│   │   ├── components/
│   │   │   ├── UploadPanel.vue      # Image upload UI
│   │   │   ├── ResultsHistory.vue   # Paginated grid
│   │   │   ├── ResultDetail.vue     # Detail view
│   │   │   └── StatisticsDashboard.vue # Charts
│   │   ├── composables/
│   │   │   ├── useToast.ts          # Error notifications
│   │   │   └── useImageUpload.ts    # Upload logic
│   │   └── stores/
│   │       └── results.ts           # Pinia store
│   └── package.json
└── docker-compose.yml
```

## Quick Start

### 1. Start Infrastructure

```bash
# Clone repo (if not already)
cd /home/lanshi/Projects/Applications/Web/guoyan-zhizao

# Checkout feature branch
git checkout feature/fruit-vegetable-recognition

# Copy environment variables
cp .env.example .env

# Start all services
./scripts/start.sh
```

**Services Running**:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080/api/v1
- AI Service: http://localhost:8002
- PostgreSQL: `postgres://gyz:gyz123@localhost:5432/gyz`
- Redis: `redis://localhost:6379`

### 2. Run Database Migrations

```bash
cd backend
export DATABASE_URL=postgres://gyz:gyz123@localhost:5432/gyz
sqlx migrate run
```

**Verify**:
```bash
psql $DATABASE_URL -c "\dt"
# Should show: uploads, classification_results
```

### 3. Development Workflow

#### Backend Development (杨正帅)

```bash
cd backend

# Install dependencies
cargo build

# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Start development server
export DATABASE_URL=postgres://gyz:gyz123@localhost:5432/gyz
export REDIS_URL=redis://localhost:6379
export AI_SERVICE_URL=http://localhost:8002
cargo run

# Backend will start on http://localhost:8080
```

**Key Files**:
- `.specify/specs/fruit-vegetable-recognition/artifacts/data-model.md` - Database schema
- `.specify/specs/fruit-vegetable-recognition/contracts/api-spec.yaml` - API contracts

#### AI Service Development (吴永杰)

```bash
cd ai

# Create virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Run type checker
mypy main.py model.py

# Format code
black .

# Start development server
uvicorn main:app --reload --host 0.0.0.0 --port 8002

# AI service will start on http://localhost:8002
```

**Model Setup**:
1. Download MobileNetV3 pretrained weights
2. Fine-tune on produce dataset (see `ai/training/README.md`)
3. Export to ONNX format: `python export_onnx.py`
4. Place `mobilenetv3.onnx` in `ai/models/`

#### Frontend Development (王喜弟)

```bash
cd frontend

# Install dependencies
npm install

# Run linter
npm run lint

# Run development server
npm run dev

# Frontend will start on http://localhost:5173
# Vite will proxy API requests to http://localhost:8080
```

**Key Components**:
- `UploadPanel.vue` - Drag-and-drop upload with progress (Constitutional Principle 3)
- `ResultsHistory.vue` - Responsive grid layout (Principle 3)
- `StatisticsDashboard.vue` - Lazy-loaded charts (Principle 4: bundle optimization)
- `useToast.ts` - Global error notification (Principle 3)

## Implementation Phases

### Phase 1: Backend Foundation (Week 1-2, 杨正帅)

**Tasks**:
1. Implement `backend/src/models.rs`:
   ```rust
   pub struct Upload {
       pub id: Uuid,
       pub file_path: String,
       pub original_filename: String,
       pub file_size_bytes: i32,
       pub image_hash: String,
       pub upload_timestamp: DateTime<Utc>,
       pub content_type: String,
   }

   pub struct ClassificationResult {
       pub id: Uuid,
       pub upload_id: Uuid,
       pub produce_type: String,
       pub confidence_score: f32,
       pub model_version: String,
       pub inference_time_ms: i32,
       pub created_at: DateTime<Utc>,
       pub metadata: Option<serde_json::Value>,
   }
   ```

2. Implement `backend/src/routes/uploads.rs`:
   - POST handler for multipart file upload
   - SHA-256 hash computation
   - Redis cache check (key: `classification:{hash}`)
   - AI service call (HTTP POST to `http://localhost:8002/classify`)
   - Database insert (uploads + classification_results)
   - Cache population (TTL: 86400 seconds)

3. Implement Redis caching layer (`backend/src/cache.rs`)
4. Add circuit breaker for AI service calls (use `tower` crate)

**Testing**:
- Unit tests: Route handlers, cache logic (mock Redis)
- Integration tests: Database operations, AI service contract
- Target: ≥85% coverage (Constitutional Principle 2)

**Quality Gates**:
- [ ] `cargo clippy` passes without warnings
- [ ] `cargo test` shows ≥85% coverage
- [ ] Integration test succeeds for upload flow

---

### Phase 2: AI Service Integration (Week 3-4, 吴永杰)

**Tasks**:
1. Implement `ai/model.py`:
   ```python
   import onnxruntime as ort
   from typing import Tuple

   class ProduceClassifier:
       def __init__(self, model_path: str):
           self.session = ort.InferenceSession(model_path)
           self.input_shape = (224, 224)

       def predict(self, image: np.ndarray) -> Tuple[str, float]:
           """Returns (produce_type, confidence)"""
           preprocessed = self.preprocess(image)
           outputs = self.session.run(None, {"input": preprocessed})
           class_idx = np.argmax(outputs[0])
           confidence = float(outputs[0][0][class_idx])
           return self.idx_to_class[class_idx], confidence
   ```

2. Implement `ai/main.py`:
   - FastAPI app with `/classify` POST endpoint
   - Type-annotated request/response models (Principle 1)
   - Error handling (invalid image, model failure)

3. Optimize inference:
   - Model quantization (FP16)
   - Warm-up on startup (preload weights)
   - Measure P95 latency (target ≤1500ms)

**Testing**:
- Unit tests: Preprocessing, model inference (mock ONNX)
- Integration tests: Backend ↔ AI service contract
- Performance tests: Load test with 50+ concurrent requests

**Quality Gates**:
- [ ] `mypy` type check passes
- [ ] `black` formatting applied
- [ ] P95 latency ≤1500ms in test environment

---

### Phase 3: Frontend Development (Week 2-4, 王喜弟)

**Tasks**:
1. Enhance `UploadPanel.vue`:
   ```vue
   <script setup lang="ts">
   import { ref } from 'vue'
   import { ElUpload, ElMessage } from 'element-plus'
   import { useImageUpload } from '@/composables/useImageUpload'

   const { upload, isUploading, result } = useImageUpload()

   const handleDrop = async (file: File) => {
     if (file.size > 10 * 1024 * 1024) {
       ElMessage.error('Image must be under 10MB, please resize')
       return
     }
     await upload(file)
   }
   </script>
   ```

2. Implement `ResultsHistory.vue`:
   - Paginated grid layout (20 results per page)
   - Responsive design (mobile/tablet/desktop)
   - Click → navigate to detail view

3. Implement `StatisticsDashboard.vue`:
   - Lazy-load Chart.js or ECharts
   - Pie chart (produce distribution)
   - Line chart (uploads over time)
   - Auto-refresh every 60 seconds

4. Implement `useToast.ts` composable:
   ```typescript
   export function useToast() {
     return {
       error: (message: string) => ElMessage.error(message),
       success: (message: string) => ElMessage.success(message),
       info: (message: string) => ElMessage.info(message),
     }
   }
   ```

**Testing**:
- Unit tests: Composables, utility functions
- E2E tests (Cypress):
  - Upload workflow
  - History navigation
  - Statistics rendering

**Quality Gates**:
- [ ] ESLint passes (Composition API enforced)
- [ ] Cypress E2E tests succeed
- [ ] Lighthouse performance: FCP <3s

---

### Phase 4: Testing & Documentation (Week 5)

**Tasks**:
1. Achieve unit test coverage ≥85% across all modules
2. Complete integration test suite (backend ↔ AI ↔ DB ↔ Redis)
3. Finalize E2E test scenarios (Cypress)
4. Update API documentation (`docs/API.md`) from OpenAPI spec
5. Update architecture diagrams (`backend/uml-output/`)
6. Load testing: Validate 50+ concurrent uploads, cache hit rate >95%

**Quality Gates**:
- [ ] All tests pass in CI
- [ ] Documentation reviewed
- [ ] Performance budgets met (AI ≤1500ms, cache >95%, FCP <3s)

---

## Testing Commands

### Backend Tests
```bash
cd backend
cargo test                           # Run all tests
cargo test --test integration       # Integration tests only
cargo tarpaulin --out Html           # Coverage report
```

### AI Service Tests
```bash
cd ai
pytest tests/ -v                     # Run all tests
pytest --cov=. --cov-report=html     # Coverage report
```

### Frontend Tests
```bash
cd frontend
npm run test:unit                    # Vitest unit tests
npm run test:e2e                     # Cypress E2E tests
npm run test:coverage                # Coverage report
```

## Performance Validation

### Measure AI Inference Latency
```bash
# Load test with k6
k6 run tests/load/inference_test.js

# Expected output:
# http_req_duration..........: avg=1234ms p95=1450ms ✓ (target ≤1500ms)
```

### Measure Cache Hit Rate
```bash
# Connect to Redis
redis-cli

# Get stats
INFO stats

# Calculate hit rate:
# keyspace_hits / (keyspace_hits + keyspace_misses) > 0.95 ✓
```

### Measure Frontend Performance
```bash
# Run Lighthouse audit
npm run lighthouse

# Expected output:
# Performance: 92/100 ✓
# First Contentful Paint: 2.1s ✓ (target <3s)
```

## Common Issues & Solutions

### Issue: AI service returns 503
**Solution**: Check circuit breaker state, restart AI service
```bash
docker-compose restart ai
```

### Issue: Cache hit rate <95%
**Solution**: Check TTL strategy, verify hash computation
```bash
redis-cli
GET classification:{your_test_hash}
TTL classification:{your_test_hash}  # Should be ~86400 seconds
```

### Issue: Frontend bundle >1MB
**Solution**: Verify lazy-loading, check bundle analyzer
```bash
npm run build
npm run analyze  # Opens visualizer
```

## Team Coordination

**Daily Standups**: Share progress on implementation phases

**Integration Points**:
- Backend ↔ AI Service: Contract defined in `api-spec.yaml` (POST /classify)
- Frontend ↔ Backend: API contracts in `api-spec.yaml`

**Merge Strategy**:
1. Backend foundation merged first (杨正帅)
2. Frontend with mocked AI service merged (王喜弟)
3. AI service integration merged last (吴永杰)

## Resources

- **Specification**: `.specify/specs/fruit-vegetable-recognition/spec.md`
- **Research**: `.specify/specs/fruit-vegetable-recognition/artifacts/research.md`
- **Data Model**: `.specify/specs/fruit-vegetable-recognition/artifacts/data-model.md`
- **API Contracts**: `.specify/specs/fruit-vegetable-recognition/contracts/api-spec.yaml`
- **Constitution**: `.specify/memory/constitution.md`

## Support

- Technical questions: Open issue with label `question`
- Bug reports: Open issue with label `bug`
- Feature requests: Open issue with label `enhancement`

---

**Last Updated**: 2025-10-13
**Maintained By**: System Architect
