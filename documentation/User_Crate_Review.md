# User Crate Design and Security Review

**Date:** 2025-08-30

## 1. Overview

This document provides a comprehensive review of the `user` crate within the Marain CMS project. The review assesses the crate's alignment with documented architectural principles, identifies potential security vulnerabilities, evaluates coding practices, and offers concrete suggestions for improvement.

The `user` crate is a critical component responsible for authentication, authorization, session management, and secure logging. The implementation demonstrates a strong foundation, leveraging robust libraries like `sqlx`, `axum-login`, `tower-sessions`, and `webauthn-rs`. The design generally adheres to the principles outlined in the project's documentation, particularly concerning the separation of the user database and the use of a cryptographically verifiable secure log.

## 2. Design and Architecture Review

The overall architecture of the `user` crate is well-structured and aligns with the modular design principles of the Marain CMS.

### Strengths:

*   **Modular Design:** The crate is well-organized into submodules (`auth`, `database`, `secure_log`), which aligns perfectly with the documented principles of separation of concerns.
*   **Dependency Injection:** The `UserManager` and its dependencies (`UserDatabase`, `AuthBackend`, `SecureLogger`) are initialized in the `app` crate and passed down, avoiding global statics and promoting testability, as per `DEV-TASKS.md` (Task 11).
*   **Secure Data Storage:** The use of a separate SQLite database (`marain_user.db`) in the `data/user-backend/` directory for sensitive user data is an excellent security practice that isolates it from general content.
*   **Comprehensive Logging:** The `SecureLogger` is a standout feature. The concept of a chained, hashed log for audibility and tamper-resistance is a strong security control.
*   **Configuration:** The use of `UserDatabaseConfig`, `SessionConfig`, and `MagicLinkConfig` structs allows for flexible configuration, which is consistent with the project's configuration-as-code philosophy.

### Areas for Improvement:

*   **Inconsistent ULID/UUID Usage:** The `Cargo.toml` file includes both `ulid` and `uuid` dependencies. The documentation and `DEV-TASKS.md` (Task 17) clearly state a migration to ULIDs for their sortable nature. However, the `passkey.rs` module still uses `uuid::Uuid::new_v4()` for the `user_uuid` in the WebAuthn flow. This should be standardized to `ulid` to maintain consistency and leverage the performance benefits of ULIDs across the entire system.
*   **Mocked Passkey Verification:** The `verify_passkey` function in `passkey.rs` is currently a stub that only checks for user existence. The comment `// PassKey verification not fully implemented - using mock verification` highlights this. While acceptable for initial scaffolding, this is a critical security function that must be fully implemented.
*   **Redundant `get_pool` Method:** The `UserDatabase` struct in `database.rs` contains two identical methods `pool()` and `get_pool()`. The `get_pool()` method is noted as deprecated in the doc comment and should be removed to clean up the API.
*   **Hardcoded Table Names:** Database table names are hardcoded as string literals within the migration queries in `database.rs`. While not a major issue, abstracting these into constants could reduce the risk of typos and make future refactoring easier.

## 3. Security Review

The crate demonstrates a strong security posture, but there are several areas where it can be hardened.

### Strengths:

*   **Secure Audit Log:** The `SecureLogger` is a robust implementation for creating a tamper-evident audit trail of all sensitive user actions.
*   **Session Management:** The use of `tower-sessions` with an `sqlx` backend and encrypted cookies (via `secret_key` in `SessionConfig`) provides secure session management.
*   **Authentication Methods:** The choice of PassKeys and Magic Links as the primary authentication methods aligns with modern passwordless security best practices.
*   **Database Isolation:** Separating the user database from the content database is a critical security measure that has been implemented correctly.

### Areas for Improvement:

*   **In-Memory Secret Key:** The `SessionConfig::default()` implementation in `store.rs` generates a random session encryption key *in memory*. This means every application restart will generate a new key, invalidating all existing user sessions. For a better user experience and proper session persistence, this key should be loaded from a secure configuration source (like an environment variable or a secrets management system) rather than being generated on the fly.
*   **Email Visibility in Magic Link:** The `send_magic_link` function in `magic_link.rs` returns early if a user email is not found. While it logs this action internally, this design prevents sending a generic "If an account with this email exists, a link has been sent" message to the user, which is a best practice to avoid user enumeration attacks. The current implementation doesn't directly expose this to an API, but it's a pattern to be mindful of.
*   **Unused `argon2` Dependency:** The `Cargo.toml` includes the `argon2` crate, presumably for future password-based authentication. If passwords are not on the immediate roadmap, this dependency should be removed to reduce the crate's attack surface. Unused dependencies can become a liability if they are not updated and security vulnerabilities are discovered in them.
*   **TODO in Passkey Implementation:** The `passkey.rs` file has several areas that are not fully implemented, most critically the verification logic. Completing this is a top security priority.

## 4. Consistency and Best Practices Review

The crate generally follows Rust best practices and the conventions outlined in `AGENTS.md`.

### Strengths:

