# RV Systems & Smart-Feature Catalog

> A living map of the systems found on real RVs, what data/controls each exposes,
> and whether an **open integration path** exists today. This drives the
> `subsystem` registry in [`SCHEMAS.md`](./SCHEMAS.md) and the integration
> priorities in the project roadmap. Add rows as new systems are researched.

Legend — **T** = telemetry (read), **C** = control (write, safety-gated).

---

## The integration landscape

The single most important fact for S.M.O.R.E.S.: **the RV industry already has an
open interoperability standard**, and meeting it is higher-leverage than
re-sensing everything.

- **RV-C** — open RVIA-published protocol on **CAN bus @ 250 kbit/s**, purpose-built
  for multi-vendor interoperability (lights, HVAC, thermostats, fans, power,
  batteries, tank panels). Already spoken by Lithionics batteries, Intellitec
  battery guards, Garnet SeeLevel tank panels, and Victron GX devices. Open-source
  bridges exist (RV-Bridge → HomeKit, rvc-proxy). **One RV-C bridge node can
  surface a large fraction of a modern coach.** Validates the handoff's CAN-bus plan.
- **Proprietary smart hubs** (compete-with / interoperate-with): **Lippert
  OneControl**, **Firefly Integrations** (built on **Spyder Controls**, an RV-C
  supplier), **Silverleaf**, **BMPRO**. Mostly multiplex + Bluetooth/WiFi app;
  Firefly/Spyder ride on RV-C underneath.
- **BLE / app backdoors** (semi-open, reverse-engineerable from an ESP32 node):
  **Victron** (VE.Direct / VE.Bus / BLE), **Mopeka** propane, **Hughes Power
  Watchdog** EMS, **BMPRO**, most aftermarket **TPMS**.

**Integration path priority for S.M.O.R.E.S. nodes/adapters:**
1. Native MQTT node (our own firmware) — full control of the envelope.
2. RV-C ↔ canonical bridge — biggest coverage-per-node on modern coaches.
3. Victron VE.Direct / BLE adapter — dominant in DIY/solar RV power.
4. Modbus (Renogy/EPEver solar), and BLE scrapers (Mopeka, Hughes, TPMS).

---

## 1. Electrical / Power

| System | T / C | Signals | Open path | `subsystem` |
|---|---|---|---|---|
| House battery / BMS | T | SOC, V, A, W, temp, cycles | Victron, Lithionics (RV-C), JBD/Daly BLE | `battery` |
| Solar charge controller (MPPT) | T/C | PV V/A/W, yield, on/off | Victron VE.Direct, Renogy/EPEver Modbus | `solar` |
| Inverter / inverter-charger | T/C | AC out, load, mode | Victron VE.Bus, Magnum | `inverter` |
| DC-DC charger (alternator) | T | charge A, state | Victron BLE | `solar`/`inverter` |
| Automatic transfer switch (ATS) | T | active source (shore/gen/inv) | RV-C / discrete sense | `shore_power` |
| Shore power | T | connected, A draw, V/Hz quality | via EMS | `shore_power` |
| EMS / surge protector | T/C | line V/Hz, current, fault, load-shed, trip | Hughes Power Watchdog (BLE), Progressive, SurgeGuard | `ems` |
| Generator | T/C | running, hours, fuel, start/stop (auto-gen-start) | RV-C / discrete | `generator` |

## 2. Water & Plumbing

| System | T / C | Signals | Open path | `subsystem` |
|---|---|---|---|---|
| Fresh / gray / black tanks | T | level %, temp | Garnet SeeLevel (RV-C), resistive/ultrasonic | `tanks` |
| Propane level | T | level % | Mopeka BLE ultrasonic, RV-C senders | `tanks` (kind=propane) |
| Water pump | T/C | on/off, pressure | discrete / RV-C | `water` |
| Water heater (gas/elec/hybrid) | T/C | mode, set/actual temp | Truma CAN, Suburban, Girard | `water` |
| Water filtration / softener | T | flow, status | discrete | `water` |
| Heated tank/pipe pads | T/C | on/off, temp | discrete | `water` |
| Leak sensors | T | wet/dry, location | discrete | `water` |

## 3. Climate / HVAC

| System | T / C | Signals | Open path | `subsystem` |
|---|---|---|---|---|
| Roof A/C (single/multi-zone) | T/C | zone temp, mode, fan | RV-C thermostats | `hvac` |
| Furnace (propane) | T/C | set/actual temp, mode | RV-C / discrete | `hvac` |
| Heat pump / mini-split | T/C | mode, set temp, compressor | vendor app/CAN | `hvac` |
| Truma Combi (heat + hot water) | T/C | mode, temps | Truma CAN/app | `hvac`/`water` |
| Roof fans (MaxxAir/Fantastic) | T/C | speed, lid, rain auto-close | discrete / RV-C | `fans` |
| Cabin / outside temp & humidity | T | temp, RH | own sensor node | `hvac` |
| Fridge / freezer temp | T | temp, door | own sensor node | `fridge` |

