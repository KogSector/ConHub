# API Documentation

Welcome to the API documentation. This document provides a comprehensive overview of the available API endpoints, their functionalities, and how to interact with them.

## Introduction

The API provides a set of endpoints to manage and interact with the application's features. It is designed to be used by developers who want to build integrations or extend the functionality of the application.

## Authentication

All API requests must be authenticated using an API key. The API key must be included in the `Authorization` header of each request.

`Authorization: Bearer <YOUR_API_KEY>`

## API Endpoints

### Data Sources

#### GET /api/data-sources

- **Description**: Retrieves a list of all available data sources.
- **Request Parameters**: None
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `dataSources`: An array of data source objects.
- **Example Response**:
  ```json
  {
    "success": true,
    "dataSources": [
      {
        "id": "1",
        "name": "Data Source 1",
        "type": "postgres",
        "createdAt": "2023-01-01T00:00:00.000Z"
      },
      {
        "id": "2",
        "name": "Data Source 2",
        "type": "s3",
        "createdAt": "2023-01-02T00:00:00.000Z"
      }
    ]
  }
  ```
- **Error Handling**:
  - `500`: Internal server error
  - `502`: Failed to fetch data sources

---

#### POST /api/data-sources/connect

- **Description**: Connects a new data source to the application.
- **Request Parameters**:
  - `type`: The type of data source to connect (e.g., `github`, `bitbucket`).
  - `url`: The URL of the repository to connect.
  - `credentials`: An object containing the credentials required to access the data source.
  - `config`: An object containing any additional configuration required for the data source.
- **Example Request Body**:
  ```json
  {
    "type": "github",
    "url": "https://github.com/owner/repo",
    "credentials": {
      "accessToken": "<YOUR_ACCESS_TOKEN>"
    },
    "config": {
      "repositories": ["owner/repo"]
    }
  }
  ```
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `message`: A message indicating the result of the request.
  - `data`: An object containing the data returned from the API.
- **Example Response**:
  ```json
  {
    "success": true,
    "message": "Repository connected successfully",
    "data": {
      "id": "3",
      "name": "owner/repo",
      "type": "github",
      "createdAt": "2023-01-03T00:00:00.000Z"
    }
  }
  ```
- **Error Handling**:
  - `400`: Bad request (e.g., missing required parameters)
  - `500`: Internal server error
  - `502`: Failed to connect data source

---

#### POST /api/data-sources/{id}/sync

- **Description**: Starts a synchronization process for a specific data source.
- **Request Parameters**:
  - `id`: The ID of the data source to sync.
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `message`: A message indicating the result of the request.
  - `data`: An object containing the data returned from the API.
- **Example Response**:
  ```json
  {
    "success": true,
    "message": "Sync started successfully",
    "data": {
      "syncId": "4",
      "status": "in-progress"
    }
  }
  ```
- **Error Handling**:
  - `400`: Bad request (e.g., missing data source ID)
  - `500`: Internal server error
  - `502`: Failed to sync data source

---

### Billing

#### GET /api/billing/plans

- **Description**: Retrieves a list of all available subscription plans.
- **Request Parameters**: None
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `plans`: An array of plan objects.
- **Example Response**:
  ```json
  {
    "success": true,
    "plans": [
      {
        "id": "1",
        "name": "Basic",
        "price": 10,
        "currency": "USD",
        "interval": "month"
      },
      {
        "id": "2",
        "name": "Premium",
        "price": 20,
        "currency": "USD",
        "interval": "month"
      }
    ]
  }
  ```
- **Error Handling**:
  - `500`: Internal server error

---

#### GET /api/billing/subscription

- **Description**: Retrieves the current user's subscription details.
- **Request Parameters**: None
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `subscription`: A subscription object.
- **Example Response**:
  ```json
  {
    "success": true,
    "subscription": {
      "id": "sub_123",
      "planId": "1",
      "status": "active",
      "currentPeriodEnd": "2023-02-01T00:00:00.000Z"
    }
  }
  ```
