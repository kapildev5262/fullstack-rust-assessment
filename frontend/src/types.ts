export type Role = 'admin' | 'staff';

export type Priority = 'low' | 'medium' | 'high';

export type TaskStatus = 'todo' | 'in_progress' | 'done';

export interface User {
  id: string;
  email: string;
  role: Role;
}

export interface LoginResponse {
  login_challenge_id: string;
  message: string;
  expires_in: number;
}

export interface VerifyResponse {
  access_token: string;
  user: User;
}

export interface EmailLog {
  email: string;
  login_challenge_id: string;
  code: string;
  created_at: string;
}

export interface Task {
  id: string;
  title: string;
  description: string;
  status: TaskStatus;
  priority: Priority;
  created_by_id: string;
  assigned_to_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface TaskView {
  id: string;
  title: string;
  status: TaskStatus;
  priority: Priority;
  assigned_to: string;
}

export interface CreateTaskRequest {
  title: string;
  description: string;
  priority: Priority;
}

export interface AssignTasksRequest {
  task_ids: string[];
  assigned_to_email: string;
}

export interface AssignTasksResponse {
  message: string;
  assigned_to: string;
  assigned_count: number;
}

export interface MyTasksResponse {
  user: {
    email: string;
    role: Role;
  };
  tasks: TaskView[];
  summary: {
    total_assigned_tasks: number;
  };
  cache: {
    hit: boolean;
  };
}

export interface SeedResponse {
  message: string;
  users: User[];
}

export interface Session {
  token: string;
  user: User;
}
