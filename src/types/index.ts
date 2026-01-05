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

export interface DawPathConfig {
  vst3: string;
  clap: string;
}

export interface DawPaths {
  reaper: DawPathConfig;
  ableton: DawPathConfig;
  flStudio: DawPathConfig;
  logic: DawPathConfig;
  other: DawPathConfig;
}

export interface CustomThemeColors {
  accent: string;
  bgPrimary: string;
  bgSecondary: string;
  bgTertiary: string;
  textPrimary: string;
  textSecondary: string;
}

export interface AppConfig {
  workspacePath: string;
  outputPath: string;
  buildFormats: string[];
  autoOpenOutput: boolean;
  showNotifications: boolean;
  theme: 'dark' | 'light' | 'custom';
  customColors: CustomThemeColors;
  setupComplete: boolean;
  // Branding
  vendorName: string;
  vendorUrl: string;
  vendorEmail: string;
  // DAW plugin paths
  dawPaths: DawPaths;
}

export interface ProjectMeta {
  id: string;
  name: string;
  description: string;
  template?: PluginTemplate;
  created_at: string;
  updated_at: string;
  path: string;
}

export type PluginTemplate = 'effect' | 'instrument';

export interface CreateProjectInput {
  name: string;
  description: string;
  template: PluginTemplate;
  vendorName?: string;
  vendorUrl?: string;
  vendorEmail?: string;
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
