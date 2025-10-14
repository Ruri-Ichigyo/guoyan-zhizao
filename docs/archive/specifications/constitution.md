<!--
Sync Impact Report
==================
Version: none → 1.0.0 (Initial constitution)
Rationale: First version establishing core principles for the fruit/vegetable detection platform

Added Sections:
- Principle 1: Code Quality Standards
- Principle 2: Testing Standards
- Principle 3: User Experience Consistency
- Principle 4: Performance Requirements

Removed Sections: N/A (initial version)

Templates Status:
✅ .specify/templates/plan-template.md - Created with constitution compliance checks
✅ .specify/templates/spec-template.md - Created with principle alignment checklist
✅ .specify/templates/tasks-template.md - Created with principle-categorized tasks

Follow-up TODOs:
- Establish baseline metrics collection for performance requirements (AI latency, cache hit rate, FCP)
- Configure CI pipelines to enforce code quality and testing standards
- Set up performance monitoring (APM, Lighthouse CI)
-->

# Project Constitution: 果蔬智造核心平台 (Fruit & Vegetable AI Detection Platform)

**Version**: 1.0.0
**Ratification Date**: 2025-10-13
**Last Amended**: 2025-10-13

## Preamble

This constitution establishes the foundational principles and governance framework for the 果蔬智造核心平台 (guoyan-zhizao), a lightweight image recognition system focused on the workflow: image upload → AI analysis → results query. The platform comprises a Rust backend, FastAPI AI service, Redis caching layer, PostgreSQL persistence, and Vue 3 frontend.

All contributions, features, and technical decisions MUST align with the principles defined herein. Deviation requires explicit justification and amendment procedure compliance.

## Core Principles

### Principle 1: Code Quality Standards

**Statement**: All code MUST adhere to language-specific quality standards that ensure maintainability, readability, and consistency across the multi-language codebase.

**Requirements**:
- **Rust Backend**: MUST pass `cargo clippy` without warnings and MUST be formatted with `cargo fmt`. CI pipelines MUST enforce these checks before merge.
- **Python AI Service**: MUST follow PEP 8 style guidelines and MUST include type annotations for all function signatures. Tools like `mypy` or `pyright` MUST validate type correctness.
- **Frontend Vue 3**: MUST use Composition API style consistently across all components. Script setup syntax (`<script setup>`) is the preferred approach for new components.

**Rationale**: Multi-language projects require strict conventions to prevent drift. Automated enforcement reduces review burden and ensures consistent code style regardless of author.

**Enforcement**: Pre-commit hooks and CI checks gate all merges. Existing code that fails these standards MUST be brought into compliance during refactoring or feature work in affected modules.

### Principle 2: Testing Standards

**Statement**: All critical paths MUST be covered by automated tests to prevent regressions and ensure system reliability.

**Requirements**:
- **Unit Test Coverage**: Core business logic MUST achieve ≥85% line coverage. Backend routes, AI inference logic, and frontend utility functions are considered core paths.
- **Integration Tests**: Service-to-service communication (backend ↔ AI service, backend ↔ Redis, backend ↔ PostgreSQL) MUST have integration tests validating contract adherence and error handling.
- **End-to-End Tests**: Frontend critical workflows (image upload, result list navigation, detection detail view) MUST be covered by Cypress or equivalent E2E framework.

**Rationale**: The system's value depends on reliable AI inference delivery. Testing prevents silent failures in the upload → analysis → query pipeline and enables confident refactoring.

**Enforcement**: Pull requests that reduce coverage below thresholds MUST be rejected unless accompanied by a documented exception approved by project maintainers. Coverage reports MUST be generated in CI and reviewed during code review.

### Principle 3: User Experience Consistency

**Statement**: User interactions MUST be predictable and uniform across all platform touchpoints to minimize cognitive load and training time.