*   **Error Handling:** The use of `thiserror` to define a custom `UserError` enum provides clear, structured error handling.
*   **Testing:** The crate includes a good number of unit tests for core functionality, such as database initialization and secure log hashing.
*   **Code Documentation:** Most public functions and structs are well-documented with explanations of their purpose.

### Areas for Improvement:

*   **Dependency Management:** The `tower` dependency is specified as version `0.5` while the `api` crate uses `0.4`. While Cargo might resolve this, it's better to align versions across the workspace to avoid potential compatibility issues, as noted in `DEV-TASKS.md` (Task 20.2).
*   **Code Comments:** While documentation is good, some complex logic, like in `passkey.rs`, could benefit from more inline comments explaining the multi-step WebAuthn flow.
*   **Clarity on `SessionManager` Role:** In 'session.rs' the `SessionManager` struct holds a `SessionConfig` but doesn't seem to use it directly in its methods (`create_session`, etc.). The session behavior is primarily driven by the `tower_sessions::Session` object passed into its methods. The name `SessionManager` might imply it manages the lifecycle of sessions, but its current role is more of a helper for session data manipulation. Clarifying its role or refactoring could improve understanding.

## 5. Summary and Recommendations

The `user` crate is a well-designed and robust component that provides a strong foundation for user management in Marain CMS. The developers have clearly prioritized security and modularity. The following recommendations are offered to further enhance the crate's quality:

**Critical:**

1.  **Complete Passkey Implementation:** Fully implement the PassKey verification logic in `passkey.rs` and remove the mock verification. This is the highest priority security fix.
2.  **Externalize Session Secret Key:** Modify [`SessionConfig`](src-tauri/user/src/auth/store.rs:86) to load the `secret_key` from a persistent, secure source instead of generating it at runtime.

**Recommended:**

3.  **Standardize on ULIDs:** Replace the usage of `uuid` with `ulid` in [`passkey.rs`](src-tauri/user/src/auth/passkey.rs:1) to align with the project's stated architectural decision.
4.  **Align `tower` Dependency:** Update the `tower` dependency in [`Cargo.toml`](src-tauri/user/Cargo.toml:39) to match the version used in other crates to ensure workspace consistency.
5.  **Remove Unused `argon2` Dependency:** If password-based authentication is not planned for the near future, remove the `argon2` crate from [`Cargo.toml`](src-tauri/user/Cargo.toml:48).
6.  **Refactor `UserDatabase`:** Remove the deprecated [`get_pool()`](src-tauri/user/src/database.rs:375) method.

By addressing these points, the `user` crate will be even more secure, consistent, and maintainable, solidifying its role as a cornerstone of the Marain CMS application.

---

## 6. Addendum: `PassKeyManager` Instantiation and State Management

Following up on the review, this addendum addresses a key point regarding the instantiation of the `PassKeyManager` within the `verify_passkey` function in `src-tauri/user/src/auth/passkey.rs`.

### The Issue

The current implementation creates a new `PassKeyManager` on every verification call with hardcoded development values:

```rust
// From src-tauri/user/src/auth/passkey.rs
let passkey_manager =
    PassKeyManager::new("localhost".to_string(), "http://localhost:3000".to_string())?;
```

This approach, while functional for initial development, presents two significant problems for a production-ready system:

1.  **Configuration Rigidity:** The WebAuthn standard requires that the **Relying Party (RP) ID** and **RP Origin** strictly match the domain from which the authentication request originates. Hardcoding these to `localhost` means that PassKey authentication will fail in any environment other than local development.
2.  **Inefficient State Management:** Creating a manager on every API call is inefficient. The application's architecture already utilizes a dependency injection pattern with a shared `AppState`. The `PassKeyManager` should be treated as a shared resource and managed within this state.

### The Solution: Dependency Injection for `PassKeyManager`

To align with the project's architecture and ensure the system is secure and configurable, the `PassKeyManager` should be instantiated once at application startup and shared via `AppState`.

This involves the following steps:

1.  **Add Configuration:** The WebAuthn RP ID and Origin should be added to the system configuration file (e.g., `config.system.dev.yaml`) to allow for environment-specific values.
2.  **Update `UserManager`:** The `UserManager` in `src-tauri/user/src/lib.rs` should be updated to create and hold an `Arc<PassKeyManager>`.
3.  **Update `AppState`:** The `AppState` struct in the `app` crate should hold the `UserManager` instance, making the `PassKeyManager` accessible to all components that have access to the application state.
4.  **Refactor `verify_passkey`:** The `verify_passkey` function should be refactored to receive the `PassKeyManager` as a parameter (likely through the `AuthBackend` which would access it from `AppState`) instead of creating it internally.

By implementing this change, the `PassKeyManager` will be:

*   **Correctly Configured:** It will use the appropriate RP ID and Origin for the environment it's running in.
*   **Efficient:** It will be created only once, reducing overhead on authentication requests.
*   **Consistent:** It will follow the established dependency injection pattern, making the codebase cleaner and more maintainable.