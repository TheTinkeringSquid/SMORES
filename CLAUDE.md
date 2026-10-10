# S.M.O.R.E.S. — Agent Instructions (Claude + Codex)
<!-- CLAUDE.md and AGENTS.md are identical; edit both together. Router, not a diary: keep under ~100 lines. -->

## What this is
**Smart Management of Onboard Resources, Electronics, and Systems**: an open-source (MIT), **offline-first** smart hub for RVs and vans.
Pipeline: subsystem nodes (ESP32/STM32) → canonical MQTT envelope → Mosquitto → Rust/Axum backend (state store, REST, SSE) → React/Vite/TS dashboard.
Status: early development. Milestone 1 runs mock telemetry through the real architecture.

## Where things live
| Need | Go to |
|---|---|
| **Data contract (source of truth for every component)** | `docs/SCHEMAS.md` |
| Subsystem catalog, integration paths, safety notes | `docs/RV_SYSTEMS.md` |
| REST endpoints and alert engine | `docs/API.md` |
| Node registry and alert thresholds | `smores.toml` |
| Dev setup, workflow, safety policy | `CONTRIBUTING.md` |
| Backend | `backend/` (Rust workspace via root `Cargo.toml`) |
| Frontend | `frontend/` |
| Dev environment (Node, Rust, Mosquitto) | `.devcontainer/`, VS Code tasks in `.vscode/tasks.json` |

## Commands (inside the Dev Container)
- Backend: `cd backend && cargo run` → http://localhost:8080
- Frontend: `cd frontend && npm i && npm run dev` → http://localhost:5173
- MQTT broker: service `mqtt`, port 1883
- **Before pushing:** `cargo build --workspace && (cd frontend && npm run typecheck)`. CI also runs backend tests and advisory fmt/clippy.

## Rules
- **Schema first.** Change `docs/SCHEMAS.md` before code that changes the envelope or a subsystem payload. Backend and frontend types derive from it.
- **Keep mock mode working**, so the stack runs with no hardware.
- **Safety:** anything that can create an unsafe state (ignition, locks while driving, propane, brakes, road lighting, high-current relays, inverter/shore/generator transfer) stays **telemetry-only** until a design review signs off. Control paths must be explicit, validated, logged, and fail-safe. Get a `plan-roaster` review before proposing any control feature.
- Offline-first: no feature may require a cloud service.
- Branch from `main` with `feat/`, `fix/`, `docs/`, `chore/`. Small, runnable increments; open a PR. Commit or push only when Nathan says so.
- Prove it: run the build, typecheck, or tests and say what you ran.
- `frontend/.env.local` holds local settings; never commit secrets.

## Applied learning
<!-- One-line bullets added when something fails repeatedly. Prune often. -->
-
