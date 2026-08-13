# Podmixer — Agent Instructions

## Project overview

Podmixer is a podcast RSS feed aggregator. It polls configured podcast RSS feeds, collects new episodes, generates aggregated `short.xml` / `long.xml` feeds, and optionally publishes new episodes to Telegram and Twitter (X). It has a web UI for managing podcasts and configuration.

## Repository structure

```
back/          Rust backend (Axum 0.8, SQLite via sqlx 0.8)
front/         React 19 + TypeScript + Vite 6 + MUI v7
compose.yml    Production deployment via Docker Compose
Dockerfile     Multi-stage build (node:22 → rust:alpine → alpine:3.22)
.justfile      Task runner (dev, front, back, build, push)
```

## Key architecture facts

- **Backend entrypoint**: `back/src/main.rs` — sets up Axum router, runs SQLite migrations at startup, spawns a background worker loop that polls all podcasts every `SLEEP_TIME` seconds.
- **API prefix**: all routes under `/api/v1/`:
  - `GET /api/v1/health/`
  - `POST/GET /api/v1/auth/login`, `/auth/logout`, `/auth/register`
  - `GET/POST/PATCH/DELETE /api/v1/podcasts/`
  - `POST /api/v1/podcasts/generate` — regenerate RSS feeds on demand
  - `GET/POST /api/v1/config/feed`, `/config/twitter`, `/config/telegram`
- **Static SPA**: built frontend lives in `back/static/`. The Axum server serves it as a fallback (`ServeFile::new("static/index.html")`), so the SPA handles its own routing.
- **RSS feeds served at**: `/rss/short.xml` and `/rss/long.xml` (static files written to `back/rss/`).
- **Database**: SQLite via sqlx. Migrations in `back/migrations/` run automatically on startup. Tables: `users`, `podcasts`, `config` (key-value store for feed metadata, Telegram, Twitter settings).
- **Auth**: JWT-based. Token stored in `localStorage` on the frontend. 60-minute expiry. Auto-logout on expiry via `setTimeout`.
- **Frontend routing**: `react-router` v7 with three layouts — `MainLayout` (public), `AuthLayout` (login), `ProtectedLayout` (requires auth).
- **Frontend API base URL**: from `VITE_BASE_URL` env var. Defaults to `http://localhost:3000` in development, empty in production (same origin).

## Commands

| Command | What it does |
|---|---|
| `just dev` | Builds frontend → copies to `back/static/` → runs backend with `RUST_LOG=debug` |
| `just front` | Runs Vite dev server (HMR at default Vite port) |
| `just back` | Runs `cargo run` with `RUST_LOG=debug` |
| `just build` | Docker build with `atareao/podmixer` tags |
| `just push` | Docker push all tags |
| `cd front && pnpm run build` | TypeScript check + Vite production build |
| `cd front && pnpm run lint` | ESLint check |
| `cd back && cargo test` | Run inline unit tests (truncate + date parsing) |

## Environment variables (backend)

| Variable | Default | Notes |
|---|---|---|
| `RUST_LOG` | `debug` | Tracing/log level |
| `DB_URL` | `podmixer.db` | SQLite file path |
| `PORT` | `3000` | Server port |
| `SECRET` | `esto-es-un-secreto` | JWT signing secret |
| `SLEEP_TIME` | `900` | Background poll interval (seconds) |
| `OLDER_THAN` | `30` | Days threshold for short vs long feed |
| `RUST_ENV` | — | Set to `production` in Docker; controls migration path resolution |
| `PUBLISHER_DRY_RUN` | `false` | Cuando es `true`, los publishers simulan las publicaciones sin enviar realmente a redes sociales. Se registra un log con estado "dry-run". Por defecto activo en `just dev` y `just back`.

## Development gotchas

- **Frontend must be built before running backend** (or use `just dev` which does both). The backend serves the SPA from `back/static/`.
- **`back/.gitignore` ignores `Cargo.lock`** — this is a mistake for an executable binary. The lockfile is committed in practice (it exists in the repo).
- **No rustfmt or clippy config** exists. Defaults apply.
- **No CI workflows** exist.
- **Tests are inline** in `back/src/main.rs` (not in a `tests/` directory). Run with `cargo test` from `back/`.
- **Database file** (`podmixer.db*`) is gitignored in `back/.gitignore` but not in the root `.gitignore`.
- **`back/rss/`** is gitignored at the root level — RSS output files are runtime artifacts.
- **`back/static/`** is gitignored — it's the built frontend output.
- **Docker**: uses `openssl` vendored feature. The `rust:alpine3.22` builder installs several dev packages for OpenSSL. The final image is `alpine:3.22` with `curl` and `sqlite`.
- **`compose.yml`** mounts two volumes: `db` (SQLite persistence) and `rss` (generated feed files). Port mapping: `4000:3000`.
- **Frontend dev server** does not proxy to the backend. In development, the frontend runs on its own port and calls the backend via `VITE_BASE_URL=http://localhost:3000`.

## Style / conventions

- Backend uses `ApiResponse` as a uniform response wrapper with `status`, `message`, and `data` fields.
- Frontend uses class components (not hooks) for stateful components like `AuthContextProvider` and `App`.
- Frontend uses `console.log` extensively for debugging — no dedicated logger.
- MUI components with `sx` prop for styling; Emotion `styled` also used.
- Dark mode is the default (and only) theme.