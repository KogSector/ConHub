# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

ConHub is a comprehensive AI-powered platform that connects multiple knowledge sources (repositories, documents, URLs) with AI agents through the Model Context Protocol (MCP). It features semantic search, code indexing, document processing, and seamless agent integration across all connected data sources.

## Development Commands

### Essential Commands

#### Start Services
```bash
npm start                    # Cross-platform startup (recommended)
npm run start:windows        # Windows PowerShell scripts
npm run start:linux         # Linux/macOS shell scripts
```

#### Stop Services
```bash
npm run stop:windows         # Windows
npm run stop:linux          # Linux/macOS  
```

#### Health Checks & Status
```bash
npm run status              # Quick service status check
npm run check:services      # Comprehensive health check with endpoint testing
npm run health              # Alias for diagnose command
npm run diagnose            # Performance diagnostics across all services
```

### Development Commands

#### Frontend Development
```bash
npm run dev:frontend        # Production build + start frontend (port 3000)
npm run dev:frontend-fast   # Development mode with hot reload (port 3000)
npm run build:frontend      # Build frontend for production
npm run start:frontend      # Start frontend from build
npm run lint                # Run ESLint on frontend code
npm run lint:fix           # Auto-fix ESLint issues
```

#### Backend Development
```bash
npm run dev:backend         # Start Rust backend server (port 3001)
npm run dev:lexor          # Start Lexor code indexing service (port 3002)
npm run build:backend      # Build backend services for production
```

#### AI Services
```bash
npm run dev:ai             # Start Python AI service with hot reload (port 8001)
```

#### Database Management
```bash
# Windows
.\scripts\setup-database.ps1     # Setup PostgreSQL database and schema
.\scripts\test-database.ps1      # Test database connection
.\scripts\test-settings.ps1      # Test settings API endpoints

# Linux/macOS
./scripts/setup-database.sh      # Setup PostgreSQL database and schema
./scripts/test-database.sh       # Test database connection
```

### Build Commands
```bash
npm run build               # Build all services for production
npm run install:all         # Install all dependencies across services
```

### Testing & Diagnostics
```bash
npm run test:speed          # Test frontend performance
npm run test:settings       # Test settings API endpoints (Windows)
npm run test:settings:linux # Test settings API endpoints (Linux)
npm run architecture:summary # Generate architecture overview (Windows)
npm run cleanup:langchain   # Clean up LangChain service artifacts (Windows)
```

### Running Single Tests
For Rust backend tests:
```bash
cd backend && cargo test <test_name>  # Run specific test
cd backend && cargo test --lib        # Run library tests only
cd backend && cargo test auth         # Run auth-related tests
```

For JavaScript/TypeScript tests:
```bash
cd frontend && npm test <test_file>   # Run specific test file
cd langchain-service && npm test     # Run LangChain service tests
```

## High-Level Architecture

ConHub implements a **microservices architecture** with four main services that communicate via HTTP APIs and share data through a central PostgreSQL database.

### Service Architecture

#### 1. Frontend Service (Next.js - Port 3000)
- **Technology**: Next.js 14 with App Router, React 18, TypeScript
- **UI Framework**: Tailwind CSS + shadcn/ui components
- **Key Features**: Real-time dashboard, search interface, authentication flows, easy connector
- **File Location**: `frontend/` directory

#### 2. Backend Service (Rust - Port 3001) 
- **Technology**: Actix Web with advanced logging and monitoring
- **Database**: PostgreSQL with SQLx for type-safe queries
- **Key Components**:
  - Authentication service with JWT tokens
  - Social platform integrations (GitHub, Bitbucket, Google Drive, Notion)
  - MCP (Model Context Protocol) server implementation
  - Feature toggle service
  - Performance monitoring with metrics
- **File Location**: `backend/src/main.rs` and modules

#### 3. Lexor Service (Rust - Port 3002)
- **Technology**: Rust with Tantivy search engine
- **Purpose**: Lightning-fast code indexing and semantic search
- **Key Features**: 
  - Tree-sitter syntax analysis for multiple languages
  - Git integration and history tracking
  - AI context generation for code understanding
  - Symbol extraction and dependency mapping
- **File Location**: `lexor/src/` directory

#### 4. AI Service (Python - Port 8001)
- **Technology**: FastAPI with Haystack framework
- **Purpose**: Document processing and AI-powered Q&A
- **Key Features**:
  - Unified AI agent management
  - Vector embeddings and semantic search
  - Document upload and processing
  - Local embedding models support
- **File Location**: `ai-service/` directory

#### 5. LangChain Service (Node.js - Port 3003)
- **Technology**: Express.js with TypeScript
- **Purpose**: Extended AI operations and agent orchestration
- **Key Features**:
  - Advanced AI agent routing
  - GitHub Copilot integration
  - Search and indexing coordination
- **File Location**: `langchain-service/src/` directory

### Data Flow Architecture