- **Error Handling**:
  - `401`: Authorization required
  - `500`: Internal server error
  - `502`: Failed to fetch subscription

---

#### POST /api/billing/subscription

- **Description**: Creates a new subscription for the current user.
- **Request Parameters**:
  - `planId`: The ID of the plan to subscribe to.
- **Example Request Body**:
  ```json
  {
    "planId": "1"
  }
  ```
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `subscription`: The newly created subscription object.
- **Example Response**:
  ```json
  {
    "success": true,
    "subscription": {
      "id": "sub_123",
      "planId": "1",
      "status": "active",
      "currentPeriodEnd": "2023-02-01T00:00:00.000Z"
    }
  }
  ```
- **Error Handling**:
  - `401`: Authorization required
  - `500`: Internal server error
  - `502`: Failed to create subscription

---

#### PUT /api/billing/subscription

- **Description**: Updates the current user's subscription.
- **Request Parameters**:
  - `planId`: The ID of the new plan to switch to.
- **Example Request Body**:
  ```json
  {
    "planId": "2"
  }
  ```
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `subscription`: The updated subscription object.
- **Example Response**:
  ```json
  {
    "success": true,
    "subscription": {
      "id": "sub_123",
      "planId": "2",
      "status": "active",
      "currentPeriodEnd": "2023-02-01T00:00:00.000Z"
    }
  }
  ```
- **Error Handling**:
  - `401`: Authorization required
  - `500`: Internal server error
  - `502`: Failed to update subscription

---

#### GET /api/billing/dashboard

- **Description**: Retrieves the billing dashboard for the current user.
- **Request Parameters**: None
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `dashboard`: A dashboard object.
- **Example Response**:
  ```json
  {
    "success": true,
    "dashboard": {
      "url": "https://billing.example.com/dashboard/123"
    }
  }
  ```
- **Error Handling**:
  - `401`: Authorization required
  - `500`: Internal server error
  - `502`: Failed to fetch billing dashboard

---

### Agents

#### POST /api/agents/cline/query

- **Description**: Queries the Cline agent with a given prompt and context.
- **Request Parameters**:
  - `prompt`: The prompt to send to the agent.
  - `context`: The context to provide to the agent.
- **Example Request Body**:
  ```json
  {
    "prompt": "Hello, world!",
    "context": {
      "key": "value"
    }
  }
  ```
- **Response**:
  - `response`: The response from the agent.
- **Example Response**:
  ```json
  {
    "response": "Hello! How can I help you today?"
  }
  ```
- **Error Handling**:
  - `404`: Cline agent not found
  - `500`: Failed to query Cline agent

---

### Repositories

#### POST /api/repositories/fetch-branches

- **Description**: Fetches the branches for a given repository URL.
- **Request Parameters**:
  - `repoUrl`: The URL of the repository.
- **Example Request Body**:
  ```json
  {
    "repoUrl": "https://github.com/owner/repo"
  }
  ```
- **Response**:
  - `branches`: An array of branch names.
  - `defaultBranch`: The name of the default branch.
  - `provider`: The name of the repository provider (e.g., `github`, `bitbucket`).
  - `repoInfo`: An object containing additional information about the repository.
- **Example Response**:
  ```json
  {
    "branches": ["main", "dev"],
    "defaultBranch": "main",
    "provider": "github",
    "repoInfo": {
      "owner": "owner",
      "repo": "repo"
    }
  }
  ```
- **Error Handling**:
  - `400`: Bad request (e.g., missing repository URL)
  - `500`: Internal server error
  - `502`: Failed to fetch branches

---

#### POST /api/agents/amazon-q/query

- **Description**: Queries the Amazon Q agent with a given prompt and context.
- **Request Parameters**:
  - `prompt`: The prompt to send to the agent.
  - `context`: The context to provide to the agent.
- **Example Request Body**:
  ```json
  {
    "prompt": "Hello, world!",
    "context": {
      "key": "value"
    }
  }
  ```
