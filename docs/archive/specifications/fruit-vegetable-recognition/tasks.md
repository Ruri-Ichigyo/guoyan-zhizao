# Implementation Tasks: Lightweight Fruit & Vegetable Recognition System

**Feature**: Lightweight Fruit & Vegetable Recognition System
**Plan Reference**: `plan.md`
**Spec Reference**: `spec.md`
**Generated**: 2025-10-13
**Status**: Ready for Implementation

## Task Organization

Tasks are organized by user story (FR1-FR5) to enable independent implementation and testing. Each story phase includes all components needed to deliver a complete, testable increment.

**User Stories**:
- **US1 (FR1)**: Image Upload and Classification - Core workflow, highest priority
- **US2 (FR2)**: Historical Results Listing - Secondary feature
- **US3 (FR3)**: Detailed Result View - Dependent on US2
- **US4 (FR4)**: Statistics Dashboard - Analytics layer
- **US5 (FR5)**: System Health Monitoring - Operational visibility

## Task Summary

| Phase | User Story | Task Count | Parallelizable | Estimated Effort |
|-------|------------|------------|----------------|------------------|
| Phase 1 | Setup & Infrastructure | 12 | 8 tasks [P] | 2 person-days |
| Phase 2 | Foundational Prerequisites | 8 | 4 tasks [P] | 3 person-days |
| Phase 3 | US1: Upload & Classification | 35 | 18 tasks [P] | 12 person-days |
| Phase 4 | US2: Historical Results Listing | 18 | 10 tasks [P] | 4 person-days |
| Phase 5 | US3: Detailed Result View | 12 | 6 tasks [P] | 2 person-days |
| Phase 6 | US4: Statistics Dashboard | 18 | 10 tasks [P] | 5 person-days |
| Phase 7 | US5: Health Monitoring | 10 | 6 tasks [P] | 2 person-days |
| Phase 8 | Polish & Integration | 12 | 6 tasks [P] | 3 person-days |
| **Total** | **8 phases** | **125 tasks** | **68 parallel** | **33 person-days** |

---

## Phase 1: Setup & Infrastructure

**Goal**: Initialize project environment and shared tooling needed by all user stories.

**Dependencies**: None (starting point)

### Setup Tasks

**T001**: [P] Configure Rust development environment
- **File**: `backend/Cargo.toml`
- **Description**: Add dependencies: axum, sqlx, tokio, tower, redis, sha2, uuid, serde_json
- **Acceptance**: `cargo build` succeeds
- **Effort**: 30min
- **Owner**: 杨正帅

**T002**: [P] Configure Python AI service environment
- **File**: `ai/requirements.txt`
- **Description**: Add dependencies: fastapi, uvicorn, torch, onnxruntime, pillow, numpy, pydantic
- **Acceptance**: `pip install -r requirements.txt` succeeds
- **Effort**: 30min
- **Owner**: 吴永杰

**T003**: [P] Configure Frontend development environment
- **File**: `frontend/package.json`
- **Description**: Add dependencies: element-plus, pinia, axios, echarts (or chart.js), @vueuse/core
- **Acceptance**: `npm install` succeeds
- **Effort**: 30min
- **Owner**: 王喜弟

**T004**: [P] Set up Rust linting and formatting in CI
- **File**: `.github/workflows/backend-ci.yml`
- **Description**: Add jobs for `cargo clippy -- -D warnings` and `cargo fmt --check`
- **Acceptance**: CI pipeline enforces Rust code quality (Principle 1)
- **Effort**: 1h
- **Owner**: 杨正帅

**T005**: [P] Set up Python linting and type checking in CI
- **File**: `.github/workflows/ai-ci.yml`
- **Description**: Add jobs for `black .` check and `mypy ai/`
- **Acceptance**: CI pipeline enforces Python PEP8 and type annotations (Principle 1)
- **Effort**: 1h
- **Owner**: 吴永杰

**T006**: [P] Set up Frontend linting in CI
- **File**: `.github/workflows/frontend-ci.yml`
- **Description**: Add job for `npm run lint` with Composition API enforcement
- **Acceptance**: CI pipeline enforces Vue Composition API style (Principle 1)
- **Effort**: 1h
- **Owner**: 王喜弟

**T007**: [P] Configure Backend test coverage reporting
- **File**: `.github/workflows/backend-ci.yml`
- **Description**: Add `cargo tarpaulin` job, fail if coverage <85%
- **Acceptance**: Coverage reports generated, threshold enforced (Principle 2)
- **Effort**: 1h
- **Owner**: 杨正帅

**T008**: [P] Configure AI service test coverage reporting
- **File**: `.github/workflows/ai-ci.yml`
- **Description**: Add `pytest --cov` job, fail if coverage <85%
- **Acceptance**: Coverage reports generated, threshold enforced (Principle 2)
- **Effort**: 1h
- **Owner**: 吴永杰

**T009**: [P] Configure Frontend test coverage reporting
- **File**: `.github/workflows/frontend-ci.yml`
- **Description**: Add `npm run test:coverage` job (vitest), fail if coverage <85%
- **Acceptance**: Coverage reports generated, threshold enforced (Principle 2)
- **Effort**: 1h
- **Owner**: 王喜弟

**T010**: [P] Set up Cypress for E2E testing
- **File**: `frontend/cypress.config.ts`
- **Description**: Initialize Cypress, configure baseUrl to `http://localhost:3000`
- **Acceptance**: `npx cypress open` launches test runner (Principle 2)
- **Effort**: 1h
- **Owner**: 王喜弟

**T011**: Create Docker Compose override for local development
- **File**: `docker-compose.override.yml`
- **Description**: Add volume mounts for hot-reload, expose debug ports
- **Acceptance**: `./scripts/start.sh` starts services with hot-reload
- **Effort**: 1h
- **Owner**: 杨正帅

**T012**: Create centralized design tokens
- **File**: `frontend/src/styles/tokens.ts`
- **Description**: Define spacing, colors, typography constants (Principle 3)
- **Acceptance**: Tokens exported and importable in components
- **Effort**: 30min
- **Owner**: 王喜弟

**Checkpoint**: ✅ Development environment configured, CI enforces constitutional principles

---

## Phase 2: Foundational Prerequisites

