# ConHub

Unify your repositories, docs, and URLs with AI for better development workflows.

## Quick Start

### Prerequisites
- **Node.js** (v18 or higher)
- **Rust** and **Cargo**
- **Git**

### Installation & Setup

1. **Clone the repository:**
   ```bash
   git clone <your-repo-url>
   cd ConHub
   ```

2. **Install dependencies:**
   ```bash
   # Install frontend dependencies
   cd frontend && npm install && cd ..
   
   # Install workspace dependencies (for running both services)
   npm install
   ```

3. **Configure Auth0:**
   - Copy `frontend/.env.example` to `frontend/.env.local`
   - Update the Auth0 configuration with your credentials

### Development

**🚀 Start ConHub (One Command)**

```bash
npm start
```

This automatically starts all services:
- ✅ **Frontend** on port 3000
- ✅ **Backend API** on port 3001
- ✅ **Lexor Service** on port 3002
- ✅ **Auto-reload** on file changes
- ✅ **Cross-platform** (Windows, Mac, Linux)

**Individual Services** (if needed):
```bash
# Backend only
npm run dev:backend

# Frontend only  
npm run dev:frontend

# Lexor service only
npm run dev:lexor
```

### Services

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:3001
- **Lexor Service**: http://localhost:3002

### Port Configuration

The services are pre-configured to use specific ports:
- **Frontend (Next.js)**: Port 3000
- **Backend (Rust/Actix)**: Port 3001
- **Lexor (Rust/Actix)**: Port 3002

No manual port management needed - everything runs on its designated port automatically!

### Auth0 Setup

1. Create an Auth0 account at https://auth0.com
2. Create a new application (Single Page Application)
3. Configure your application settings:
   - Allowed Callback URLs: `http://localhost:3000`
   - Allowed Logout URLs: `http://localhost:3000`
   - Allowed Web Origins: `http://localhost:3000`
4. Update `frontend/.env.local` with your Auth0 credentials

### Available Scripts

- `npm start` - Start the complete ConHub application (frontend + backend + lexor)
- `npm run dev` - Same as start (alias for development)
- `npm run dev:frontend` - Start only frontend on port 3000
- `npm run dev:backend` - Start only backend on port 3001
- `npm run dev:lexor` - Start only lexor service on port 3002
- `npm run build` - Build all services for production
- `npm run install:all` - Install all dependencies

## Project Structure

```
ConHub/
├── frontend/          # Next.js frontend (Port 3000)
├── backend/           # Rust backend (Port 3001)
├── lexor/             # Rust lexor service (Port 3002)
├── haystack-service/  # Python AI service
├── langchain-service/ # TypeScript AI service
└── package.json       # Workspace configuration
```

## Tech Stack

- **Frontend**: Next.js 14, React 18, TypeScript, Tailwind CSS, Auth0
- **Backend**: Rust, Actix Web
- **Lexor**: Rust, Actix Web, Tantivy (code indexing and search)
- **Authentication**: Auth0
- **Styling**: Tailwind CSS, shadcn/ui components

A full-stack application that connects multiple knowledge sources (repositories, documents, URLs) with AI agents for enhanced development context.

## Project Structure

- `frontend/` - Next.js application
- `backend/` - Rust Actix web server

## Development

### Prerequisites
- Node.js (v18 or higher)
- Rust (latest stable)

### Setup

1. **Backend (Rust Actix)**
```bash
cd backend
cargo run
```
Backend will run on http://localhost:3001

2. **Frontend (Next.js)**
```bash
cd frontend
npm install
npm run dev
```
Frontend will run on http://localhost:3000

### API Testing
Visit http://localhost:3000/test to test the connection between frontend and backend.

## Technologies

- **Frontend**: Next.js 14, React 18, TypeScript, Tailwind CSS, shadcn/ui
- **Backend**: Rust, Actix Web
- **Communication**: REST API with CORS enabled

## Features

- ✅ Multi-source connectivity (Git repos, docs, URLs)
- ✅ AI agent integration
- ✅ RAG (Retrieval-Augmented Generation) architecture
- ✅ Next.js App Router with TypeScript
- ✅ Tailwind CSS styling with shadcn/ui
- ✅ Rust Actix backend
- ✅ Secure authentication with Auth0
- ✅ Real-time sync and indexing
- ✅ Context-aware AI responses

## Ports

- Frontend: 3000
- Backend: 3001
- Lexor: 3002
- Haystack Service: 8000
- LangChain Service: 8001