# Marain CMS Documentation

Welcome to the Marain CMS documentation hub. This resource is designed to help you understand the system, whether you are a content manager, a frontend developer consuming the API, or a backend developer extending the core.

## 🚀 Getting Started

If you're new to the project, here’s where to begin:

- **Project Setup & Commands**: See the main [**Project README**](../README.md) for instructions on how to build, run, and test the application.
- **Core Concepts**: For a deep dive into the architecture, data models, and development workflows, refer to our comprehensive [**Developer Guide**](./DEVELOPER-GUIDE.md).

## 📖 Key Documentation Areas

This documentation is structured to guide you to the information you need quickly.

### For All Developers

| Document | Description |
| :--- | :--- |
| [**Developer Guide**](./DEVELOPER-GUIDE.md) | **(Start Here)** The primary technical guide covering architecture, data models, API, and core concepts. |
| [**AGENTS.md**](../AGENTS.md) | **(Required Reading)** Outlines the rules for project structure, coding style, and testing that all contributors must follow. |
| [**DEV-TASKS.md**](./DEV-TASKS.md) | Tracks the current and past development tasks, providing insight into the project's evolution. |
| [**API Specification**](REST-API/openapi.json) | The official OpenAPI 3.0 specification for the system's RESTful API. |

### For Backend & Module Developers

| Document | Description |
| :--- | :--- |
| [**Entity Content Storage System**](./entity-content-storage-system/entity-content-storage.md) | Details how entities and their fields are mapped to the database schema. |
| [**Entity Management System**](./entity-management-system/entity-management.md) | Explains the entity trait system and how content types are loaded and managed. |
| [**Configuration Management System**](./configuration-management-system/configuration-management-system.md) | Describes the hot-reloading configuration system. |
| [**User Management System**](./user-management-system/user-management-system.md) | Covers the design of the user, authentication, and authorization systems. |
| [**ULID/UUID Conversion**](./user-management-system/ulid-uuid-conversion.md) | Explains the strategy for bridging ULIDs with the `webauthn-rs` library's UUID requirements. |

### For Content Administrators

*(Documentation for content managers and administrators is forthcoming.)*
