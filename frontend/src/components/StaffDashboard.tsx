import { useEffect, useState } from 'react';
import { api, ApiError, getErrorMessage } from '../api';
import type { MyTasksResponse, Session } from '../types';

interface Props {
  session: Session;
}

export default function StaffDashboard({ session }: Props) {
  const [data, setData] = useState<MyTasksResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState('');
  const [permissionResult, setPermissionResult] = useState('');
  const [permissionDenied, setPermissionDenied] = useState(false);

  const token = session.token;

  useEffect(() => {
    let active = true;

    api.viewMyTasks(token)
      .then(response => {
        if (active) setData(response);
      })
      .catch(err => {
        if (active) setError(getErrorMessage(err));
      })
      .finally(() => {
        if (active) setLoading(false);
      });

    return () => {
      active = false;
    };
  }, [token]);

  async function refreshTasks() {
    setLoading(true);
    setError('');

    try {
      setData(await api.viewMyTasks(token));
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  async function testCreatePermission() {
    setTesting(true);
    setPermissionResult('');
    setPermissionDenied(false);

    try {
      await api.createTask(token, {
        title: 'Unauthorized task',
        description: 'Staff permission verification',
        priority: 'high',
      });

      setPermissionResult(
        'Unexpected result: staff task creation was allowed.'
      );
    } catch (err) {
      if (err instanceof ApiError && err.status === 403) {
        setPermissionDenied(true);
        setPermissionResult(`403 Forbidden: ${err.message}`);
      } else {
        setPermissionResult(getErrorMessage(err));
      }
    } finally {
      setTesting(false);
    }
  }

  return (
    <main className="dashboard">
      <div className="page-heading">
        <div>
          <span className="eyebrow">STAFF WORKSPACE</span>
          <h1>My assigned tasks</h1>
          <p className="muted">
            View and track the tasks assigned to you.
          </p>
        </div>

        <button
          className="secondary"
          onClick={refreshTasks}
          disabled={loading}
        >
          {loading ? 'Refreshing...' : 'Refresh tasks'}
        </button>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <span>Assigned tasks</span>
          <strong>{data?.summary.total_assigned_tasks ?? '—'}</strong>
        </div>

        <div className="stat-card">
          <span>High priority</span>
          <strong>
            {data?.tasks.filter(task => task.priority === 'high').length ?? '—'}
          </strong>
        </div>

        <div className="stat-card">
          <span>Cache status</span>
          <strong className="cache-value">
            {data ? (data.cache.hit ? 'HIT' : 'MISS') : '—'}
          </strong>
          {data && (
            <span className="muted small">
              {data.cache.hit ? 'Served from cache' : 'Fetched from database'}
            </span>
          )}
        </div>
      </div>

      {error && <div className="alert error" role="alert">{error}</div>}

      <section className="panel">
        <h2>Assigned tasks</h2>
        <p className="muted small">{session.user.email}</p>

        {loading ? (
          <div className="empty-state">Loading your tasks...</div>
        ) : !data || data.tasks.length === 0 ? (
          <div className="empty-state">No assigned tasks found.</div>
        ) : (
          <div className="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Task title</th>
                  <th>Priority</th>
                  <th>Status</th>
                  <th>Assigned to</th>
                </tr>
              </thead>

              <tbody>
                {data.tasks.map(task => (
                  <tr key={task.id}>
                    <td><strong>{task.title}</strong></td>
                    <td>
                      <span className={`badge ${task.priority}`}>
                        {task.priority}
                      </span>
                    </td>
                    <td>
                      <span className="badge neutral">
                        {task.status.replaceAll('_', ' ')}
                      </span>
                    </td>
                    <td>{task.assigned_to}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section className="panel permission-panel">
        <h2>Access permissions</h2>
        <p className="muted">
          Staff members can view their assigned tasks.
          Only administrators can create or assign tasks.
        </p>

        <button
          className="secondary"
          onClick={testCreatePermission}
          disabled={testing}
        >
          {testing ? 'Testing permission...' : 'Attempt task creation'}
        </button>

        {permissionResult && (
          <div
            className={`alert ${permissionDenied ? 'warning' : 'error'}`}
            role="status"
          >
            {permissionResult}
          </div>
        )}
      </section>
    </main>
  );
}
