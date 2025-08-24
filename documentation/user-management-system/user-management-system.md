# User Management System

The User Management System is responsible for handling all aspects of user authentication, authorization, and data management within Marain CMS. It is designed to be secure, scalable, and extensible, providing a robust foundation for managing user access and permissions.

## Core Components

The system is comprised of three main components:

1.  **[Authentication](./authentication.md):** Verifies the identity of users attempting to access the system.
2.  **[Authorization](./authorization.md):** Determines what actions an authenticated user is allowed to perform.
3.  **[User Management](./user-management.md):** Provides interfaces for creating, reading, updating, and deleting user accounts.

### Flow

These steps are always performed in this order: 

1. Authentication
2. Authorization
3. User Management

And every action is always logged to the secure.log

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
    C --> E