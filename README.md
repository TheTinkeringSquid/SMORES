# S.M.O.R.E.S.

**Smart Management of Onboard Resources, Electronics, and Systems** — a modular,
**fully open-source, offline-first** smart hub for RVs and vans.

Today an RVer juggles a half-dozen disconnected apps: the tire-pressure monitor,
tank levels, HVAC, solar/power, and (in a motorhome) powertrain each live in
their own pane of glass and don't talk to each other. S.M.O.R.E.S. unifies them.
Distributed nodes normalize each system into one canonical telemetry format,
publish it to a local MQTT bus, and a single dashboard shows everything at once —
no cloud required.

```
RV subsystems (TPMS, tanks, solar, HVAC, powertrain, ...)
   → ESP32 / STM32 nodes & adapters  (normalize to canonical envelope)
   → MQTT bus (Mosquitto)
   → Rust / Axum backend  (state store · REST · SSE)
   → Unified dashboard (React) — one pane of glass
```

## Repository layout

```
backend/     Rust / Axum API + MQTT subscriber + state store
frontend/    React / Vite / TypeScript dashboard
docs/        SCHEMAS.md (the data contract) · RV_SYSTEMS.md (subsystem catalog)
.devcontainer/  Dev Container: Node, Rust, Mosquitto MQTT broker
Cargo.toml   Rust workspace
```

## Quick start

Runs inside a VS Code Dev Container so your host stays clean (Node, Rust, and the
MQTT broker run in containers).

1. Install **VS Code**, **Docker Desktop**, and the **Dev Containers** extension.
2. Open this folder → Command Palette → **Dev Containers: Reopen in Container**.
3. In the container terminal:
   - **Backend:** `cd backend && cargo run` → http://localhost:8080
   - **Frontend:** `cd frontend && npm i && npm run dev` → http://localhost:5173
4. The dashboard fetches mock telemetry from `http://localhost:8080/api/v1/...`.

MQTT broker runs as service `mqtt` on port `1883`. VS Code tasks `backend:run`,
`frontend:dev`, and `start:all` are available.

## Documentation

- **[docs/SCHEMAS.md](docs/SCHEMAS.md)** — the canonical MQTT envelope and every
  subsystem schema. This is the source of truth all components derive from.
- **[docs/RV_SYSTEMS.md](docs/RV_SYSTEMS.md)** — catalog of real RV systems, the
  data/controls each exposes, open integration paths (RV-C, Victron, Modbus,
  BLE), and the canonical `subsystem` registry.
- **[CONTRIBUTING.md](CONTRIBUTING.md)** — dev setup, workflow, and safety policy.

## Status

Early development. Milestone 1 flows mock telemetry through the real
architecture (MQTT → typed backend state store → REST → dashboard cards) for
battery, tanks, and TPMS — provable end-to-end with **no hardware required**.

## License

[MIT](LICENSE) — open source, no cloud dependencies for core function.
