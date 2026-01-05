export type CheckStatus = 'installed' | 'notinstalled' | 'needsconfig';

export interface CheckResult {
  status: CheckStatus;
  version: string | null;
  message: string | null;
}

export interface PrerequisiteStatus {
  xcode_cli: CheckResult;
  rust: CheckResult;
  claude_cli: CheckResult;
  claude_auth: CheckResult;
}

export interface AppConfig {
  workspacePath: string;
  outputPath: string;
  buildFormats: string[];
  autoOpenOutput: boolean;
  showNotifications: boolean;
  theme: 'dark' | 'light';
  setupComplete: boolean;
}

export interface ProjectMeta {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
  path: string;
}

export interface CreateProjectInput {
  name: string;
  description: string;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
  filesModified?: string[];
  summary?: string;
  commitHash?: string;
  version?: number;  // Version number for commits (1, 2, 3...) - only set if files were changed
  reverted: boolean;
}

export interface ChatState {
  messages: ChatMessage[];
  activeVersion: number | null;
}
