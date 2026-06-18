import { useEffect, useState } from "react";

type BatteryTelemetry = {
  soc_percent?: number;
  voltage_v: number;
  current_a?: number;
  temp_c?: number;
  timestamp: string;
};

export default function App() {
  const [data, setData] = useState<BatteryTelemetry | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const base = import.meta.env.VITE_API_BASE ?? "http://localhost:8080/api/v1";
        const res = await fetch(`${base}/battery`);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const json = await res.json();
        setData(json.battery);
      } catch (e: any) {
        setErr(e.message);
      }
    };
    load();
    const id = setInterval(load, 3000);
    return () => clearInterval(id);
  }, []);

  if (err) return <div style={{padding:16}}>Error: {err}</div>;
  if (!data) return <div style={{padding:16}}>Waiting for telemetry…</div>;

  return (
    <div style={{padding:16, display:"grid", gap:12}}>
      <h1>S.M.O.R.E.S. Dashboard</h1>
      <div style={{border:"1px solid #444", borderRadius:12, padding:16}}>
        <h2>Battery</h2>
        <div>SOC: {data.soc_percent?.toFixed(1) ?? "—"}%</div>
        <div>Voltage: {data.voltage_v.toFixed(2)} V</div>
        {data.current_a !== undefined && <div>Current: {data.current_a.toFixed(1)} A</div>}
        {data.temp_c !== undefined && <div>Temp: {data.temp_c.toFixed(1)} °C</div>}
        <small style={{opacity:0.7}}>
          {new Date(data.timestamp).toLocaleString()}
        </small>
      </div>
    </div>
  );
}