**Goal**: Implement blocking infrastructure required before ANY user story can be implemented.

**Dependencies**: Phase 1 (Setup)

### Database Foundation

**T013**: Create database migration script
- **File**: `backend/migrations/001_create_uploads_and_results.sql`
- **Description**: Write SQL for `uploads` and `classification_results` tables with constraints and indexes (see data-model.md)
- **Acceptance**: Migration script contains all fields, constraints, indexes from spec
- **Effort**: 1h
- **Owner**: 杨正帅

**T014**: Run database migrations
- **Command**: `cd backend && sqlx migrate run`
- **Description**: Apply migration to local PostgreSQL
- **Acceptance**: `psql $DATABASE_URL -c "\dt"` shows both tables
- **Effort**: 15min
- **Owner**: 杨正帅

**T015**: Define Rust data models
- **File**: `backend/src/models.rs`
- **Description**: Define `Upload` and `ClassificationResult` structs with SQLx derives
- **Acceptance**: Structs compile, implement `From<PgRow>` for database mapping
- **Effort**: 2h
- **Owner**: 杨正帅

**T016**: Write unit tests for data models
- **File**: `backend/tests/unit/models_test.rs`
- **Description**: Test validation rules (file_size ≤10MB, confidence 0.0-1.0, etc.)
- **Acceptance**: Tests pass, demonstrate constraint enforcement
- **Effort**: 1h
- **Owner**: 杨正帅

### Redis Cache Foundation

**T017**: [P] Implement Redis connection pool
- **File**: `backend/src/cache.rs`
- **Description**: Initialize Redis client with connection pool (use `redis` crate)
- **Acceptance**: Pool can be cloned and shared across async tasks
- **Effort**: 1h
- **Owner**: 杨正帅

**T018**: [P] Implement cache get/set operations
- **File**: `backend/src/cache.rs`
- **Description**: Add `get_result(hash: &str)` and `set_result(hash: &str, result: &ClassificationResult, ttl: u64)`
- **Acceptance**: Functions compile, use key pattern `classification:{hash}`
- **Effort**: 2h
- **Owner**: 杨正帅

**T019**: [P] Add circuit breaker for Redis failures
- **File**: `backend/src/cache.rs`
- **Description**: Wrap Redis calls with error handling, degrade gracefully if Redis down
- **Acceptance**: System continues working when Redis unavailable (logs warning)
- **Effort**: 2h
- **Owner**: 杨正帅

**T020**: [P] Write unit tests for cache layer
- **File**: `backend/tests/unit/cache_test.rs`
- **Description**: Test hit/miss scenarios, TTL expiration, circuit breaker (mock Redis)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 杨正帅

**Checkpoint**: ✅ Database schema deployed, Redis cache operational with circuit breaker

---

## Phase 3: US1 - Image Upload and Classification (Priority 1)

**Goal**: Implement the core workflow: user uploads image → AI classification → result displayed.

**Dependencies**: Phase 2 (Foundational Prerequisites)

**User Story**: FR1 - As an end user, I can upload a produce image and receive an AI classification result within 5 seconds.

**Independent Test Criteria**:
- User can drag-drop or select an image file (≤10MB, JPEG/PNG/WebP)
- System validates file format and size, shows Toast error if invalid
- System classifies image and returns produce type + confidence (≥92% accuracy)
- Result persists to database and Redis cache (24h TTL)
- Response time ≤5 seconds P95 (AI inference ≤1500ms P95)
- Upload works without history, detail view, or statistics features

### Backend: Upload Route (US1)

**T021**: Implement multipart file upload handler
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Parse multipart request using `axum::extract::Multipart`, extract file field
- **Acceptance**: Handler receives file bytes, original filename, content type
- **Effort**: 2h
- **Owner**: 杨正帅

**T022**: Implement file validation logic
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Validate file size (≤10MB), content type (JPEG/PNG/WebP), magic bytes check
- **Acceptance**: Returns 400 Bad Request if validation fails, includes error details
- **Effort**: 2h
- **Owner**: 杨正帅

**T023**: Implement SHA-256 hash computation
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Compute SHA-256 hash of file contents using `sha2` crate
- **Acceptance**: Hash string matches expected format (64 hex characters)
- **Effort**: 1h
- **Owner**: 杨正帅

**T024**: Implement cache lookup before AI call
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Check Redis for `classification:{hash}`, return cached result if hit
- **Acceptance**: Cache hit skips AI service call (logs cache hit event)
- **Effort**: 1h
- **Owner**: 杨正帅

**T025**: Implement file storage to disk
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Save file to `/uploads/YYYY/MM/DD/{uuid}.{ext}`, create directories if needed
- **Acceptance**: File stored at correct path, UUID prevents collisions
- **Effort**: 2h
- **Owner**: 杨正帅

**T026**: Implement database insert for uploads
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Insert into `uploads` table (id, file_path, original_filename, file_size_bytes, image_hash, content_type)
- **Acceptance**: Row inserted, returns upload_id UUID
- **Effort**: 1h
- **Owner**: 杨正帅

**T027**: Write unit tests for upload validation
- **File**: `backend/tests/unit/uploads_validation_test.rs`
- **Description**: Test file size limits, content type validation, hash computation
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 杨正帅

### AI Service: Classification Endpoint (US1)

**T028**: [P] Prepare MobileNetV3 model (blocking for吴永杰)
- **File**: `ai/models/mobilenetv3.onnx`
- **Description**: Fine-tune MobileNetV3 on produce dataset (1000+ images), export to ONNX, quantize FP16
- **Acceptance**: Model achieves ≥92% accuracy on validation set (NFR2)
- **Effort**: 16h (2 days)
- **Owner**: 吴永杰

**T029**: Implement ONNX model loader
- **File**: `ai/model.py`
- **Description**: Load ONNX model using `onnxruntime.InferenceSession`, warm up on startup
- **Acceptance**: Model loads successfully, preloads weights
- **Effort**: 2h
- **Owner**: 吴永杰

**T030**: [P] Implement image preprocessing
- **File**: `ai/preprocessing.py`
- **Description**: Resize image to 224x224, normalize pixel values (mean/std for ImageNet)
- **Acceptance**: Preprocessing outputs numpy array (1, 3, 224, 224) shape
- **Effort**: 2h
- **Owner**: 吴永杰

