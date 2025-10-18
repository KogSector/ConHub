# ConHub Repository Guidelines

## Architecture Overview
- **Frontend**: Next.js 14 + TypeScript located in `frontend/`
- **Backend**: Rust (Actix-web) located in `backend/`
- **Unified Indexer**: Rust service in `indexers/`
- **MCP Service**: Node.js service in `mcp/`

## Coding Conventions
### TypeScript / JavaScript
1. **Style**: Follow ESLint and Prettier defaults in the repo.
2. **Imports**: Use `@` alias for `src/frontend` in Next.js.
3. **State Management**: Prefer React hooks + context; avoid external state libraries unless necessary.
4. **Testing**: Use existing testing setup; keep tests alongside code when possible.
5. **Code Organization**: Organize files by feature, not type. We maintain a single package.json, package-lock.json, Cargo.toml, Cargo.lock, .env, requirements.txt, etc. and other config files at root level shared by any file/folder that requires these.
6. **Type Safety**: Utilize TypeScript's features like interfaces and generics wherever applicable.
7. **Naming**: Be descriptive but concise. Avoid abbreviations unless widely accepted.
8. **Comments**: Add comments explaining complex logic or non-obvious decisions.
9. **Performance**: Optimize performance-critical sections using profiling tools.
10. **Security**: Follow OWASP guidelines for web security best practices.
11. **Accessibility**: Follow WCAG guidelines for accessible design.
12. **Documentation**: Write clear JSDoc comments for functions/classes.
13. **Version Control**: Commit often with meaningful messages.
14. **Git Flow**: Use Git flow branching strategy.
15. **Optimizations**: All code written should be optimized, type strict and must follow best practices. These include some practises like lazy-loading, memoization, caching, lazy-evaluation,etc. Use Data Structures heavily if needed and use them wewisely, no matter how complex it is. Optimal code is the requirement.
16. **Modularity**: Break down large components into smaller reusable parts.
17. **DRY Principle**: Don't Repeat Yourself - use helper functions/methods as needed.
18. **Consistency**: Maintain consistent coding style throughout the project.
19. **Avoid Global State**: Wherever possible, prefer local component state over global stores.
20. **Error Handling**: Gracefully handle errors without crashing the app.

### Rust Services
1. **Formatting**: Run `cargo fmt` before committing.
2. **Linting**: Address `cargo clippy` warnings where possible.
3. **Error Handling**: Use `anyhow`/`thiserror` patterns per existing code.
4. **Async**: Use `tokio` runtime for asynchronous tasks.

### Documentation
- Update `README.md` or service-specific docs when modifying public behavior.
- Document new environment variables in `.env.example` and `README.md`.

### Infrastructure
- **Docker**: Each service has its own Dockerfile. Ensure multi-stage builds when adding dependencies.
- **Env Files**: Keep secrets out of version control.

## Branching & Pull Requests
1. Create feature branches for changes.
2. Keep PRs scoped and documented with summary + testing steps.

## Testing & CI
- Run unit/integration tests relevant to changes.
- Ensure linting and formatting pass locally before pushing.

## Commits
- Use conventional commit messages where possible (e.g., `feat:`, `fix:`, `docs:`).

## Review Checklist
- [ ] Code formatted
- [ ] Tests added/updated
- [ ] Docs updated if behavior changes
- [ ] No sensitive data included