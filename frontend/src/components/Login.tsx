import { useState } from 'react';
import type { FormEvent } from 'react';
import { api, getErrorMessage } from '../api';
import type { Session } from '../types';

interface Props {
  onLogin: (session: Session) => void;
}

export default function Login({ onLogin }: Props) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [challengeId, setChallengeId] = useState('');
  const [code, setCode] = useState('');
  const [devCode, setDevCode] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');

  async function handleLogin(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setLoading(true);
    setError('');
    setMessage('');

    try {
      const response = await api.login(email, password);
      setChallengeId(response.login_challenge_id);
      setPassword('');
      setMessage('Verification code sent. Valid for 5 minutes.');
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleVerify(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      const response = await api.verify2FA(challengeId, code);

      onLogin({
        token: response.access_token,
        user: response.user,
      });
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  async function getDevCode() {
    setLoading(true);
    setError('');

    try {
      const log = await api.latestEmail();

      if (log.login_challenge_id !== challengeId) {
        throw new Error('Latest email belongs to another login.');
      }

      setDevCode(log.code);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  async function seedUsers() {
    setLoading(true);
    setError('');
    setMessage('');

    try {
      const response = await api.seedUsers();
      setMessage(response.message);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  function resetLogin() {
    setChallengeId('');
    setCode('');
    setDevCode('');
    setError('');
    setMessage('');
  }

  return (
    <main className="login-page">
      <section className="login-card">
        <div className="brand-mark">TM</div>
        <span className="eyebrow">TASK MANAGEMENT SYSTEM</span>

        <h1>{challengeId ? 'Verify your identity' : 'Welcome back'}</h1>

        <p className="muted">
          {challengeId
            ? `Enter the verification code sent to ${email}`
            : 'Sign in to manage your workspace.'}
        </p>

        {error && <div className="alert error" role="alert">{error}</div>}
        {message && <div className="alert success" role="status">{message}</div>}

        {!challengeId ? (
          <>
            <form onSubmit={handleLogin}>
              <label>
                Email address
                <input
                  type="email"
                  value={email}
                  onChange={e => setEmail(e.target.value)}
                  placeholder="you@example.com"
                  autoComplete="email"
                  required
                />
              </label>

              <label>
                Password
                <input
                  type="password"
                  value={password}
                  onChange={e => setPassword(e.target.value)}
                  autoComplete="current-password"
                  placeholder="Enter your password"
                  required
                />
              </label>

              <button className="primary full" disabled={loading}>
                {loading ? 'Signing in...' : 'Sign in'}
              </button>
            </form>

            <div className="demo-section">
              <h3>Development accounts</h3>

              <button
                className="secondary full"
                onClick={() => {
                  setEmail('admin@example.com');
                  setPassword('Admin@123');
                }}
                disabled={loading}
              >
                Use Admin account
              </button>

              <button
                className="secondary full"
                onClick={() => {
                  setEmail('jamesbond@example.com');
                  setPassword('James@123');
                }}
                disabled={loading}
              >
                Use James Bond account
              </button>

              <button
                className="text-button"
                onClick={seedUsers}
                disabled={loading}
              >
                Initialize demo users
              </button>
            </div>
          </>
        ) : (
          <>
            <form onSubmit={handleVerify}>
              <label>
                Verification code
                <input
                  className="otp-input"
                  type="text"
                  inputMode="numeric"
                  autoComplete="one-time-code"
                  pattern="[0-9]{6}"
                  maxLength={6}
                  value={code}
                  onChange={e =>
                    setCode(e.target.value.replace(/\D/g, ''))
                  }
                  placeholder="000000"
                  required
                />
              </label>

              <button
                className="primary full"
                disabled={loading || code.length !== 6}
              >
                {loading ? 'Verifying...' : 'Verify and sign in'}
              </button>
            </form>

            <div className="demo-section">
              <h3>Development email inbox</h3>
              <p className="muted small">
                Email delivery is simulated locally.
              </p>

              <button
                className="secondary full"
                onClick={getDevCode}
                disabled={loading}
              >
                Retrieve verification code
              </button>

              {devCode && (
                <div className="dev-code">
                  <strong>{devCode}</strong>
                  <button
                    className="secondary"
                    onClick={() => setCode(devCode)}
                  >
                    Use code
                  </button>
                </div>
              )}

              <button
                className="text-button"
                onClick={resetLogin}
                disabled={loading}
              >
                Back to login
              </button>
            </div>
          </>
        )}
      </section>
    </main>
  );
}