**Requirements**:
- **Upload Interactions**: All image upload entry points MUST support drag-and-drop in addition to file picker dialogs. Visual feedback (hover states, progress indicators) MUST be consistent.
- **Result Display**: Detection results MUST use a responsive grid layout that adapts to viewport sizes. Card-based presentation with consistent spacing and typography is mandatory.
- **Error Handling**: All user-facing errors (network failures, invalid uploads, inference errors) MUST be communicated via Toast notifications with actionable messages. Console-only errors are prohibited in production builds.

**Rationale**: A small-scope platform gains user adoption through polish. Consistency reduces support burden and accelerates user onboarding.

**Enforcement**: UX review MUST occur for all frontend PRs. Design tokens (spacing, colors, typography) MUST be centralized in a shared configuration file. Deviations require design review approval.

### Principle 4: Performance Requirements

**Statement**: System responsiveness directly impacts user satisfaction and operational costs. Performance budgets are enforceable contracts.

**Requirements**:
- **AI Inference Latency**: P95 latency for AI inference requests MUST remain ≤1500ms measured from backend request initiation to response delivery. This includes network overhead to the AI service.
- **Redis Cache Hit Rate**: Detection result cache MUST maintain >95% hit rate for repeated queries. Cache invalidation strategies MUST be documented and tested.
- **Frontend Load Time**: Initial page load (First Contentful Paint) MUST complete within 3 seconds on a simulated 3G connection as measured by Lighthouse performance audits.

**Rationale**: The platform's "lightweight" positioning requires disciplined performance management. Slow inference degrades UX; poor caching increases infrastructure costs; slow frontends drive user abandonment.

**Enforcement**: Performance metrics MUST be collected in production (APM recommended). Lighthouse audits MUST run in CI against preview deployments. PRs that degrade metrics beyond thresholds require performance impact analysis and mitigation plans before merge approval.

## Governance

### Amendment Procedure

1. **Proposal**: Any project contributor MAY propose a constitutional amendment via pull request against `.specify/memory/constitution.md`.
2. **Discussion**: The proposal MUST remain open for review for a minimum of 7 calendar days to allow stakeholder feedback.
3. **Approval**: Amendments require approval from at least two project maintainers. Breaking changes (MAJOR version bumps) require unanimous maintainer approval.
4. **Version Update**: The `CONSTITUTION_VERSION` MUST be incremented according to semantic versioning:
   - **MAJOR**: Removal or redefinition of existing principles that invalidate prior guidance.
   - **MINOR**: Addition of new principles or material expansion of existing sections.
   - **PATCH**: Clarifications, typo fixes, or non-semantic wording improvements.
5. **Sync Impact Report**: The amendment pull request MUST include an updated Sync Impact Report (HTML comment at file top) documenting version change, modified sections, and downstream template updates required.
6. **Template Propagation**: Before merging, all referenced templates (plan, spec, tasks, commands) MUST be updated to reflect the amended principles.

### Versioning Policy

- The constitution version is independent of project release versions.
- Historical versions SHOULD be preserved via git tags (`constitution-v1.0.0`) for auditability.
- The `LAST_AMENDED_DATE` MUST reflect the merge date of the most recent amendment.

### Compliance Review

- **Quarterly Review**: Project maintainers MUST conduct a compliance audit every quarter to assess adherence to principles. Results SHOULD be documented in project meeting notes or issues.
- **Exception Tracking**: Approved exceptions to principles (granted during emergency fixes or technical constraints) MUST be logged as issues with `constitution-exception` label. Exceptions MUST include remediation plans or be escalated for constitutional amendment.
- **Onboarding**: New contributors MUST be directed to read this constitution as part of the contribution guidelines. First-time PR authors receive a bot comment linking to this document.

## References

- Project README: `/README.md`
- API Documentation: `/docs/API.md`
- Architecture Diagrams: `/backend/UML-DIAGRAMS-README.md`
- Deployment Guide: `/docs/DEPLOY.md`

---

*This constitution is a living document. Propose improvements via pull request.*