# CEDAR Authorization Implementation Plan

This document outlines the plan to integrate the [CEDAR](https://www.cedarpolicy.com/) policy language for fine-grained authorization within the Marain CMS. This will replace the basic RBAC placeholder and provide a more powerful and flexible authorization model.

## 1. Core Objectives

-   **Integrate CEDAR:** Use the `cedar-policy` Rust crate to create a robust authorization engine.
-   **Policy as Code:** Store CEDAR policies in `.cedar` files within the `/config` directory, enabling hot-reloading.
-   **Dynamic Entities:** Represent CMS content types, users, and roles as CEDAR entities.
-   **Hierarchical Roles:** Support dynamic group hierarchies stored in the database for flexible permission modeling.
-   **Schema-Driven Policies:** Allow entity schemas (`.schema.yaml`) to contain authorization-related metadata.

## 2. CEDAR Integration Architecture

We will introduce a new `authz` crate and an `AuthorizationMiddleware` to handle policy decisions.

```mermaid
graph TD
    subgraph "API Request Flow"
        direction LR
        A[Incoming Request] --> B[Axum Router];
        B --> C[Authn Middleware];
        C --> D[Authorization Middleware];
        D --> E[Route Handler];
    end

    subgraph "Authorization Middleware Logic"
        direction TB
        F[1. Extract Principal, Action, Resource] --> G[2. Load Policies & Schema];
        G --> H[3. Construct CEDAR Entities];
        H --> I[4. Call is_authorized[]];
        I --> J{Decision};
    end

    subgraph "Data Sources"
        direction TB
        K[config/*.cedar] --> L[Policy Set];
        M[schemas/*.schema.yaml] --> N[CEDAR Schema];
        O[User & Content DBs] --> P[CEDAR Entities];
    end

    J --> |Allow| E;
    J --> |Deny| Q[Return 403 Forbidden];
    L & N & P --> H;
```

## 3. Implementation Tasks

Here is the breakdown of the tasks required to implement this system.

1.  **Create `authz` Crate**
    *   Create a new crate `src-tauri/authz`.
    *   Add `cedar-policy` and other necessary dependencies (`tokio`, `serde`, `thiserror`).
    *   Define the core `AuthzEngine` struct responsible for loading policies and making authorization decisions.

2.  **Define Core Authz Data Structures**
    *   In `authz/src/types.rs`, define structs for `Principal`, `Action`, and `Resource` that can be constructed from request data.
    *   Example `Action`: `(type="API", id="read_entity")`
    *   Example `Resource`: `(type="Content", id="article::some-ulid")`

3.  **Implement Policy & Schema Loading**
    *   Modify the `schema-manager` crate to watch for `.cedar` files in the `config/` directory.
    *   The `schema-manager` will load all `.cedar` files into a single `PolicySet`.
    *   Implement logic to transform the existing YAML entity schemas into a CEDAR schema. This will allow CEDAR to validate policies against known entity types and attributes.

4.  **Implement Entity Construction**
    *   Create functions to convert Marain entities into CEDAR entities.
    *   **Users:** The `user` entity from `user.schema.yaml` will be the `Principal`. It should include its roles.
    *   **Content Entities:** Content items from the database will be `Resources`.
    *   **Groups/Roles:** Create a mechanism to represent dynamic user groups or roles from the user database as CEDAR entities, allowing for parent-child relationships.

5.  **Implement `AuthorizationMiddleware`**
    *   Replace the stubbed-out `auth_middleware` in `src-tauri/api/src/middleware_hooks.rs` with the new authorization middleware.
    *   The middleware will:
        *   Extract the authenticated user (`Principal`).
        *   Determine the `Action` and `Resource` from the request path and method.
        *   Call the `AuthzEngine.is_authorized()` method.
        *   Proceed to the next handler on `Allow` or return `403 Forbidden` on `Deny`.

6.  **Extend Entity Schemas for Authorization**
    *   Add a new optional `cedar` section to the entity schema (`.schema.yaml`) format.
    *   This section can define the CEDAR entity type mapping and principal/resource relationships.
    *   Example for `schemas/article.schema.yaml`:
        ```yaml
        id: article
        name: Article
        cedar:
          # This content type can be a resource in a policy
          is_resource: true
          # Maps to the 'Marain::Article' entity type in CEDAR
          entity_type: "Marain::Article"
          # Defines which field links to the owner
          owner_field: "author" # 'author' is an entity_reference to a 'user'
        ```

7.  **Create Initial Policies**
    *   Create default policies in `config/policies.cedar`.
    *   Example Policy 1: An admin can do anything.
        ```cedar
        permit(
            principal_is "Marain::Role::\"admin\"",
            action,
            resource
        );
        ```
    *   Example Policy 2: A user can read any public article.
        ```cedar
        permit(
            principal,
            action == Action::"read_entity",
            resource
        )
        when { resource.visibility == "public" };
        ```
    *   Example Policy 3: The owner of an article can edit it.
        ```cedar
        permit(
            principal,
            action == Action::"update_entity",
            resource
        )
        when { principal == resource.owner };
        ```

8.  **Write Comprehensive Tests**
    *   Create a test suite in the `authz` crate.
    *   Test policy loading and validation.
    *   Test authorization decisions with mock `Principal`, `Action`, and `Resource` data.
    *   Create integration tests in the `api` crate to test the middleware with real API requests.

## 4. Todo List

Here is the checklist of tasks to be performed.

- [ ] 1. Create `src-tauri/authz` crate and add dependencies.
- [ ] 2. Define `Principal`, `Action`, `Resource` structs in `authz` crate.
- [ ] 3. Update `schema-manager` to load `.cedar` policies and generate a CEDAR schema.
- [ ] 4. Implement logic to convert Marain users and content into CEDAR entities.
- [ ] 5. Implement the `AuthorizationMiddleware` in the `api` crate.
- [ ] 6. Update entity schema format to include an optional `cedar` block.
- [ ] 7. Create initial set of default `.cedar` policies in `/config`.
- [ ] 8. Implement unit and integration tests for the authorization system.
- [ ] 9. Update existing documentation (`DEVELOPER-GUIDE.md`, `user-management-system/authorization.md`) to reflect the new CEDAR-based system.
- [ ] 10. Update `AGENTS.md` with guidelines for creating and modifying policies.

This plan provides a clear path to implementing a powerful, policy-driven authorization system using CEDAR.