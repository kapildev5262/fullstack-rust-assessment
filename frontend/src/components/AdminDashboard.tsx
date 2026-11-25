import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';
import { api, getErrorMessage } from '../api';
import type { Priority, Session, Task } from '../types';

interface Props {
  session: Session;
}

export default function AdminDashboard({ session }: Props) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<Priority>('medium');
  const [assignee, setAssignee] = useState('jamesbond@example.com');

  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');

  const token = session.token;

  useEffect(() => {
    let active = true;

    api.listTasks(token)
      .then(data => {
        if (active) setTasks(data);
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
      setTasks(await api.listTasks(token));
      setSelected([]);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  async function createTask(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setBusy(true);
    setError('');
    setMessage('');

    try {
      const task = await api.createTask(token, {
        title: title.trim(),
        description: description.trim(),
        priority,
      });

      setTasks(previous => [...previous, task]);
      setTitle('');
      setDescription('');
      setPriority('medium');
      setMessage('Task created successfully.');
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function assignTasks(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setBusy(true);
    setError('');
    setMessage('');

    try {
      const response = await api.assignTasks(token, {
        task_ids: selected,
        assigned_to_email: assignee,
      });

      setMessage(
        `${response.assigned_count} task(s) assigned to ${response.assigned_to}.`
      );

      setSelected([]);
      await refreshTasks();
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  function toggleTask(id: string) {
    setSelected(previous =>
      previous.includes(id)
        ? previous.filter(taskId => taskId !== id)
        : [...previous, id]
    );
  }

  const assignedCount = tasks.filter(
    task => task.assigned_to_id !== null
  ).length;

  return (
    <main className="dashboard">
      <div className="page-heading">
        <div>
          <span className="eyebrow">ADMIN WORKSPACE</span>
          <h1>Task management</h1>
          <p className="muted">
            Create tasks and manage staff assignments.
          </p>
        </div>

        <button
          className="secondary"
          onClick={refreshTasks}
          disabled={loading || busy}
        >
          Refresh tasks
        </button>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <span>Total tasks</span>
          <strong>{tasks.length}</strong>
        </div>

        <div className="stat-card">
          <span>Assigned tasks</span>
          <strong>{assignedCount}</strong>
        </div>

        <div className="stat-card">
          <span>Unassigned tasks</span>
          <strong>{tasks.length - assignedCount}</strong>
        </div>
      </div>

      {error && <div className="alert error" role="alert">{error}</div>}
      {message && <div className="alert success" role="status">{message}</div>}

      <div className="admin-grid">
        <section className="panel">
          <h2>Create task</h2>
          <p className="muted small">Add a task to the workspace.</p>

          <form onSubmit={createTask}>
            <label>
              Task title
              <input
                value={title}
                onChange={e => setTitle(e.target.value)}
                placeholder="Enter task title"
                maxLength={200}
                required
              />
            </label>

            <label>
              Description
              <textarea
                value={description}
                onChange={e => setDescription(e.target.value)}
                placeholder="Describe the task"
                rows={4}
              />
            </label>

            <label>
              Priority
              <select
                value={priority}
                onChange={e => setPriority(e.target.value as Priority)}
              >
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
              </select>
            </label>

            <button
              className="primary full"
              disabled={busy || loading || !title.trim()}
            >
              {busy ? 'Processing...' : 'Create task'}
            </button>
          </form>
        </section>

        <section className="panel">
          <h2>All tasks</h2>
          <p className="muted small">
            Select tasks to assign to a staff member.
          </p>

          {loading ? (
            <div className="empty-state">Loading tasks...</div>
          ) : tasks.length === 0 ? (
            <div className="empty-state">No tasks created yet.</div>
          ) : (
            <div className="table-wrapper">
              <table>
                <thead>
                  <tr>
                    <th>Select</th>
                    <th>Task</th>
                    <th>Priority</th>
                    <th>Status</th>
                    <th>Assignment</th>
                  </tr>
                </thead>

                <tbody>
                  {tasks.map(task => (
                    <tr key={task.id}>
                      <td>
                        <input
                          type="checkbox"
                          aria-label={`Select ${task.title}`}
                          checked={selected.includes(task.id)}
                          onChange={() => toggleTask(task.id)}
                          disabled={busy}
                        />
                      </td>

                      <td>
                        <strong>{task.title}</strong>
                        <div className="muted small">
                          {task.description}
                        </div>
                      </td>

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

                      <td>
                        {task.assigned_to_id ? (
                          <span className="badge assigned">Assigned</span>
                        ) : (
                          <span className="muted small">Unassigned</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          <form className="assignment-form" onSubmit={assignTasks}>
            <label>
              Assign to staff
              <select
                value={assignee}
                onChange={e => setAssignee(e.target.value)}
              >
                <option value="jamesbond@example.com">
                  James Bond (jamesbond@example.com)
                </option>
              </select>
            </label>

            <button
              className="primary"
              disabled={busy || loading || selected.length === 0}
            >
              Assign {selected.length} selected task(s)
            </button>
          </form>
        </section>
      </div>
    </main>
  );
}
