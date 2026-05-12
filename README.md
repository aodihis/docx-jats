# docx-jast

Convert Microsoft Word (`.docx`) research manuscripts into JATS XML.

## Requirements

| Tool | Version | Install |
|------|---------|---------|
| Rust + Cargo | 1.75+ | https://rustup.rs |
| Node.js | 20+ | https://nodejs.org |
| npm | 10+ | bundled with Node.js |

## Project structure

```
docx-jast/
├── backend/   # Rust / Axum API server (port 3001)
└── frontend/  # SvelteKit UI (port 5173)
```

## Installation

### 1. Clone the repository

```bash
git clone <your-repo-url>
cd docx-jast
```

### 2. Backend

```bash
cd backend
cargo build
```

### 3. Frontend

```bash
cd frontend
npm install
```

## Running in development

Open two terminals.

**Terminal 1 — backend**

```bash
cd backend
cargo run
```

The API will be available at `http://localhost:3001`.

**Terminal 2 — frontend**

```bash
cd frontend
npm run dev
```

Open `http://localhost:5173` in your browser.

The Vite dev server proxies `/api/*` to `http://localhost:3001`, so no CORS configuration is needed in development.

## Running tests

**Unit tests (backend)**

```bash
cd backend
cargo test --lib
```

**Integration tests (backend)**

```bash
cd backend
cargo test --test '*'
```

**All backend tests**

```bash
cd backend
cargo test
```

**Frontend type checking**

```bash
cd frontend
npm run check
```

## API

### `GET /health`

Returns `200 ok` when the server is running.

### `POST /convert`

Upload a `.docx` file and receive JATS XML.

**Request** — multipart/form-data with a field named `file`.

```bash
curl -X POST http://localhost:3001/convert \
  -F "file=@your-manuscript.docx"
```

**Response**

```json
{
  "success": true,
  "xml": "<article>...</article>",
  "warnings": ["No abstract found"],
  "metadata": {
    "title": "A Study on Testing",
    "author_count": 0,
    "section_count": 3,
    "reference_count": 12,
    "has_abstract": true
  }
}
```

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `docx_jast_backend=debug,tower_http=info` | Backend log level (tracing directives) |
| `LOG_FORMAT` | `pretty` | Log format: `pretty` (colored, dev) or `json` (structured, production) |
| `VITE_API_BASE` | `http://localhost:3001` | Backend URL (full, not proxied — SvelteKit intercepts `/api/*` before Vite proxy) |

Copy `.env.example` to `.env` in the `frontend/` directory if you need to override the API base URL.

```bash
cp frontend/.env.example frontend/.env
```

## What is supported

- IMRAD-style English manuscripts
- Title, abstract, authors
- Headings (H1 / H2 / H3) → JATS `<sec>`
- Plain body paragraphs → JATS `<p>`
- References section → JATS `<ref-list>`

## What is not supported (yet)

- Equations, figures, images
- Complex citations
- Multi-column layouts
- Footnotes
- Authentication / multi-user