## 4. Safety & Detection

| System | T / C | Signals | Open path | `subsystem` |
|---|---|---|---|---|
| LP / propane detector (floor) | T | alarm | discrete 12V | `safety` |
| Carbon monoxide detector | T | alarm | discrete 12V | `safety` |
| Smoke detector | T | alarm | discrete | `safety` |
| Combo CO/LP (Safe-T-Alert, RV Safe) | T | alarm(s) | discrete | `safety` |
| Fire suppression | T | armed/discharged | discrete | `safety` |
| TPMS (Tire Linc, aftermarket) | T | pressure, temp, fast-leak alarm | 433 MHz / BLE receiver node | `tpms` |

## 5. Chassis / Mobility

| System | T / C | Signals | Open path | `subsystem` |
|---|---|---|---|---|
| Auto-leveling jacks (Lippert/HWH) | T/C | position, level, deploy/retract | RV-C / discrete | `leveling` |
| Slide-outs | T/C | position, in/out | RV-C / discrete | `slides` |
| Awnings (Lippert/Carefree) | T/C | position, wind/rain auto-retract | RV-C / discrete | `awning` |
| Air suspension | T | pressure, ride height | chassis CAN | `chassis` |
| Brake controller (towables) | T/C | gain, output | discrete | `chassis` |
| Powertrain (motorhome) | T | RPM, speed, coolant, fuel, OBD2/J1939 | OBD2 ELM327 / J1939 | `powertrain` |

## 6. Lighting & Exterior

| System | T / C | Signals | Open path | `subsystem` |
|---|---|---|---|---|
| Interior zones / dimming | T/C | on/off, brightness, scene | RV-C (newer coaches) | `lighting` |
| Exterior / porch / scare lights | T/C | on/off | RV-C / discrete | `lighting` |
| Awning RGB / accent strips | T/C | color, brightness, mode | RV-C / WS2812 node | `lighting` |

## 7. Connectivity & Infotainment

| System | T / C | Signals | Open path | `subsystem` |
|---|---|---|---|---|
| Starlink (Roam/Mini) | T | online, throughput, **power draw** | gRPC/local API | `connectivity` |
| Cellular booster / router | T | signal, data, failover state | Peplink/TravlFi API | `connectivity` |
| Cameras (backup/side/hitch) | T | stream, motion | RTSP / vendor | `cameras` |
| Stereo / TV / antenna | T/C | source, volume | vendor | `infotainment` |
| GPS / location | T | lat/lon, speed, heading | own node | `location` |

## 8. Security & Access

| System | T / C | Signals | Open path | `subsystem` |
|---|---|---|---|---|
| Smart door locks | T/C | locked/unlocked | OneControl / own node | `security` |
| Entry / compartment sensors | T | open/closed | discrete | `security` |
| Motion sensors | T | motion | discrete | `security` |
| Geofence / movement alert | T | moved, location | own logic | `security` |

> **Safety interlock:** per the project safety policy, no control logic may create
> an unsafe state — e.g. lock-while-driving, ignition, propane, or road-lighting
> control must be explicit, validated, logged, and fail-safe. Default these to
> telemetry-only until a design review signs off on control.

## 9. Appliances

| System | T / C | Signals | `subsystem` |
|---|---|---|---|
| Refrigerator (absorption / 12V compressor) | T | mode, temp, door | `fridge` |
| Cooktop / oven / microwave | T | on, power draw | `appliances` |
| Washer / dryer / dishwasher / ice maker | T | cycle, status | `appliances` |

---

## Canonical subsystem ids (registry)

Implemented in M1: `battery`, `tanks`, `tpms`.

Reserved (named now for node-author consistency; stored as `unknown` until
implemented): `solar`, `inverter`, `shore_power`, `ems`, `generator`, `water`,
`hvac`, `fans`, `fridge`, `safety`, `leveling`, `slides`, `awning`, `chassis`,
`powertrain`, `lighting`, `connectivity`, `cameras`, `infotainment`, `location`,
`security`, `appliances`.

## Sources

RVIA RV-C · Wikipedia RV-C · Intellitec, Victron RV-C docs · RV-Bridge & rvc-proxy
(GitHub) · Lippert OneControl & sensors · Firefly Integrations · Garnet SeeLevel ·
BMPRO · Progressive Industries & Hughes Power Watchdog EMS · Keystone & Safe-T-Alert
safety devices · 2026 RV connectivity guides (Starlink/Peplink/cell boosters).
