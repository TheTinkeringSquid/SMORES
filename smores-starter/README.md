# S.M.O.R.E.S. Starter (Dev Container + Backend + Frontend)

This starter keeps your host clean by running Node, Rust, and the MQTT broker **inside a VS Code Dev Container**.

## Quick Start
1. Install **VS Code**, **Docker Desktop**, and the **Dev Containers** extension.
2. Open this folder in VS Code → Command Palette → **Dev Containers: Reopen in Container**.
3. Inside the container terminal:
   - **Backend:** `cd backend && cargo run`
   - **Frontend:** `cd frontend && npm i && npm run dev`
4. Open the frontend URL (usually http://localhost:5173). It fetches mock data from http://localhost:8080/api/v1/battery.

## Notes
- MQTT broker runs as service `mqtt` (port 1883). Wire it in later by pointing your backend MQTT client at host `mqtt` in the same Docker network.
- VS Code tasks available: **backend:run**, **frontend:dev**, **start:all**.
