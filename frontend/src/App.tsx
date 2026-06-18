import { Dashboard } from "./pages/Dashboard";

export default function App() {
  return (
    <div className="app">
      <header className="app__header">
        <h1>S.M.O.R.E.S.</h1>
        <span className="app__sub">
          Smart Management of Onboard Resources, Electronics, and Systems
        </span>
      </header>
      <main className="app__main">
        <Dashboard />
      </main>
    </div>
  );
}
