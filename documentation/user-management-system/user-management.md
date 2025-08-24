# User Management

User management involves the creation, reading, updating, and deletion (CRUD) of user accounts.

## User Schema

The user schema defines the structure of user data stored in the `marain_user.db` database.

```yaml
id: user
name: User
fields:
  - id: username
    type: text
    required: true
    unique: true
  - id: email
    type: text
    required: true
    unique: true
  - id: backup-email
    type: text
    required: false
    unique: true
  - id: phone-number
    type: text
    required: true
    unique: true
  - id: backup-phone-number
    type: text
    required: true
    unique: true
  - id: passkey
    type: long_text
    required: false
  - id: magic_link_token
    type: text
    required: false
  - id: roles
    type: entity_reference
    target_entity: role
    cardinality: -1
```

## API Endpoints

The following API endpoints will be available for user management after RBAC is implemented, and will be protected by the authorization system.

-   `POST /api/v1/users`: Create a new user.
-   `GET /api/v1/users/{user_id}`: Retrieve a user's details.
-   `PUT /api/v1/users/{user_id}`: Update a user's details.
-   `DELETE /api/v1/users/{user_id}`: Delete a user.
-   `GET /api/v1/users`: List all users (with pagination).

## Implementation Guidelines

User management will be handled by a dedicated `user` crate.

-   **User Crate**: This crate will contain the business logic for user CRUD operations, interacting directly with the `marain_user.db`.
-   **API Handlers**: The API endpoints will be implemented in the `api` crate. These handlers will call functions from the `user` crate to perform the requested operations.
-   **Authorization**: Each handler will require specific permissions. For example, `POST /api/v1/users` might require the `create_user` permission. This will be enforced by the authorization middleware.
-   **Data Validation**: Input data from API requests will be validated before being passed to the `user` crate. This includes checking for required fields, valid email formats, etc.