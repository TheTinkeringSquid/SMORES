# Contributing to S.M.O.R.E.S.

S.M.O.R.E.S. (Smart Management of Onboard Resources, Electronics, and Systems) is
an open-source, offline-first smart hub for RVs and vans. Contributions are
welcome — firmware nodes, backend, dashboard, docs, and integration adapters.

## The one rule that matters most

**The schema is the contract.** [`docs/SCHEMAS.md`](docs/SCHEMAS.md) defines the
canonical MQTT envelope and every subsystem payload. Backend models, dashboard
types, and node firmware all derive from it. If you need a new field or
subsystem, **change `docs/SCHEMAS.md` first**, then the code. See
[`docs/RV_SYSTEMS.md`](docs/RV_SYSTEMS.md) for the catalog of RV systems and the
canonical `subsystem` ids.

## Dev environment

Everything runs inside a VS Code Dev Container (Node, Rust, and a Mosquitto MQTT
broker), so your host stays clean.

1. Install **VS Code**, **Docker Desktop**, and the **Dev Containers** extension.
2. Open the repo → Command Palette → **Dev Containers: Reopen in Container**.
3. In the container:
   - Backend: `cd backend && cargo run` (serves on `:8080`)
   - Frontend: `cd frontend && npm i && npm run dev` (serves on `:5173`)
   - MQTT broker is the `mqtt` service on `:1883`.

VS Code tasks `backend:run`, `frontend:dev`, and `start:all` are provided.

## Workflow

- Branch from `main`; use prefixes `feat/`, `fix/`, `docs/`, `chore/`.
- Keep changes incremental and runnable; keep **mock mode working** so the stack
  is usable without hardware.
- Open a PR. CI runs backend build/test (+ advisory fmt/clippy) and frontend
  typecheck/build.
- Before pushing: `cargo build --workspace && (cd frontend && npm run typecheck)`.

## Safety

Vehicle/RV control features are safety-sensitive. Anything that could create an
unsafe state — ignition, locks while driving, propane, brakes, road lighting,
high-current relays, inverter/shore/generator transfer — must be telemetry-only
until a design review signs off, and must be explicit, validated, logged, and
fail-safe. See the safety notes in `docs/RV_SYSTEMS.md`.

## License

By contributing you agree your contributions are licensed under the repository's
[MIT License](LICENSE).
