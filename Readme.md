```markdown
# Employee Search – HTMX + Axum + SQLite

A lightweight, hypermedia-driven employee search built with Rust and HTMX, including a command-line tool for fast lookups.

## ✨ Features

- **Lightning-fast search** by first or last name (prefix search, server‑side)
- **Detail view** for each employee (department, current title, current salary)
- **Clean frontend** with zero custom JavaScript – only HTMX and Pico.css
- **CLI tool** for the same search directly in the terminal
- **Robust** – includes a readiness endpoint and optimised SQLite connection

## 🧰 Tech Stack

- [Axum](https://github.com/tokio-rs/axum) – web framework in Rust
- [HTMX](https://htmx.org) – hypermedia interactions without writing JavaScript
- [SQLite](https://sqlite.org) – local, file-based database
- [Caddy](https://caddyserver.com) – reverse proxy & static file server
- [Pico.css](https://picocss.com) – minimal, semantic CSS
- [process‑compose](https://github.com/F1bonacc1/process-compose) – process runner for the Axum + Caddy pair

## 📂 Project structure

```
.
├── Caddyfile              # Reverse proxy and static files
├── Cargo.toml             # Rust workspace with two binaries
├── process-compose.yaml   # Process runner configuration
├── src/
│   ├── main.rs            # Axum server (binary: server)
│   └── bin/
│       └── search.rs      # CLI tool (binary: search)
├── index.html             # HTMX frontend for the web search
├── employees.db           # SQLite database (not in repo)
└── README.md
```

## 🚀 Quickstart

### Prerequisites

- Rust & Cargo (≥ 1.70)
- Caddy (≥ 2.6) – `brew install caddy` or similar
- [process‑compose](https://github.com/F1bonacc1/process-compose) – single binary, see releases
- The SQLite database `employees.db` with the [Employee Schema](https://dev.mysql.com/doc/employee/en/) (provide a dump or generate it yourself)

### Build

```bash
cargo build --release
```

Two binaries are produced:
- `target/release/server` – the web server
- `target/release/search` – the CLI tool

### Run

#### a) Web server + frontend (with process‑compose)

```bash
process-compose up
```

This starts Axum on port 3000 and Caddy on port 8080, with Axum’s readiness probe ensuring Caddy only serves once the database is reachable.

Open [http://localhost:8080](http://localhost:8080) and type a last name.

#### b) CLI tool

```bash
# From the project directory
cargo run --bin search -- "baru"

# Or use the built binary
./target/release/search "baru"
```

Optional: specify a different database path

```bash
./search "baru" --database /path/to/employees.db
```

**Example output:**

```
  Employee search for 'baru'
  ══════════════════════════════════════════════════
  ──────────────────────────────────────────────────
  ID      First name      Last name       
  ──────────────────────────────────────────────────
  10110   Sanjai          Luders         
  10425   Barun           Plesums        
  ──────────────────────────────────────────────────
  Total: 2 Hits
```

## ⚡ Performance

SQLite is optimised with the following PRAGMAs (applied automatically by both the server and CLI):

- `journal_mode=WAL` – concurrent reads and writes
- `cache_size=-20000` – 20 MB in‑memory cache
- `synchronous=NORMAL` – reduced write latency

Additionally, three indexes were created to speed up current department/title/salary lookups (see `sql/optimizations.sql` in the repo).

## 🔍 API endpoints (Axum)

| Route                      | Description                                    |
|----------------------------|------------------------------------------------|
| `POST /search`             | Search form (q) → HTML table rows              |
| `GET /employee/:id/detail` | Employee detail info as HTML                   |
| `GET /health`              | Readiness probe (JSON), only on port 3000      |

## 📄 License

MIT – feel free to use and adapt.
```
