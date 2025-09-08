# ConHub

A full-stack application with Next.js frontend and Rust Actix backend.

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

- ✅ Next.js App Router
- ✅ TypeScript support
- ✅ Tailwind CSS styling
- ✅ shadcn/ui components
- ✅ Rust Actix backend
- ✅ CORS configuration
- ✅ Environment variables
- ✅ API client utilities

## Ports

- Frontend: 3000
- Backend: 3001