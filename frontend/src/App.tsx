import { useState } from 'react';
import Login from './components/Login';
import AdminDashboard from './components/AdminDashboard';
import StaffDashboard from './components/StaffDashboard';
import type { Session } from './types';
import './App.css';

const SESSION_KEY = 'task-manager-session';

function restoreSession(): Session | null {
  try {
    const stored = sessionStorage.getItem(SESSION_KEY);

    if (!stored) return null;

    const session = JSON.parse(stored) as Session;

    if (
      !session.token ||
      !session.user ||
      !['admin', 'staff'].includes(session.user.role)
    ) {
      sessionStorage.removeItem(SESSION_KEY);
      return null;
    }

    return session;
  } catch {
    sessionStorage.removeItem(SESSION_KEY);
    return null;
  }
}

export default function App() {
  const [session, setSession] = useState<Session | null>(
    restoreSession
  );

  function handleLogin(newSession: Session) {
    sessionStorage.setItem(
      SESSION_KEY,
      JSON.stringify(newSession)
    );

    setSession(newSession);
  }

  function logout() {
    sessionStorage.removeItem(SESSION_KEY);
    setSession(null);
  }

  if (!session) {
    return <Login onLogin={handleLogin} />;
  }

  return (
    <>
      <header className="navbar">
        <div className="nav-brand">
          <div className="brand-mark compact">TM</div>
          <strong>Task Manager</strong>
        </div>

        <div className="nav-user">
          <div className="user-details">
            <strong>
              {session.user.role === 'admin' ? 'Admin' : 'James Bond'}
            </strong>
            <span className="muted small">{session.user.email}</span>
          </div>

          <span className="badge role-badge">
            {session.user.role}
          </span>

          <button className="secondary" onClick={logout}>
            Logout
          </button>
        </div>
      </header>

      {session.user.role === 'admin' ? (
        <AdminDashboard session={session} />
      ) : (
        <StaffDashboard session={session} />
      )}
    </>
  );
}
