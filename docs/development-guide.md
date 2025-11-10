# Development Guide

Welcome to the ConHub development guide. This document provides all the necessary information to get you started with building and testing the platform locally.

## 1. Core Development Tools

Our backend services are built in Rust and managed as a unified Cargo workspace. The frontend is a Next.js application.

### Key Commands

The entire project can be managed using a combination of `cargo` for the backend and `npm` for the frontend and orchestration scripts.

- **Build the entire workspace**:
  ```bash
  cargo build --workspace
  ```
- **Format all code**:
  ```bash
  cargo fmt --workspace
  ```
- **Run a specific service** (e.g., `auth-service`):
  ```bash
  cargo run -p auth-service
  ```

## 2. Feature Toggles for Efficient Development

To streamline development, we use feature toggles located in `feature-toggles.json`. These flags allow you to enable or disable parts of the system, significantly speeding up startup times and reducing local resource consumption.

```json
{
  "Auth": false,
  "Heavy": false,
  "Docker": false
}
```

| Flag | `false` (Default/Development) | `true` (Production-like) |
| :--- | :--- | :--- |
| **`Auth`** | Disables authentication and database connections. Ideal for frontend UI work. | Enables full authentication and requires database services to be running. |
| **`Heavy`** | Disables resource-intensive processes like embedding and indexing. | Enables all data processing features for full-stack testing. |
| **`Docker`** | Services run directly on the host machine for faster iteration and hot-reloading. | All services run inside Docker containers, simulating the production environment. |

**Note**: For more details on the Docker toggle, see the [Docker Toggle Feature Guide](DOCKER_TOGGLE_FEATURE.md).

## 3. Testing the Platform

Our test suite is designed to ensure code quality and stability.

### Running Tests

- **Run all backend tests**:
  ```bash
  cargo test --workspace
  ```
- **Run frontend tests**:
  ```bash
  cd frontend && npm test
  ```
- **Run end-to-end integration tests** (requires Docker):
  ```bash
  npm run test:docker
  ```

### Configuring Test Environments

You can simulate different environments by modifying `feature-toggles.json` before running tests:

- **Minimal UI Testing**:
  ```bash
  echo '{"Auth": false, "Heavy": false}' > feature-toggles.json
  ```
- **Testing with Authentication**:
  ```bash
  echo '{"Auth": true, "Heavy": false}' > feature-toggles.json
  ```
- **Full Stack Testing**:
  ```bash
  echo '{"Auth": true, "Heavy": true}' > feature-toggles.json
  ```

## 4. Interacting with the GraphQL API

The unified GraphQL API is the primary way to interact with the backend. The endpoint is available at `http://localhost:8000/api/graphql` during local development.

### Example Queries

- **Fetch user and repository data**:
  ```graphql
  query GetUserData {
    me {
      user_id
      roles
    }
    repositories {
      id
      name
      url
    }
  }
  ```

- **Generate embeddings**:
  ```graphql
  mutation GenerateEmbeddings {
    embed(texts: ["Hello, world!"], normalize: true) {
      embeddings
      dimension
    }
  }