**T031**: Implement inference predict function
- **File**: `ai/model.py`
- **Description**: Add `predict(image: np.ndarray) -> Tuple[str, float, int]` returning (produce_type, confidence, inference_time_ms)
- **Acceptance**: Returns highest confidence class, measures latency
- **Effort**: 2h
- **Owner**: 吴永杰

**T032**: [P] Create FastAPI app and classify endpoint
- **File**: `ai/main.py`
- **Description**: Create FastAPI app, add `POST /classify` endpoint accepting multipart image
- **Acceptance**: Endpoint accepts file, calls model.predict(), returns JSON
- **Effort**: 2h
- **Owner**: 吴永杰

**T033**: [P] Add Pydantic request/response models
- **File**: `ai/main.py`
- **Description**: Define type-annotated models for classify request/response (Principle 1)
- **Acceptance**: `mypy` passes, request validation enforced
- **Effort**: 1h
- **Owner**: 吴永杰

**T034**: Implement error handling in AI service
- **File**: `ai/main.py`
- **Description**: Handle invalid image format (400), inference failure (500), timeout (504)
- **Acceptance**: Returns appropriate HTTP status codes with error messages
- **Effort**: 1h
- **Owner**: 吴永杰

**T035**: Write unit tests for preprocessing
- **File**: `ai/tests/test_preprocessing.py`
- **Description**: Test image resizing, normalization, edge cases (non-square images, grayscale)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 吴永杰

**T036**: Write unit tests for inference wrapper
- **File**: `ai/tests/test_model.py`
- **Description**: Mock ONNX session, test predict function, error handling
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 吴永杰

### Backend: AI Service Integration (US1)

**T037**: Implement HTTP client for AI service
- **File**: `backend/src/ai_client.rs`
- **Description**: Create async HTTP client (using `reqwest`) to call `POST http://localhost:8002/classify`
- **Acceptance**: Sends multipart request, parses JSON response
- **Effort**: 2h
- **Owner**: 杨正帅

**T038**: Add circuit breaker for AI service calls
- **File**: `backend/src/ai_client.rs`
- **Description**: Wrap AI client with `tower` circuit breaker (5 failures → open, 60s recovery)
- **Acceptance**: Circuit opens on repeated failures, returns 503 on open state
- **Effort**: 3h
- **Owner**: 杨正帅

**T039**: Implement database insert for classification results
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Insert into `classification_results` table (id, upload_id, produce_type, confidence_score, model_version, inference_time_ms)
- **Acceptance**: Row inserted with FK constraint to uploads
- **Effort**: 1h
- **Owner**: 杨正帅

**T040**: Implement cache population after classification
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Store result in Redis with key `classification:{hash}`, TTL 86400s
- **Acceptance**: Cached result can be retrieved on subsequent uploads of same image
- **Effort**: 1h
- **Owner**: 杨正帅

**T041**: Implement upload response formatting
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Return JSON response matching spec format (success, data, error, metadata fields)
- **Acceptance**: Response includes id, produce_type, confidence, image_url, timestamp
- **Effort**: 1h
- **Owner**: 杨正帅

**T042**: Write integration test for upload flow
- **File**: `backend/tests/integration/upload_flow_test.rs`
- **Description**: Test file upload → AI call → DB insert → cache population (use test fixtures)
- **Acceptance**: Test passes end-to-end with Docker Compose services
- **Effort**: 3h
- **Owner**: 杨正帅

**T043**: Write contract test for AI service
- **File**: `backend/tests/integration/ai_service_contract_test.rs`
- **Description**: Mock AI service responses, test request/response schema validation
- **Acceptance**: Test verifies contract adherence (produce_type, confidence fields present)
- **Effort**: 2h
- **Owner**: 杨正帅

### Frontend: Upload Component (US1)

**T044**: [P] Create useToast composable
- **File**: `frontend/src/composables/useToast.ts`
- **Description**: Wrap Element Plus Message API, provide error(), success(), info() methods (Principle 3)
- **Acceptance**: Composable can be imported and used in components
- **Effort**: 1h
- **Owner**: 王喜弟

**T045**: [P] Create UploadPanel component
- **File**: `frontend/src/components/UploadPanel.vue`
- **Description**: Use `<script setup>`, Element Plus `<el-upload>` with drag-and-drop (Principle 1, 3)
- **Acceptance**: Component renders upload zone with drag-drop UI
- **Effort**: 2h
- **Owner**: 王喜弟

**T046**: Implement client-side file validation
- **File**: `frontend/src/components/UploadPanel.vue`
- **Description**: Validate file size ≤10MB client-side, show Toast error if invalid
- **Acceptance**: Toast error appears before upload for oversized files (Principle 3)
- **Effort**: 1h
- **Owner**: 王喜弟

**T047**: Implement upload progress bar
- **File**: `frontend/src/components/UploadPanel.vue`
- **Description**: Show Element Plus progress bar during upload (Principle 3)
- **Acceptance**: Progress indicator visible during multipart upload
- **Effort**: 1h
- **Owner**: 王喜弟

**T048**: Implement API call to backend uploads endpoint
- **File**: `frontend/src/components/UploadPanel.vue`
- **Description**: Use Axios to POST multipart to `/api/v1/uploads`
- **Acceptance**: Request includes file in FormData, handles response/error
- **Effort**: 2h
- **Owner**: 王喜弟

**T049**: Implement result display in UploadPanel
- **File**: `frontend/src/components/UploadPanel.vue`
- **Description**: Show produce type + confidence percentage after successful upload
- **Acceptance**: Result displays with formatting (e.g., "Apple - 96% confidence")
- **Effort**: 1h
- **Owner**: 王喜弟

**T050**: Implement error handling with Toast notifications
- **File**: `frontend/src/components/UploadPanel.vue`
- **Description**: Catch Axios errors, display user-friendly Toast messages (Principle 3)
- **Acceptance**: Upload failures show Toast errors (file too large, unsupported format, AI service down)
- **Effort**: 1h
- **Owner**: 王喜弟

