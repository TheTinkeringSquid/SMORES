import { useState } from "react";
import { Dashboard } from "./pages/Dashboard";
import { Settings } from "./pages/Settings";

type View = "dashboard" | "settings";

export default function App() {
  const [view, setView] = useState<View>("dashboard");
  return (
    <div className="app">
      <header className="app__header">
        <h1>S.M.O.R.E.S.</h1>
        <nav className="app__nav">
          <button
            className={view === "dashboard" ? "active" : ""}
            onClick={() => setView("dashboard")}
          >
            Dashboard
          </button>
          <button
            className={view === "settings" ? "active" : ""}
            onClick={() => setView("settings")}
          >
            Settings
          </button>
        </nav>
      </header>
      <main className="app__main">{view === "dashboard" ? <Dashboard /> : <Settings />}</main>
    </div>
  );
}
