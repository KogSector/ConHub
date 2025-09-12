# ConHub LangChain Service

A Node.js service that provides LangChain-powered data source indexing and AI search capabilities for the ConHub platform.

## Features

- **Multi-source data indexing**: Connect and index content from GitHub, Google Drive, Notion, web pages, and more
- **Vector search**: Semantic search across all indexed content using embeddings
- **AI-powered Q&A**: Ask questions about your connected data sources
- **Real-time sync**: Keep your indexed content up-to-date with automatic syncing
- **RESTful API**: Easy integration with the frontend and other services

## Architecture

This service runs alongside the Rust backend and provides:
- Data source connectors (GitHub, Google Drive, Notion, etc.)
- Document processing and chunking
- Vector embeddings generation
- Semantic search capabilities
- API endpoints for the frontend

## Quick Start

### Prerequisites

- Node.js 18+ and npm
- OpenAI API key (for embeddings)
- Vector database (Qdrant or Pinecone)

### Installation

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Set up environment variables:**
   ```bash
   cp .env.example .env
   ```
   
   Edit `.env` and add your API keys:
   ```bash
   OPENAI_API_KEY=your_openai_api_key_here
   QDRANT_URL=http://localhost:6333  # or Pinecone config
   # ... other config
   ```

3. **Start the service:**
   ```bash
   npm run dev
   ```

The service will start on `http://localhost:3001`

## API Endpoints

### Data Sources
- `GET /api/data-sources` - List connected data sources
- `POST /api/data-sources/connect` - Connect a new data source
- `DELETE /api/data-sources/:id` - Disconnect a data source
- `POST /api/data-sources/:id/sync` - Sync/re-index a data source

### Indexing
- `POST /api/indexing/repository` - Index a GitHub repository
- `POST /api/indexing/document` - Index a document or file
- `GET /api/indexing/status/:id` - Get indexing status

### Search
- `POST /api/search/universal` - Search across all indexed content
- `POST /api/search/code` - Search specifically in code repositories
- `POST /api/search/documents` - Search in documents and files

## Example Usage

### Connect a GitHub Repository
```bash
curl -X POST http://localhost:3001/api/data-sources/connect \\
  -H "Content-Type: application/json" \\
  -d '{
    "type": "github",
    "credentials": {
      "accessToken": "your_github_token",
      "username": "your_username"
    },
    "config": {
      "repositories": ["owner/repo-name"]
    }
  }'
```

### Index a Repository
```bash
curl -X POST http://localhost:3001/api/indexing/repository \\
  -H "Content-Type: application/json" \\
  -d '{
    "repoUrl": "https://github.com/owner/repo-name",
    "accessToken": "your_github_token"
  }'
```

### Search Content
```bash
curl -X POST http://localhost:3001/api/search/universal \\
  -H "Content-Type: application/json" \\
  -d '{
    "query": "how to implement authentication",
    "limit": 10
  }'
```

## Supported Data Sources

- **GitHub**: Public and private repositories
- **Google Drive**: Documents, spreadsheets, presentations
- **Notion**: Pages and databases
- **Web Crawler**: Any public web pages
- **Local Files**: Upload and index local documents

## Vector Databases

Choose one of the supported vector databases:

### Qdrant (Recommended for self-hosting)
```bash
# Run Qdrant with Docker
docker run -p 6333:6333 qdrant/qdrant
```

### Pinecone (Managed service)
- Sign up at [Pinecone](https://pinecone.io)
- Create an index
- Add your API key to `.env`

## Development

### Project Structure
```
src/
├── index.ts              # Main application entry
├── routes/               # API route handlers
│   ├── indexing.ts
│   ├── search.ts
│   └── dataSources.ts
├── services/             # Business logic
│   ├── indexingService.ts
│   ├── searchService.ts
│   ├── dataSourceService.ts
│   └── vectorStore.ts
├── middleware/           # Express middleware
└── utils/               # Utilities and helpers
```

### Scripts
- `npm run dev` - Start development server with hot reload
- `npm run build` - Build TypeScript to JavaScript
- `npm start` - Start production server
- `npm run lint` - Run ESLint
- `npm test` - Run tests

## Integration with ConHub

This service is designed to work with the main ConHub application:

1. **Frontend Integration**: Your React frontend calls these APIs to manage data sources and perform searches
2. **Rust Backend**: Can proxy requests to this service or call it directly
3. **OpenGrok**: This service can work alongside OpenGrok for code-specific features

## Configuration

Key environment variables:

```bash
# Service Configuration
NODE_ENV=development
PORT=3001

# AI/LLM Configuration
OPENAI_API_KEY=your_key_here

# Vector Database (choose one)
QDRANT_URL=http://localhost:6333
PINECONE_API_KEY=your_key_here
PINECONE_INDEX_NAME=conhub-index

# Data Source APIs
GITHUB_ACCESS_TOKEN=your_token_here
GOOGLE_DRIVE_CLIENT_ID=your_id_here
NOTION_API_KEY=your_key_here

# External Services
OPENGROK_URL=http://localhost:8080
RUST_BACKEND_URL=http://localhost:8000
```

## Production Deployment

For production deployment:

1. **Build the service:**
   ```bash
   npm run build
   ```

2. **Use a process manager:**
   ```bash
   pm2 start dist/index.js --name conhub-langchain
   ```

3. **Set up reverse proxy** (nginx/Apache) for HTTPS

4. **Monitor and scale** as needed

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details