**Authentication Flow:**
1. Frontend → Backend (`/auth` endpoints) → JWT token generation
2. Tokens stored securely with session management
3. Feature toggles control authentication bypass for development
4. We have to add Auth0 authentication flows with Google, GitHub, Microsoft and GitLab

**Data Source Integration:**
1. Backend connectors (`social_integration_service`) fetch data from external APIs
2. Raw data stored in PostgreSQL (`social_data` table)
3. Processed data indexed by Lexor and AI services
4. Search results aggregated and cached

**AI Query Processing:**
1. Frontend query → LangChain service coordination
2. Context building from multiple services (Lexor, AI service, Backend)
3. MCP protocol for standardized AI context sharing
4. Results aggregated and returned to frontend

**MCP (Model Context Protocol) Implementation:**
- Standardized context sharing between AI systems
- Resource discovery and context management
- Tool execution framework for AI agents
- Security and permission management

### Key Integration Points

#### Database Schema (PostgreSQL)
- **Users & Authentication**: User accounts, roles, JWT sessions
- **Social Connections**: Platform integrations, tokens, sync logs
- **Data Storage**: External data cached locally with metadata
- **API Management**: Tokens, webhooks, rate limiting

#### Service Communication
- **HTTP APIs**: RESTful endpoints for service-to-service communication
- **Shared Database**: Central PostgreSQL instance for persistent state
- **Event-driven Updates**: Background tasks for data synchronization
- **Health Monitoring**: Comprehensive health checks across all services

### Performance & Scalability

#### Backend Performance (Rust)
- **Async/Await**: Full async implementation with Tokio runtime
- **Connection Pooling**: SQLx connection pooling for database efficiency
- **Metrics Collection**: Prometheus-compatible metrics for monitoring
- **Structured Logging**: Advanced tracing with performance monitoring

#### AI Context Optimization
- **Intelligent Caching**: Context and search result caching
- **Relevance Scoring**: AI-driven context prioritization
- **Resource Management**: Memory-efficient vector operations
- **Batch Processing**: Efficient bulk operations for indexing

#### Frontend Performance
- **Code Splitting**: Next.js automatic code splitting
- **Component Optimization**: React optimization patterns
- **API Caching**: Smart caching of API responses
- **Progressive Loading**: Incremental data loading for large datasets

## Important Development Patterns

### Feature Toggle System
ConHub uses a feature toggle system located in `feature-toggles.json`:
- **Login Toggle**: Controls authentication system (set to `false` for development)
- Runtime configuration changes without restarts
- Mock data provided when features are disabled

### Error Handling Strategy
- **Structured Errors**: Consistent error formatting across services
- **Logging Integration**: Comprehensive error logging with context
- **User-Friendly Messages**: Frontend error boundaries with helpful messages
- **Recovery Mechanisms**: Automatic retry logic for transient failures

### Authentication Architecture
- **JWT-based**: Secure token authentication with configurable expiration
- **Social OAuth**: Integration with GitHub, Google, and other providers
- **Session Management**: Server-side session tracking with cleanup tasks
- **Permission System**: Role-based access control (admin, user, moderator)

### AI Integration Patterns
- **Context Building**: Structured context from multiple data sources
- **Tool Framework**: Extensible tool system for AI agents
- **MCP Protocol**: Standardized AI context sharing implementation
- **Performance Monitoring**: AI operation timing and success tracking

## Platform-Specific Notes

### Windows Development
- PowerShell scripts (`.ps1`) for service management
- Execution policy bypass for development scripts
- Windows-specific paths in npm scripts

### Linux/macOS Development  
- Bash scripts (`.sh`) for service management
- Standard Unix conventions for file paths
- Cross-platform compatibility considerations

## Environment Configuration

Key environment variables for development:
```bash
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/conhub

# JWT Authentication
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production

# AI Services
OPENAI_API_KEY=your_openai_key
GITHUB_ACCESS_TOKEN=your_github_token

# Logging
LOG_LEVEL=info
RUST_LOG=info
ENABLE_PERFORMANCE_MONITORING=true
```

## Common Troubleshooting

### Service Startup Issues
- Check port availability (3000-3003, 8001)
- Verify PostgreSQL is running and accessible
- Ensure all dependencies are installed (`npm install`, `pip install -r requirements.txt`)

### Database Connection Issues
- Run database setup scripts before first start
- Verify DATABASE_URL environment variable
- Check PostgreSQL service status

### Build Issues
- Ensure Rust toolchain is up to date
- Clear build caches: `cargo clean` for Rust services
- Verify Node.js version compatibility (18+)

## Development Workflow

1. **Environment Setup**: Run database setup scripts, configure `.env`
2. **Service Startup**: Use `npm start` for all services
3. **Development**: Services auto-reload on file changes
4. **Testing**: Use service-specific test commands
5. **Production Build**: `npm run build` for optimized builds

<citations>
  <document>
      <document_type>WARP_DOCUMENTATION</document_type>
      <document_id>getting-started/quickstart-guide/coding-in-warp</document_id>
  </document>
</citations>
