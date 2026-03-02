# Drishti Docs

Published URL: <https://singh-sumit.github.io/drishti/>

This site is built with Docusaurus (TypeScript) and includes Mermaid diagram support.
Search strategy for v0.4 is dependency-light: sidebar navigation plus browser search on page content (no external search service).

## Local Development

```bash
npm ci
npm run start
```

With `baseUrl=/drishti/`, local preview routes are served under:

- `http://localhost:3000/drishti/` (`npm run serve`)

## Production Build

```bash
npm ci
npm run build
```

The build fails on broken links and broken markdown links.

## Deployment

Deployment is handled by `.github/workflows/docs.yml`:

- pull requests run docs build validation
- pushes to `main` deploy GitHub Pages via `actions/deploy-pages`
