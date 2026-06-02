import { HashRouter, Routes, Route } from 'react-router-dom';
import { Nav } from './components/Nav';
import { Overview } from './pages/Overview';
import { Achievements } from './pages/Achievements';
import { Runs } from './pages/Runs';
import { Settings } from './pages/Settings';

export function App() {
  return (
    <HashRouter>
      <div className="app-layout">
        <Nav />
        <main className="app-shell">
          <Routes>
            <Route path="/" element={<Overview />} />
            <Route path="/achievements" element={<Achievements />} />
            <Route path="/runs" element={<Runs />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </main>
      </div>
    </HashRouter>
  );
}