- **Response**:
  - `response`: The response from the agent.
- **Example Response**:
  ```json
  {
    "response": "Hello! How can I help you today?"
  }
  ```
- **Error Handling**:
  - `404`: Amazon Q agent not found
  - `500`: Failed to query Amazon Q agent

---

#### POST /api/agents/cursor/query

- **Description**: Queries the Cursor IDE agent with a given prompt and context.
- **Request Parameters**:
  - `prompt`: The prompt to send to the agent.
  - `context`: The context to provide to the agent.
- **Example Request Body**:
  ```json
  {
    "prompt": "Hello, world!",
    "context": {
      "key": "value"
    }
  }
  ```
- **Response**:
  - `response`: The response from the agent.
- **Example Response**:
  ```json
  {
    "response": "Hello! How can I help you today?"
  }
  ```
- **Error Handling**:
  - `404`: Cursor IDE agent not found
  - `500`: Failed to query Cursor IDE agent

---

#### POST /api/agents/github-copilot/query

- **Description**: Queries the GitHub Copilot agent with a given prompt and context.
- **Request Parameters**:
  - `prompt`: The prompt to send to the agent.
  - `context`: The context to provide to the agent.
- **Example Request Body**:
  ```json
  {
    "prompt": "Hello, world!",
    "context": {
      "key": "value"
    }
  }
  ```
- **Response**:
  - `response`: The response from the agent.
- **Example Response**:
  ```json
  {
    "response": "Hello! How can I help you today?"
  }
  ```
- **Error Handling**:
  - `404`: GitHub Copilot agent not found
  - `500`: Failed to query GitHub Copilot agent

---

### Authentication

#### POST /api/auth/oauth/callback

- **Description**: Handles the OAuth callback from the authentication provider. This endpoint is responsible for creating or updating a user with the social connection information.
- **Request Parameters**:
  - `provider`: The name of the OAuth provider (e.g., `github`, `google`, `bitbucket`).
  - `provider_user_id`: The user's ID from the OAuth provider.
  - `email`: The user's email address.
  - `name`: The user's name.
  - `avatar_url`: The URL of the user's avatar.
  - `access_token`: The access token from the OAuth provider.
  - `refresh_token`: The refresh token from the OAuth provider.
  - `expires_at`: The expiration date of the access token.
  - `scope`: The scope of the access token.
- **Example Request Body**:
  ```json
  {
    "provider": "github",
    "provider_user_id": "12345",
    "email": "user@example.com",
    "name": "Test User",
    "avatar_url": "https://example.com/avatar.png",
    "access_token": "<ACCESS_TOKEN>",
    "refresh_token": "<REFRESH_TOKEN>",
    "expires_at": 1672531199,
    "scope": "read:user user:email"
  }
  ```
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `data`: An object containing the user's ID.
- **Example Response**:
  ```json
  {
    "success": true,
    "data": {
      "user_id": "user_123"
    }
  }
  ```
- **Error Handling**:
  - `500`: Failed to sync user with backend

---

### Logs

#### POST /api/logs

- **Description**: Receives and processes frontend logs.
- **Request Parameters**:
  - `sessionId`: The ID of the user's session.
  - `timestamp`: The timestamp of the log.
  - `logs`: An array of log entries.
  - `performance`: An array of performance entries.
  - `userActions`: An array of user action entries.
- **Example Request Body**:
  ```json
  {
    "sessionId": "session_123",
    "timestamp": "2023-01-01T00:00:00.000Z",
    "logs": [
      {
        "level": "info",
        "source": "frontend",
        "message": "User clicked a button"
      }
    ]
  }
  ```
- **Response**:
  - `success`: A boolean indicating whether the request was successful.
  - `message`: A message indicating the result of the request.
- **Example Response**:
  ```json
  {
    "success": true,
    "message": "Logs received"
  }
  ```
- **Error Handling**:
  - `500`: Failed to process logs

---
