# Lexor - Code Indexing and Search Service

A high-performance code indexing and search service built with Rust and Actix Web.

## Features

- **Source Code Indexing**: Fast indexing of source code repositories
- **Full-Text Search**: Advanced search capabilities across codebases
- **Symbol Cross-Referencing**: Find definitions, references, and relationships
- **Git History Analysis**: Track changes and evolution over time
- **Multi-Language Support**: Support for multiple programming languages
- **Performance Optimized**: Built for speed and scalability

## Quick Start

### Prerequisites
- Rust (latest stable)

### Development

```bash
cd lexor
cargo run
```

The service will start on http://localhost:3002

### API Endpoints

- `GET /` - Service information
- `GET /health` - Health check
- `POST /api/search` - Search code
- `GET /api/projects` - List projects
- `POST /api/projects` - Add project
- `POST /api/projects/{id}/index` - Index project
- `GET /api/stats` - Get statistics

## Configuration

The service uses default configuration. Custom configuration can be added through environment variables or config files.

## Tech Stack

- **Rust** - Systems programming language
- **Actix Web** - Web framework
- **Tantivy** - Full-text search engine
- **Tree-sitter** - Syntax highlighting and parsing
- **Git2** - Git integration