**T051**: Write unit tests for useToast composable
- **File**: `frontend/tests/unit/useToast.spec.ts`
- **Description**: Mock Element Plus Message, test error/success/info methods
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 1h
- **Owner**: 王喜弟

**T052**: Write unit tests for UploadPanel component
- **File**: `frontend/tests/unit/UploadPanel.spec.ts`
- **Description**: Test file validation logic, upload state transitions, error handling (mock Axios)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 王喜弟

**T053**: Write Cypress E2E test for upload workflow
- **File**: `frontend/cypress/e2e/upload.cy.ts`
- **Description**: Test drag-drop image → see result → verify success (Principle 2)
- **Acceptance**: E2E test passes with real backend/AI service
- **Effort**: 2h
- **Owner**: 王喜弟

**T054**: Write Cypress E2E test for upload error handling
- **File**: `frontend/cypress/e2e/upload-errors.cy.ts`
- **Description**: Test upload 11MB file → see Toast error, test unsupported format → see Toast error
- **Acceptance**: E2E tests verify Toast notifications appear (Principle 3)
- **Effort**: 1h
- **Owner**: 王喜弟

**T055**: Measure and validate AI inference latency
- **File**: `tests/performance/inference_latency.js` (k6 script)
- **Description**: Load test with 50+ concurrent uploads, measure P95 latency
- **Acceptance**: P95 latency ≤1500ms (Principle 4, NFR3)
- **Effort**: 2h
- **Owner**: 吴永杰

**Checkpoint**: ✅ US1 complete - Users can upload images and receive classification results. MVP deliverable!

---

## Phase 4: US2 - Historical Results Listing (Priority 2)

**Goal**: Implement paginated history view of all past classifications.

**Dependencies**: Phase 3 (US1 - Upload must exist to have history)

**User Story**: FR2 - As an end user, I can view a paginated list of historical classification results sorted by most recent first.

**Independent Test Criteria**:
- User can navigate to /results page
- System displays grid of result cards (20 per page) with thumbnails, produce name, confidence, timestamp
- Pagination controls allow navigation between pages
- Results load within 2 seconds for 100+ records (NFR3)
- History works independently (no dependency on statistics or detail view for basic listing)

### Backend: Results Listing Route (US2)

**T056**: Implement GET /api/v1/results endpoint
- **File**: `backend/src/routes/results.rs`
- **Description**: Add paginated query handler accepting page, limit query params
- **Acceptance**: Returns JSON array of results with pagination metadata
- **Effort**: 2h
- **Owner**: 杨正帅

**T057**: Implement database query with pagination
- **File**: `backend/src/routes/results.rs`
- **Description**: Query `uploads` JOIN `classification_results` ORDER BY upload_timestamp DESC LIMIT/OFFSET
- **Acceptance**: Query uses index `idx_uploads_timestamp`, returns correct page
- **Effort**: 2h
- **Owner**: 杨正帅

**T058**: Implement pagination metadata calculation
- **File**: `backend/src/routes/results.rs`
- **Description**: Calculate total pages, validate page bounds
- **Acceptance**: Response includes {total, page, limit, total_pages}
- **Effort**: 1h
- **Owner**: 杨正帅

**T059**: Write unit tests for results route
- **File**: `backend/tests/unit/results_route_test.rs`
- **Description**: Test pagination logic, query parameter validation (mock DB)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 杨正帅

**T060**: Write integration test for results pagination
- **File**: `backend/tests/integration/results_pagination_test.rs`
- **Description**: Insert 50 test records, query multiple pages, verify order
- **Acceptance**: Test confirms correct pagination, sorting by timestamp DESC
- **Effort**: 2h
- **Owner**: 杨正帅

### Frontend: Results History Component (US2)

**T061**: [P] Create Pinia results store
- **File**: `frontend/src/stores/results.ts`
- **Description**: Define state (results array, pagination), actions (fetchResults(page))
- **Acceptance**: Store compiles, can be used in components
- **Effort**: 1h
- **Owner**: 王喜弟

**T062**: [P] Create ResultsHistory component
- **File**: `frontend/src/components/ResultsHistory.vue`
- **Description**: Use `<script setup>`, fetch results from Pinia store on mount
- **Acceptance**: Component renders loading state initially
- **Effort**: 2h
- **Owner**: 王喜弟

**T063**: Implement responsive grid layout
- **File**: `frontend/src/components/ResultsHistory.vue`
- **Description**: Use Element Plus `<el-row>` `<el-col>` with responsive breakpoints (Principle 3)
- **Acceptance**: Grid adapts to mobile (1 col), tablet (2 cols), desktop (4 cols)
- **Effort**: 2h
- **Owner**: 王喜弟

**T064**: Implement result card display
- **File**: `frontend/src/components/ResultsHistory.vue`
- **Description**: Render each result as card showing thumbnail, produce name, confidence, timestamp
- **Acceptance**: Cards use consistent spacing from design tokens (Principle 3)
- **Effort**: 2h
- **Owner**: 王喜弟

**T065**: Implement pagination controls
- **File**: `frontend/src/components/ResultsHistory.vue`
- **Description**: Add Element Plus `<el-pagination>`, bind to page state
- **Acceptance**: Clicking page number triggers fetchResults(page)
- **Effort**: 1h
- **Owner**: 王喜弟

**T066**: Implement loading and empty states
- **File**: `frontend/src/components/ResultsHistory.vue`
- **Description**: Show loading skeleton while fetching, empty state if no results
- **Acceptance**: Loading state visible during fetch, empty state shows helpful message
- **Effort**: 1h
- **Owner**: 王喜弟

**T067**: Write unit tests for results store
- **File**: `frontend/tests/unit/results.store.spec.ts`
- **Description**: Test fetchResults action, pagination state updates (mock Axios)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 王喜弟

**T068**: Write unit tests for ResultsHistory component
- **File**: `frontend/tests/unit/ResultsHistory.spec.ts`
- **Description**: Test grid rendering, pagination controls, loading/empty states
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 王喜弟

**T069**: Write Cypress E2E test for history navigation
- **File**: `frontend/cypress/e2e/history.cy.ts`
- **Description**: Upload 25 images, navigate to history, paginate → verify 2 pages
- **Acceptance**: E2E test verifies pagination works correctly
- **Effort**: 2h
- **Owner**: 王喜弟

