import type {
  LoginResponse,
  VerifyResponse,
  EmailLog,
  Task,
  CreateTaskRequest,
  AssignTasksRequest,
  AssignTasksResponse,
  MyTasksResponse,
  SeedResponse,
} from './types';

const API_URL =
  import.meta.env.VITE_API_URL || 'http://localhost:8080';

export class ApiError extends Error {
  readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

async function request<T>(
  path: string,
  method = 'GET',
  body?: unknown,
  token?: string,
): Promise<T> {
  const headers: Record<string, string> = {
    Accept: 'application/json',
  };

  if (body !== undefined) {
    headers['Content-Type'] = 'application/json';
  }

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  const response = await fetch(`${API_URL}${path}`, {
    method,
    headers,
    body: body !== undefined
      ? JSON.stringify(body)
      : undefined,
  });

  const data = await response.json().catch(() => null);

  if (!response.ok) {
    throw new ApiError(
      data?.error || `Request failed (${response.status})`,
      response.status,
    );
  }

  return data as T;
}

export const api = {
  seedUsers(): Promise<SeedResponse> {
    return request('/seed/users', 'POST');
  },

  login(
    email: string,
    password: string,
  ): Promise<LoginResponse> {
    return request('/auth/login', 'POST', {
      email,
      password,
    });
  },

  verify2FA(
    challengeId: string,
    code: string,
  ): Promise<VerifyResponse> {
    return request('/auth/verify-2fa', 'POST', {
      login_challenge_id: challengeId,
      code,
    });
  },

  latestEmail(): Promise<EmailLog> {
    return request('/dev/email-logs/latest');
  },

  createTask(
    token: string,
    body: CreateTaskRequest,
  ): Promise<Task> {
    return request('/tasks', 'POST', body, token);
  },

  listTasks(token: string): Promise<Task[]> {
    return request('/tasks', 'GET', undefined, token);
  },

  assignTasks(
    token: string,
    body: AssignTasksRequest,
  ): Promise<AssignTasksResponse> {
    return request('/tasks/assign', 'POST', body, token);
  },

  viewMyTasks(token: string): Promise<MyTasksResponse> {
    return request(
      '/tasks/view-my-tasks',
      'GET',
      undefined,
      token,
    );
  },
};

export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return 'An unexpected error occurred';
}
