# Authorization

Authorization determines what an authenticated user is permitted to do. Marain CMS will implement a Role-Based Access Control (RBAC) system.

## Roles and Permissions

-   **Roles:** Collections of permissions (e.g., "Administrator", "Editor", "Contributor").
-   **Permissions:** Specific actions that can be performed (e.g., "create_entity", "edit_entity", "delete_user").

## Authorization Logic

```mermaid
graph TD
    A[API Request] --> B{Middleware};
    B --> C{Get User from Session};
    C --> D{Get User's Roles};
    D --> E{Get Role's Permissions};
    E --> F{Check if Required Permission Exists};
    F -->|Yes| G[Allow Access];
    F -->|No| H[Deny Access];
```

## Data Model

The authorization model will be stored in the `marain_user.db` database with the following tables:

-   `users`: Stores user information.
-   `roles`: Stores available roles.
-   `permissions`: Stores available permissions.
-   `user_roles`: Maps users to roles.
-   `role_permissions`: Maps roles to permissions.

## Implementation Guidelines

The authorization logic will be implemented as a custom Axum middleware.

-   **Middleware**: The middleware will extract the `AuthSession` to get the current user.
-   **Permission Check**: It will query the `marain_user.db` to fetch the user's roles and the permissions associated with those roles.
-   **Route Protection**: The middleware will be applied to specific routes or routers that require authorization. The required permission(s) will be specified as metadata on the route.
-   **Caching**: User roles and permissions can be cached in the session to reduce database lookups on subsequent requests. The `tower-sessions` `Session` object can be used for this purpose. When a user's roles or permissions are modified, the cache must be invalidated.