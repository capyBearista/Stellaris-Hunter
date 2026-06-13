import { NavLink } from 'react-router-dom';
import { LayoutDashboard, Trophy, Map, FolderOpen, Settings } from 'lucide-react';

export function Nav() {
  return (
    <nav className="app-nav">
      <span className="nav-brand">Stellaris Hunter</span>
      <NavLink to="/" end className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}>
        <LayoutDashboard size={18} /> Overview
      </NavLink>
      <NavLink
        to="/achievements"
        className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
      >
        <Trophy size={18} /> Achievements
      </NavLink>
      <NavLink
        to="/planner"
        className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
      >
        <Map size={18} /> Planner
      </NavLink>
      <NavLink to="/runs" className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}>
        <FolderOpen size={18} /> Runs
      </NavLink>
      <NavLink
        to="/settings"
        className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
      >
        <Settings size={18} /> Settings
      </NavLink>
    </nav>
  );
}
