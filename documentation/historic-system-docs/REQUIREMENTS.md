# Requirements Document

## Introduction

Marain CMS is a headless, API-first content management system designed with modularity and configuration-as-code principles. The system consists of a Rust-based backend providing a RESTful API and a Svelte 5 frontend wrapped in Tauri for desktop deployment. The architecture emphasizes developer experience through LLM-assisted development workflows and comprehensive testing strategies.

## Requirements

### Requirement 1

**User Story:** As a developer, I want a headless CMS with a RESTful API, so that I can build multiple frontend experiences and integrations that consume the same content backend.

#### Acceptance Criteria

1. WHEN the system is deployed THEN it SHALL expose a versioned RESTful API with paths prefixed by `/api/v1/`
2. WHEN API requests are made THEN the system SHALL respond with JSON data
3. WHEN the API specification is updated THEN all endpoints SHALL conform to the OpenAPI 3.0 specification
4. IF the system is running THEN it SHALL operate independently of any frontend application
5. WHEN in development mode THEN the system SHALL expose test endpoints under `/api/v1/test/` that are not compiled into production builds

### Requirement 2

**User Story:** As a content manager, I want to manage content through a desktop application, so that I can have a native experience for content creation and editing.

#### Acceptance Criteria

1. WHEN the application is launched THEN it SHALL display a Svelte 5-based admin interface
2. WHEN content is created or edited THEN the interface SHALL communicate with the backend API
3. WHEN the application is built THEN it SHALL be packaged as a Tauri desktop application
4. IF the user interacts with the interface THEN it SHALL provide real-time feedback and validation

### Requirement 3

**User Story:** As a developer, I want modular functionality, so that I can extend the system with custom features without modifying core code.

#### Acceptance Criteria

1. WHEN a new module is created THEN it SHALL be implemented as a separate Rust crate
2. WHEN modules are loaded THEN they SHALL register their functionality through a hook system
3. WHEN module API endpoints are defined THEN they SHALL be documented in the main OpenAPI specification
4. IF a module defines entities THEN they SHALL be specified in a schema.json file

### Requirement 4

**User Story:** As a developer, I want configuration as code, so that I can version control site structure and automate deployments.

#### Acceptance Criteria

1. WHEN content types are defined THEN they SHALL be specified in YAML schema files
2. WHEN field groups are created THEN they SHALL be reusable across multiple entities
3. WHEN the system starts THEN it SHALL load all schema definitions from the schemas directory
4. IF schema files are updated THEN the system SHALL reflect changes without manual database migrations

### Requirement 5

**User Story:** As a content creator, I want flexible content modeling, so that I can create complex data structures that match my content needs.

#### Acceptance Criteria

1. WHEN defining entities THEN I SHALL be able to use primitive field types (text, integer, boolean, datetime, etc.)
2. WHEN creating complex structures THEN I SHALL be able to nest fields using component types
3. WHEN establishing relationships THEN I SHALL be able to reference other entities using entity_reference fields
4. IF field cardinality is specified THEN the system SHALL enforce single or multiple value constraints

### Requirement 6

**User Story:** As a developer, I want comprehensive testing capabilities, so that I can ensure system reliability and prevent regressions.

#### Acceptance Criteria

1. WHEN backend code is written THEN it SHALL include corresponding Rust unit tests
2. WHEN frontend components are created THEN they SHALL include Playwright end-to-end tests
3. WHEN tests are executed THEN they SHALL run against a locally running development server
4. IF any test fails THEN the build process SHALL prevent deployment

### Requirement 7

**User Story:** As a developer, I want database flexibility, so that I can choose between local development and production database solutions.

#### Acceptance Criteria

1. WHEN developing locally THEN the system SHALL support SQLite as the default database
2. WHEN deploying to production THEN the system SHALL optionally support PostgreSQL
3. WHEN database operations are performed THEN they SHALL use the SQLx abstraction layer
4. IF database migrations are needed THEN they SHALL be handled automatically by the system

### Requirement 8

**User Story:** As a developer, I want LLM-assisted development workflows, so that I can efficiently build and extend the system with AI assistance.

#### Acceptance Criteria

1. WHEN making API changes THEN the OpenAPI specification SHALL be updated first
2. WHEN creating content models THEN schema files SHALL be created or updated first
3. WHEN generating code THEN it SHALL conform to the existing architecture patterns
4. IF verification commands are available THEN they SHALL be run to ensure code quality and correctness

### Requirement 9

**User Story:** As a developer, I want every development task to result in a manually testable deliverable, so that I can manually confirm the functionality works as expected.

#### Acceptance Criteria

1. WHEN creating tasks THEN the manual testing of each piece of functionality SHALL be considered
2. WHEN manual testing is needed THEN admin UI testing SHALL be preferred but CLI tests are acceptable
3. WHEN implementing tasks THEN the manual testing SHALL be completed and documented before marking the task complete
4. IF manual testing fails THEN the developer SHALL be consulted on the failure and suitable automated tests SHALL be created

### Requirement 10

**User Story:** As a developer, I want test endpoints available during development, so that I can verify system functionality without affecting production data.

#### Acceptance Criteria

1. WHEN in development mode THEN test endpoints SHALL be available under `/api/v1/test/` path
2. WHEN building for production THEN test endpoints SHALL be excluded from the compiled binary
3. WHEN test endpoints are called THEN they SHALL provide simple responses for connectivity and functionality testing
4. IF the system is in production mode THEN test endpoints SHALL return 404 or be completely unavailable