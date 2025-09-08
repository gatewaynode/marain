# User Management System

The User Management System is responsible for handling all aspects of user authentication, authorization, and data management within Marain CMS. It is designed to be secure, scalable, and extensible, providing a robust foundation for managing user access and permissions.

## Core Components

The system is comprised of three main components, implemented in the `src-tauri/user/` crate:

1.  **[Authentication](./authentication.md):** Verifies the identity of users attempting to access the system. Supports passwordless methods like magic links ([src-tauri/user/src/auth/magic_link.rs](src-tauri/user/src/auth/magic_link.rs)) and passkeys ([src-tauri/user/src/auth/passkey.rs](src-tauri/user/src/auth/passkey.rs)), with session management via [session.rs](src-tauri/user/src/auth/session.rs). All auth flows use secure logging ([secure_log.rs](src-tauri/user/src/secure_log.rs)).

2.  **[Authorization](./authorization.md):** Determines what actions an authenticated user is allowed to perform, based on roles and permissions stored in the user database.

3.  **[User Management](./user-management.md):** Provides interfaces for creating, reading, updating, and deleting user accounts, with database operations in [database.rs](src-tauri/user/src/database.rs). Includes ULID/UUID conversion bridge for compatibility with libraries like webauthn-rs ([ulid_uuid_bridge.rs](src-tauri/user/src/ulid_uuid_bridge.rs)).

### Flow

These steps are always performed in this order:

1. Authentication
2. Authorization
3. User Management

And every action is always logged to the secure.log in `{DATA_PATH}/user-backend/secure.log`. See [DEVELOPER-GUIDE.md](../DEVELOPER-GUIDE.md#critical-path-configurations--development-workflow) for data path details.

## High-Level Architecture

```mermaid
graph TD
    subgraph "User Management System Flow"
        A[Authentication] --> B[Authorization] --> C[User Management]
    end

    subgraph "Data Stores"
        D[User Database: marain_user.db]
        E[Secure Log: secure.log]
    end

    A --> D
    B --> D
    C --> D

    A --> E
    B --> E
    UM --> Bridge

## Security Best Practices

To ensure the User Management System remains secure, follow these guidelines, aligned with the core principles in [DEVELOPER-GUIDE.md](../DEVELOPER-GUIDE.md#core-principles--requirements):

- **Input Validation and Sanitization:** All user inputs (e.g., emails in magic links, credentials in passkeys) must be validated using parameterized queries in SQLx to prevent SQL injection. Use the hashing utilities from `src-tauri/content/src/hashing.rs` for any password-related operations.

- **Secure Logging:** All authentication, authorization, and management actions are logged to `secure.log` without sensitive data (e.g., no plain-text passwords). Logs are rotated and encrypted if needed for production.

- **ID Management:** Standardize on ULIDs for internal IDs, converting to UUIDs only at boundaries (e.g., webauthn-rs integration) using the ULID/UUID bridge to avoid inconsistencies.

- **Session Security:** Sessions use secure tokens with short TTLs; implement rate limiting on auth endpoints to prevent brute-force attacks.

- **Dependency Security:** Keep dependencies like Tokio, SQLx, and webauthn-rs updated to latest stable versions. Run regular audits with `cargo audit`.

- **Testing:** Include unit tests for auth flows in `src-tauri/user/` and E2E tests for user management in the frontend. Verify no sensitive data leaks in logs or responses.

For database flexibility (SQLite dev, PostgreSQL prod), see [DEVELOPER-GUIDE.md](../DEVELOPER-GUIDE.md#database-flexibility).