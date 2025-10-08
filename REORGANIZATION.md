# ConHub Codebase Reorganization

## ✅ Completed Changes

### 1. **Indexing Services Consolidation**
- Created `indexers/` folder at root level
- Moved `lexor/` → `indexers/lexor/`
- Moved `doc-search/` → `indexers/doc-search/`
- Moved `langchain-service/` → `indexers/langchain-service/`
- Moved `lexor_data/` → `indexers/lexor_data/`

### 2. **Scripts Organization**
- Created subfolders under `scripts/`:
  - `scripts/services/` - Service management scripts
  - `scripts/maintenance/` - Cleanup and maintenance scripts
  - `scripts/deployment/` - Deployment scripts (empty, ready for future use)
- Moved scripts to appropriate subfolders:
  - `start.ps1`, `stop.ps1`, `status.ps1`, `run-backend.ps1`, `run-lexor.ps1` → `services/`
  - `cleanup-ports.ps1`, `force-stop.ps1` → `maintenance/`

### 3. **Configuration Files Cleanup**
- **Removed duplicate `tsconfig.json`** from frontend folder
- **Updated root `tsconfig.json`** to include new indexers paths
- **Fixed `Cargo.toml`** to point to new lexor path
- **Updated `docker-compose.yml`** to reflect new folder structure
- **Fixed `tsconfig.langchain.json`** paths

### 4. **Port Conflicts Resolution**
- **Fixed LangChain service port** from conflicting 3002 to 8002
- **Updated environment variables** and logging
- **Fixed port configuration** in logger.ts

### 5. **Frontend Configuration Fix**
- **Updated `next.config.js`** to properly work with frontend directory
- **Fixed `package.json` scripts** to run frontend from correct directory
- **Updated dev:frontend script** to `cd frontend && next dev -p 3000`

## 🎯 Current Service Architecture

```
ConHub/
├── frontend/           (Port 3000) - Next.js UI
├── backend/           (Port 3001) - Rust API
├── indexers/
│   ├── lexor/         (Port 3002) - Code indexing
│   ├── doc-search/    (Port 8001) - Document search
│   └── langchain-service/ (Port 8002) - AI operations
├── scripts/
│   ├── services/      - Service management
│   ├── maintenance/   - Cleanup scripts
│   └── deployment/    - Future deployment scripts
└── [shared configs at root]
```

## 🔧 Fixed Issues

1. **✅ npm start not working** - Fixed script paths and frontend configuration
2. **✅ Port conflicts** - LangChain now uses 8002 instead of 3002
3. **✅ Frontend can't find app directory** - Fixed Next.js configuration
4. **✅ Duplicate tsconfig.json** - Removed duplicate, using root only
5. **✅ Shared files organization** - All shared configs now at root level

## 🚀 Ready to Test

The codebase is now properly organized and should work with:
```bash
npm start
```

All services should start on their designated ports without conflicts.