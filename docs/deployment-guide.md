# Deployment Guide

This guide outlines the procedures for deploying the ConHub platform to various environments, from local development to production on Azure.

## 1. Local Deployment with Docker Compose

For local development and staging environments, Docker Compose is the recommended tool for orchestrating the platform's services.

- **Standard Production Build**:
  This command builds and starts all services in a production-like configuration.
  ```bash
  docker-compose -f docker-compose.yml -f docker-compose.prod.yml up --build
  ```

- **Custom Environment Deployment**:
  You can specify the environment mode when starting the services.
  ```bash
  ENV_MODE=docker docker-compose up
  ```

## 2. Production Deployment on Azure Container Apps

For production, we use Azure Container Apps to host our services.

- **Deploy to Azure**:
  The deployment process is automated with a PowerShell script.
  ```bash
  ./deploy-to-azure.ps1 -Environment production
  ```

- **Monitor the Deployment**:
  You can monitor the logs of the deployed services using the Azure CLI.
  ```bash
  az containerapp logs show --name conhub-backend --resource-group conhub-rg
  ```

## 3. Performance and Monitoring

Monitoring the health and performance of the platform is crucial for maintaining stability and reliability.

### Key Performance Metrics

- **Build Performance**:
  - **Cargo Workspace**: Achieves 40-60% faster builds by sharing dependencies.
  - **Docker BuildKit**: Utilizes parallel builds and caching to accelerate container image creation.

- **Runtime Metrics**:
  - **GraphQL Federation**: A single API endpoint minimizes network latency.
  - **Connection Pooling**: 95% connection reuse reduces database overhead.
  - **Caching**: An 85% hit rate for embeddings significantly improves response times.
  - **Search Latency**: Vector queries are executed in under 50ms.

### Health Monitoring

- **Platform-Wide Health Check**:
  ```bash
  curl http://localhost/health
  ```

- **Individual Service Health Checks**:
  ```bash
  # Backend Service
  curl http://localhost:8000/health

  # Auth Service
  curl http://localhost:3010/health
  
  # AI Service
  curl http://localhost:3012/health