**T070**: Add route for history page
- **File**: `frontend/src/router/index.ts`
- **Description**: Add `/results` route pointing to ResultsHistory component
- **Acceptance**: Navigating to /results renders history page
- **Effort**: 30min
- **Owner**: 王喜弟

**T071**: Measure and validate history page load time
- **File**: `tests/performance/history_load.js` (k6 script)
- **Description**: Seed 200 results, measure time to render first page
- **Acceptance**: Page loads within 2 seconds with 100+ records (NFR3)
- **Effort**: 1h
- **Owner**: 杨正帅

**T072**: Add navigation link from UploadPanel to History
- **File**: `frontend/src/components/UploadPanel.vue`
- **Description**: After successful upload, show "View History" button → navigate to /results
- **Acceptance**: Button visible after upload, navigates correctly
- **Effort**: 30min
- **Owner**: 王喜弟

**T073**: Write integration test for upload → history flow
- **File**: `frontend/cypress/e2e/upload-to-history.cy.ts`
- **Description**: Upload image → click "View History" → verify uploaded image appears in list
- **Acceptance**: E2E test confirms upload persists and displays in history
- **Effort**: 1h
- **Owner**: 王喜弟

**Checkpoint**: ✅ US2 complete - Users can view paginated history of all classifications

---

## Phase 5: US3 - Detailed Result View (Priority 3)

**Goal**: Implement detailed view for individual classification results.

**Dependencies**: Phase 4 (US2 - History must exist to navigate to details)

**User Story**: FR3 - As an end user, I can click on a result card to view full details including full-size image and metadata.

**Independent Test Criteria**:
- User can click result card in history → navigate to /results/{id}
- Detail page displays full-size image, produce type, confidence, timestamp, model version, inference time
- Back button returns to history
- Detail view loads quickly (<1 second for single record)

### Backend: Result Detail Route (US3)

**T074**: Implement GET /api/v1/results/{id} endpoint
- **File**: `backend/src/routes/results.rs`
- **Description**: Add detail query handler accepting UUID path param
- **Acceptance**: Returns single result JSON with full details
- **Effort**: 1h
- **Owner**: 杨正帅

**T075**: Implement database query for single result
- **File**: `backend/src/routes/results.rs`
- **Description**: Query uploads JOIN classification_results WHERE id = $1
- **Acceptance**: Returns 404 if not found, 200 with result if found
- **Effort**: 1h
- **Owner**: 杨正帅

**T076**: Write unit tests for detail route
- **File**: `backend/tests/unit/results_detail_test.rs`
- **Description**: Test valid ID → returns result, invalid ID → 404
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 1h
- **Owner**: 杨正帅

**T077**: Write integration test for result detail
- **File**: `backend/tests/integration/results_detail_test.rs`
- **Description**: Insert test record, query by ID, verify all fields present
- **Acceptance**: Test confirms metadata field (JSONB) correctly serialized
- **Effort**: 1h
- **Owner**: 杨正帅

### Frontend: Result Detail Component (US3)

**T078**: [P] Create ResultDetail component
- **File**: `frontend/src/components/ResultDetail.vue`
- **Description**: Use `<script setup>`, fetch result by ID from route params
- **Acceptance**: Component loads result data on mount
- **Effort**: 2h
- **Owner**: 王喜弟

**T079**: [P] Implement full-size image display
- **File**: `frontend/src/components/ResultDetail.vue`
- **Description**: Display image at original size (or max viewport size), responsive
- **Acceptance**: Image displays correctly, lazy-loads if large
- **Effort**: 1h
- **Owner**: 王喜弟

**T080**: Implement result metadata display
- **File**: `frontend/src/components/ResultDetail.vue`
- **Description**: Show produce type, confidence, timestamp, model version, inference time
- **Acceptance**: All fields formatted correctly (timestamps as readable dates)
- **Effort**: 1h
- **Owner**: 王喜弟

**T081**: Implement back button navigation
- **File**: `frontend/src/components/ResultDetail.vue`
- **Description**: Add back button using `router.push('/results')` or `router.back()`
- **Acceptance**: Clicking back returns to history page
- **Effort**: 30min
- **Owner**: 王喜弟

**T082**: Add route for detail page
- **File**: `frontend/src/router/index.ts`
- **Description**: Add `/results/:id` route pointing to ResultDetail component
- **Acceptance**: Navigating to /results/{uuid} renders detail page
- **Effort**: 30min
- **Owner**: 王喜弟

**T083**: Add click handler to history cards
- **File**: `frontend/src/components/ResultsHistory.vue`
- **Description**: Make result cards clickable, navigate to `/results/{id}` on click
- **Acceptance**: Clicking card navigates to detail view
- **Effort**: 30min
- **Owner**: 王喜弟

**T084**: Write unit tests for ResultDetail component
- **File**: `frontend/tests/unit/ResultDetail.spec.ts`
- **Description**: Test data fetching, metadata display, back navigation (mock Axios, router)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 王喜弟

**T085**: Write Cypress E2E test for detail view flow
- **File**: `frontend/cypress/e2e/result-detail.cy.ts`
- **Description**: Navigate to history → click card → verify detail page → click back
- **Acceptance**: E2E test verifies full navigation flow
- **Effort**: 1h
- **Owner**: 王喜弟

**Checkpoint**: ✅ US3 complete - Users can view detailed information for individual results

---

## Phase 6: US4 - Statistics Dashboard (Priority 4)

**Goal**: Implement analytics dashboard with charts visualizing usage patterns.

**Dependencies**: Phase 3 (US1 - Statistics aggregate upload data)

**User Story**: FR4 - As a system operator or business analyst, I can view aggregate statistics including top produce types, upload trends, and average confidence.

**Independent Test Criteria**:
- User can navigate to /statistics page
- Dashboard displays total uploads, top 5 produce types (pie chart), upload trends (line chart), average confidence
- Dashboard loads within 3 seconds (NFR3)
- Charts render correctly with real data
- Dashboard works independently (no dependency on history or detail views)

### Backend: Statistics Route (US4)

