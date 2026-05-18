# Ballista

Tauri v2 desktop app for launching and managing integration engine administrator instances. Rust backend + Nuxt 4/Vue 3 frontend. Originally forked from [kayyagari/ballista](https://github.com/kayyagari/ballista).

## Build

```bash
npm install          # frontend dependencies
npm run tauri build  # full build (frontend + Rust + .app/.dmg bundle)
```

Rust-only check/test (from repo root):
```bash
cargo check
cargo test
```

## Project Structure

- `src-tauri/src/` — Rust backend (Tauri commands, pinned peer trust, webstart/JNLP handling)
  - `main.rs` — Tauri command handlers and app setup
  - `connection.rs` — ConnectionStore, connection persistence, cert trust management
  - `webstart.rs` — JNLP parsing, jar downloading, Java process launching
  - `errors.rs` — LaunchError type and peer trust response formatting
- `app/` — Nuxt 4 frontend (pages, components, composables, types)
- `src-tauri/lib/` — Bundled Java console jar

## Conventions

- Tauri commands use `rename_all = "snake_case"` — JS side must use snake_case parameter names
- Self-signed certs are expected — Ballista accepts them after user pinning
- Rust error handling: prefer `?` operator and `ok_or_else` over `.unwrap()` — return errors to frontend, don't panic
- Mutex locks: use `.expect("descriptive message")` since poisoning is unrecoverable
- Frontend uses Tailwind CSS v4 with `@theme` design tokens in `app/assets/css/main.css`
- Icons: Phosphor Icons via `ph:` prefix

## Remotes

- `origin` — upstream `kayyagari/ballista`
- `mine` — fork `pacmano1/launcher`
