import { NavLink } from 'react-router-dom';
import { Terminal, Award, Radar, Database, Sliders } from 'lucide-react';

export function Nav() {
  return (
    <nav className="app-nav">
      <span className="nav-brand">Stellaris Hunter</span>
      <NavLink to="/" end className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}>
        <Terminal size={18} aria-hidden="true" /> Overview
      </NavLink>
      <NavLink
        to="/achievements"
        className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
      >
        <Award size={18} aria-hidden="true" /> Achievements
      </NavLink>
      <NavLink
        to="/planner"
        className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
      >
        <Radar size={18} aria-hidden="true" /> Planner
      </NavLink>
      <NavLink to="/runs" className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}>
        <Database size={18} aria-hidden="true" /> Runs
      </NavLink>
      <NavLink
        to="/settings"
        className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
      >
        <Sliders size={18} aria-hidden="true" /> Settings
      </NavLink>
    </nav>
  );
}