**T086**: Implement GET /api/v1/statistics endpoint
- **File**: `backend/src/routes/statistics.rs`
- **Description**: Add statistics aggregation handler
- **Acceptance**: Returns JSON with total_uploads, top_produce, average_confidence, upload_trends
- **Effort**: 3h
- **Owner**: 杨正帅

**T087**: Implement top produce query
- **File**: `backend/src/routes/statistics.rs`
- **Description**: Query `GROUP BY produce_type ORDER BY count DESC LIMIT 5`, calculate percentages
- **Acceptance**: Returns array of {name, count, percentage}
- **Effort**: 2h
- **Owner**: 杨正帅

**T088**: Implement upload trends query
- **File**: `backend/src/routes/statistics.rs`
- **Description**: Query `GROUP BY DATE(upload_timestamp)` for last 30 days
- **Acceptance**: Returns array of {date, count} sorted by date DESC
- **Effort**: 2h
- **Owner**: 杨正帅

**T089**: Implement statistics caching
- **File**: `backend/src/routes/statistics.rs`
- **Description**: Cache statistics in Redis for 5 minutes (reduce DB load)
- **Acceptance**: Subsequent requests within 5 min return cached data
- **Effort**: 1h
- **Owner**: 杨正帅

**T090**: Write unit tests for statistics route
- **File**: `backend/tests/unit/statistics_route_test.rs`
- **Description**: Test aggregation logic, caching behavior (mock DB, Redis)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 杨正帅

**T091**: Write integration test for statistics aggregation
- **File**: `backend/tests/integration/statistics_test.rs`
- **Description**: Insert 100 varied test records, query statistics, verify calculations
- **Acceptance**: Test confirms correct aggregation and percentage calculations
- **Effort**: 2h
- **Owner**: 杨正帅

### Frontend: Statistics Dashboard Component (US4)

**T092**: [P] Install chart library (ECharts or Chart.js)
- **File**: `frontend/package.json`
- **Description**: Add echarts dependency, configure for lazy-loading (Principle 4)
- **Acceptance**: `npm install` succeeds, library can be imported
- **Effort**: 30min
- **Owner**: 王喜弟

**T093**: [P] Create StatisticsDashboard component (lazy-loaded)
- **File**: `frontend/src/components/StatisticsDashboard.vue`
- **Description**: Use `<script setup>`, lazy-load via router (code splitting for bundle optimization)
- **Acceptance**: Component loads asynchronously, reduces main bundle size (Principle 4)
- **Effort**: 2h
- **Owner**: 王喜弟

**T094**: Implement API call to statistics endpoint
- **File**: `frontend/src/components/StatisticsDashboard.vue`
- **Description**: Fetch `/api/v1/statistics` on mount, store in reactive state
- **Acceptance**: Data fetched and stored in component state
- **Effort**: 1h
- **Owner**: 王喜弟

**T095**: Implement key metrics display
- **File**: `frontend/src/components/StatisticsDashboard.vue`
- **Description**: Display total uploads, average confidence as large numbers with labels
- **Acceptance**: Metrics displayed prominently using design tokens
- **Effort**: 1h
- **Owner**: 王喜弟

**T096**: Implement pie chart for produce distribution
- **File**: `frontend/src/components/StatisticsDashboard.vue`
- **Description**: Render pie chart using ECharts, show top 5 produce types with percentages
- **Acceptance**: Chart renders correctly with data from API
- **Effort**: 2h
- **Owner**: 王喜弟

**T097**: Implement line chart for upload trends
- **File**: `frontend/src/components/StatisticsDashboard.vue`
- **Description**: Render line chart using ECharts, show upload count over last 30 days
- **Acceptance**: Chart renders with time series data, X-axis shows dates
- **Effort**: 2h
- **Owner**: 王喜弟

**T098**: Implement auto-refresh mechanism
- **File**: `frontend/src/components/StatisticsDashboard.vue`
- **Description**: Use setInterval to refetch statistics every 60 seconds, clear on unmount
- **Acceptance**: Dashboard updates automatically without user interaction
- **Effort**: 1h
- **Owner**: 王喜弟

**T099**: Add route for statistics page (lazy-loaded)
- **File**: `frontend/src/router/index.ts`
- **Description**: Add `/statistics` route with lazy import `() => import('./components/StatisticsDashboard.vue')`
- **Acceptance**: Route lazy-loads component (verified in bundle analyzer)
- **Effort**: 30min
- **Owner**: 王喜弟

**T100**: Write unit tests for StatisticsDashboard
- **File**: `frontend/tests/unit/StatisticsDashboard.spec.ts`
- **Description**: Test data fetching, chart rendering, auto-refresh (mock Axios, ECharts)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 王喜弟

**T101**: Write Cypress E2E test for statistics rendering
- **File**: `frontend/cypress/e2e/statistics.cy.ts`
- **Description**: Seed varied data, navigate to /statistics, verify charts display
- **Acceptance**: E2E test confirms charts render with correct data
- **Effort**: 2h
- **Owner**: 王喜弟

**T102**: Measure and validate dashboard load time
- **File**: `tests/performance/statistics_load.js`
- **Description**: Measure time from navigation to chart render complete
- **Acceptance**: Dashboard loads within 3 seconds (Principle 4, NFR3)
- **Effort**: 1h
- **Owner**: 王喜弟

**T103**: Run bundle analyzer and verify lazy-loading
- **Command**: `npm run build && npm run analyze`
- **Description**: Verify StatisticsDashboard and ECharts are in separate chunk
- **Acceptance**: Main bundle <500KB, statistics chunk lazy-loaded (Principle 4)
- **Effort**: 30min
- **Owner**: 王喜弟

**Checkpoint**: ✅ US4 complete - Users can view analytics dashboard with usage insights

---

## Phase 7: US5 - System Health Monitoring (Priority 5)

**Goal**: Implement health check endpoint for operational monitoring.

**Dependencies**: Phase 2 (Foundational - Health check tests all infrastructure)

**User Story**: FR5 - As a DevOps engineer, I can query the /health endpoint to monitor system component status.

**Independent Test Criteria**:
- GET /health returns overall status (healthy/degraded/unhealthy)
- Response includes component-level status: PostgreSQL, Redis, AI service
- Response includes response times for each component
- Health check responds within 30 seconds (NFR Success Criteria)
- Health check works independently (no dependency on user-facing features)

