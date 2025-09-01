# Agent Documentation Hub

This document provides a streamlined entry point for LLM agents to access the necessary context for development tasks within the Marain CMS project.

## Core Objective

Your primary objective is to assist in developing, extending, and maintaining the Marain CMS by following the established architectural patterns and coding standards.

## 1. 📜 **Rules of Engagement (Required Reading)**

Before performing any action, you **MUST** adhere to the rules and guidelines outlined in the root `AGENTS.md` file. This document details project structure, build commands, coding styles, and testing protocols.

- **Primary Ruleset**: [**AGENTS.md**](../AGENTS.md)

## 2. 🏗️ **Architectural & Technical Context**

For a complete understanding of the system's architecture, data models, API, and development workflows, refer to the comprehensive Developer Guide.

- **Primary Technical Document**: [**Developer Guide**](./DEVELOPER-GUIDE.md)

### Key Architectural Concepts to Understand:

| Concept | Location in Developer Guide | Summary |
| :--- | :--- | :--- |
| **High-Level Architecture** | [Link](./DEVELOPER-GUIDE.md#2-system-architecture) | Headless Rust backend (API) with a Tauri/Svelte frontend. Modular, with a focus on configuration-as-code. |
| **Directory & Crate Structure** | [Link](./DEVELOPER-GUIDE.md#3-directory--crate-structure) | `src-tauri/` is a Cargo workspace with distinct crates for API, database, entities, etc. |
| **API & Request Lifecycle** | [Link](./DEVELOPER-GUIDE.md#4-api--request-lifecycle) | Axum-based API using middleware, ReDB caching, and a specific request flow. |
| **Data Modeling (Entities)** | [Link](./DEVELOPER-GUIDE.md#5-data-modeling--storage) | Content types are defined in YAML schemas. The database structure is generated from these schemas. |
| **Development Workflow** | [Link](./DEVELOPER-GUIDE.md#6-critical-path-configurations--development-workflow) | **MUST** update schemas or `openapi.yaml` *before* generating code. Use verification commands. |
| **ULID/UUID Bridge** | [Link](./DEVELOPER-GUIDE.md#6-critical-path-configurations--development-workflow) | Use ULIDs internally. Convert to UUID only when interfacing with `webauthn-rs`. |

## 3. 🎯 **Task Execution Workflow**

1.  **Analyze Task**: Understand the user's request.
2.  **Consult Documentation**: Use this guide and the linked documents to find the relevant context.
3.  **Update Spec/Schema First**: For any changes affecting the API or data model, modify `openapi.yaml` or the relevant `.schema.yaml` file first.
4.  **Generate Code**: Write or modify Rust/Svelte code that adheres to the patterns in the Developer Guide.
5.  **Generate Tests**: Create corresponding unit or E2E tests.
6.  **Verify**: Run the verification commands specified in the [Developer Guide](./DEVELOPER-GUIDE.md#verification-commands) and `AGENTS.md`.