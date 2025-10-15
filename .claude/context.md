# Project Context

This file provides context for AI-assisted development.

## Technology Stack

<!-- TECH_STACK_BEGIN -->
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
<!-- TECH_STACK_END -->

## Manual Notes

Add custom project notes here. They will be preserved during updates.