### Backend: Health Check Route (US5)

**T104**: Implement GET /health endpoint
- **File**: `backend/src/routes/mod.rs` or `backend/src/routes/health.rs`
- **Description**: Add health check handler that tests all components
- **Acceptance**: Returns JSON with status and components fields
- **Effort**: 2h
- **Owner**: 杨正帅

**T105**: Implement PostgreSQL connectivity check
- **File**: `backend/src/routes/health.rs`
- **Description**: Execute simple query (`SELECT 1`), measure response time
- **Acceptance**: Returns {status: "up"/"down", response_time_ms}
- **Effort**: 1h
- **Owner**: 杨正帅

**T106**: [P] Implement Redis connectivity check
- **File**: `backend/src/routes/health.rs`
- **Description**: Execute PING command, measure response time
- **Acceptance**: Returns {status: "up"/"down", response_time_ms}
- **Effort**: 1h
- **Owner**: 杨正帅

**T107**: [P] Implement AI service connectivity check
- **File**: `backend/src/routes/health.rs`
- **Description**: Call AI service health endpoint or timeout after 5s
- **Acceptance**: Returns {status: "up"/"down", response_time_ms, error if down}
- **Effort**: 1h
- **Owner**: 杨正帅

**T108**: Implement overall status calculation
- **File**: `backend/src/routes/health.rs`
- **Description**: Determine overall status: healthy (all up), degraded (partial), unhealthy (critical down)
- **Acceptance**: Status reflects worst component state
- **Effort**: 1h
- **Owner**: 杨正帅

**T109**: Add /health endpoint to AI service
- **File**: `ai/main.py`
- **Description**: Add simple `GET /health` returning {"status": "healthy"}
- **Acceptance**: Endpoint returns 200 OK when service running
- **Effort**: 30min
- **Owner**: 吴永杰

**T110**: Write unit tests for health check logic
- **File**: `backend/tests/unit/health_check_test.rs`
- **Description**: Test status calculation, component checks (mock DB, Redis, HTTP)
- **Acceptance**: Tests pass, coverage ≥85%
- **Effort**: 2h
- **Owner**: 杨正帅

**T111**: Write integration test for health endpoint
- **File**: `backend/tests/integration/health_check_test.rs`
- **Description**: Start all services, query /health, verify all components "up"
- **Acceptance**: Test confirms healthy state with Docker Compose
- **Effort**: 1h
- **Owner**: 杨正帅

**T112**: Write integration test for degraded state
- **File**: `backend/tests/integration/health_degraded_test.rs`
- **Description**: Stop AI service, query /health, verify "degraded" status
- **Acceptance**: Test confirms partial failure detection
- **Effort**: 1h
- **Owner**: 杨正帅

**T113**: Document health check endpoint in API docs
- **File**: `docs/API.md`
- **Description**: Add health check section with example responses
- **Acceptance**: Documentation complete (contributes to NFR6: 100% API docs)
- **Effort**: 30min
- **Owner**: 杨正帅

**Checkpoint**: ✅ US5 complete - Health monitoring operational for all system components

---

## Phase 8: Polish & Cross-Cutting Concerns

**Goal**: Finalize cross-cutting features, performance optimization, and comprehensive testing.

**Dependencies**: All user story phases (Phase 3-7)

### Performance Optimization

**T114**: [P] Implement frontend bundle optimization
- **File**: `frontend/vite.config.ts`
- **Description**: Configure code splitting, tree shaking, WebP image support
- **Acceptance**: Main bundle <500KB, Lighthouse performance ≥90 (Principle 4)
- **Effort**: 2h
- **Owner**: 王喜弟

**T115**: [P] Implement backend thumbnail generation
- **File**: `backend/src/routes/uploads.rs`
- **Description**: Generate 300x300 WebP thumbnails on upload, save to `thumb_{uuid}.webp`
- **Acceptance**: Thumbnails served for history grid (reduces payload size)
- **Effort**: 3h
- **Owner**: 杨正帅

**T116**: [P] Validate cache hit rate in production simulation
- **Command**: Redis `INFO stats`
- **Description**: Run load test with repeated images, measure cache hit rate
- **Acceptance**: Cache hit rate >95% (Principle 4, NFR4)
- **Effort**: 2h
- **Owner**: 杨正帅

### Comprehensive Testing

**T117**: Run full coverage validation across all modules
- **Commands**: `cargo tarpaulin`, `pytest --cov`, `npm run test:coverage`
- **Description**: Verify all modules achieve ≥85% coverage
- **Acceptance**: All coverage reports ≥85% (Principle 2)
- **Effort**: 2h
- **Owner**: All

**T118**: Run load test for concurrency validation
- **File**: `tests/performance/concurrency_test.js` (k6)
- **Description**: Simulate 50+ concurrent uploads, measure latency increase
- **Acceptance**: Latency increase <10% (NFR1: concurrency requirement)
- **Effort**: 2h
- **Owner**: 杨正帅

**T119**: Run Lighthouse audit on all pages
- **Command**: `npx lighthouse http://localhost:3000 --preset=desktop`
- **Description**: Audit upload, history, detail, statistics pages
- **Acceptance**: All pages FCP <3s, Performance ≥90 (Principle 4)
- **Effort**: 1h
- **Owner**: 王喜弟

**T120**: Validate model accuracy on test dataset
- **File**: `ai/tests/accuracy_validation.py`
- **Description**: Run model on 1000+ image validation set, calculate accuracy
- **Acceptance**: Accuracy ≥92% (NFR2: model accuracy requirement)
- **Effort**: 2h
- **Owner**: 吴永杰

### Documentation & Deployment

**T121**: [P] Update API documentation from OpenAPI spec
- **File**: `docs/API.md`
- **Description**: Generate docs from `contracts/api-spec.yaml`, add examples
- **Acceptance**: All 5 endpoints documented (NFR6: 100% API documentation)
- **Effort**: 2h
- **Owner**: 杨正帅

**T122**: [P] Update architecture diagrams
- **File**: `backend/uml-output/*.png`
- **Description**: Regenerate UML diagrams reflecting implemented architecture
- **Acceptance**: Diagrams match final implementation
- **Effort**: 1h
- **Owner**: 杨正帅

