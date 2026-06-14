import { Component, type ReactNode } from 'react';
import { HashRouter, Routes, Route } from 'react-router-dom';
import { Nav } from './components/Nav';
import { Overview } from './pages/Overview';
import { Achievements } from './pages/Achievements';
import { Planner } from './pages/Planner';
import { Runs } from './pages/Runs';
import { Settings } from './pages/Settings';

// ---------------------------------------------------------------------------
// Error boundary — prevents a single page crash from killing the entire app
// ---------------------------------------------------------------------------

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
}

class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <section className="panel">
          <h2>Something went wrong</h2>
          <p role="alert" className="error">
            {this.state.error.message}
          </p>
          <button onClick={() => this.setState({ error: null })}>Try again</button>
        </section>
      );
    }
    return this.props.children;
  }
}

export function App() {
  return (
    <HashRouter>
      <div className="app-layout">
        <Nav />
        <main className="app-shell">
          <ErrorBoundary>
            <Routes>
              <Route path="/" element={<Overview />} />
              <Route path="/achievements" element={<Achievements />} />
              <Route path="/planner" element={<Planner />} />
              <Route path="/runs" element={<Runs />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </ErrorBoundary>
        </main>
      </div>
    </HashRouter>
  );
}
