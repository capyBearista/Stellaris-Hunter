import { NavLink } from 'react-router-dom';

export function Nav() {
  return (
    <nav className="app-nav">
      <span className="nav-brand">Stellaris Hunter</span>
      <NavLink to="/" end className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}>
        Overview
      </NavLink>
      <NavLink
        to="/achievements"
        className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
      >
        Achievements
      </NavLink>
      <NavLink to="/runs" className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}>
        Runs
      </NavLink>
      <NavLink
        to="/settings"
        className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
      >
        Settings
      </NavLink>
    </nav>
  );
}