**T123**: [P] Document deployment process
- **File**: `docs/DEPLOY.md`
- **Description**: Add production deployment steps, environment variables, monitoring setup
- **Acceptance**: Deployment guide complete and tested
- **Effort**: 2h
- **Owner**: 杨正帅

**T124**: Create production Docker Compose config
- **File**: `docker-compose.prod.yml`
- **Description**: Production-optimized config with resource limits, health checks
- **Acceptance**: Config starts all services in production mode
- **Effort**: 2h
- **Owner**: 杨正帅

**T125**: Set up Prometheus metrics collection
- **Files**: `backend/src/metrics.rs`, `prometheus.yml`
- **Description**: Instrument code with metrics (upload_requests_total, inference_latency_seconds, cache_hit_rate)
- **Acceptance**: Metrics exposed at /metrics endpoint, Prometheus scrapes successfully
- **Effort**: 3h
- **Owner**: 杨正帅

**Checkpoint**: ✅ Phase 8 complete - System polished, fully tested, production-ready

---

## Dependency Graph

```
Phase 1 (Setup) ──┐
                  ├──> Phase 2 (Foundational) ──┐
                  │                              ├──> Phase 3 (US1: Upload) ──┬──> Phase 8 (Polish)
                  │                              │                            │
                  │                              └──> Phase 7 (US5: Health) ──┘
                  │
                  └──> Phase 3 (US1: Upload) ──┬──> Phase 4 (US2: History) ──> Phase 5 (US3: Detail) ──> Phase 8
                                                 │
                                                 └──> Phase 6 (US4: Statistics) ──────────────────────> Phase 8
```

**Critical Path**: Phase 1 → Phase 2 → Phase 3 (US1) → Phase 8 (minimum viable product)

**Parallelizable Phases**:
- Phase 4 (US2), Phase 6 (US4), Phase 7 (US5) can start after Phase 3
- Phase 5 (US3) depends on Phase 4

---

## Parallel Execution Examples

### Within Phase 3 (US1 - Upload):

**Parallel Group 1** (After T013-T020 complete):
- T021-T027 (Backend upload route) + T028-T036 (AI service) + T044-T050 (Frontend upload component)
- **Team**: 杨正帅 (backend), 吴永杰 (AI), 王喜弟 (frontend) work in parallel

**Parallel Group 2** (After Parallel Group 1):
- T037-T043 (Backend AI integration) + T051-T054 (Frontend tests)
- **Team**: 杨正帅 + 王喜弟 work in parallel

### Within Phase 4 (US2 - History):

**Parallel Group**:
- T056-T060 (Backend results route) + T061-T070 (Frontend history component)
- **Team**: 杨正帅 + 王喜弟 work in parallel

### Within Phase 6 (US4 - Statistics):

**Parallel Group**:
- T086-T091 (Backend statistics route) + T092-T101 (Frontend dashboard)
- **Team**: 杨正帅 + 王喜弟 work in parallel

---

## Implementation Strategy

### MVP Scope (Minimum Viable Product)

**Recommendation**: Deliver Phase 1 + Phase 2 + Phase 3 (US1) first.

**MVP Deliverable**:
- ✅ User can upload image and receive AI classification
- ✅ Results persist to database and cache
- ✅ All tests pass (≥85% coverage)
- ✅ Performance meets constitutional budgets

**Why this is an MVP**: Core value proposition (upload → classify) works end-to-end. History, details, and statistics are enhancements.

### Incremental Delivery Plan

**Iteration 1** (Week 1-2):
- Phase 1 (Setup)
- Phase 2 (Foundational)
- Phase 3 (US1: Upload & Classification)
- **Deliverable**: MVP with upload workflow

**Iteration 2** (Week 3):
- Phase 4 (US2: History)
- Phase 5 (US3: Detail)
- **Deliverable**: MVP + historical review capability

**Iteration 3** (Week 4):
- Phase 6 (US4: Statistics)
- Phase 7 (US5: Health Monitoring)
- **Deliverable**: Full feature set with analytics and ops visibility

**Iteration 4** (Week 5):
- Phase 8 (Polish & Integration)
- **Deliverable**: Production-ready system

### Testing Strategy

**TDD Approach** (Tests before implementation):
- Write unit tests (T016, T020, T027, etc.) before implementing features
- Write integration tests (T042, T043, T060, etc.) to define contracts
- Write E2E tests (T053, T054, T069, etc.) to validate user workflows

**Continuous Validation**:
- Run `cargo test`, `pytest`, `npm test` after each task
- Check coverage reports (must stay ≥85%)
- Run E2E tests in CI on every PR

---

## Task Execution Checklist

For each task, verify:

- [ ] File created/modified at specified path
- [ ] Acceptance criteria met
- [ ] Unit tests written and passing (if applicable)
- [ ] Code quality checks pass (clippy/black/eslint)
- [ ] Coverage remains ≥85%
- [ ] Constitutional principles satisfied
- [ ] Task marked complete in tracking system

---

## Success Metrics (Post-Implementation)

Track these metrics after all phases complete:

- [ ] **AI Inference P95 ≤1500ms** (Principle 4, NFR3)
- [ ] **Redis Cache Hit Rate >95%** (Principle 4, NFR4)
- [ ] **Frontend FCP <3s** (Principle 4, Lighthouse)
- [ ] **Unit Test Coverage ≥85%** (Principle 2)
- [ ] **Model Accuracy ≥92%** (NFR2)
- [ ] **50+ Concurrent Users Supported** (NFR1)
- [ ] **API Documentation 100%** (NFR6)
- [ ] **Zero Critical Bugs in 7 Days** (Success Criteria)

---

**Total Tasks**: 125
**Parallelizable Tasks**: 68 (54%)
**Estimated Total Effort**: 33 person-days
**Team Size**: 3 (杨正帅, 吴永杰, 王喜弟)
**Estimated Calendar Time**: 5 weeks (with parallel execution)

---

**Next Steps**: Start with Phase 1 (Setup & Infrastructure). Execute tasks T001-T012 in parallel where marked [P].

**Suggested First Sprint**: Phase 1 + Phase 2 + first half of Phase 3 (up to T037) = MVP core logic ready for integration testing.
