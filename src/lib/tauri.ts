import { invoke } from '@tauri-apps/api/core';
import type { PrerequisiteStatus } from '../types';

export async function checkPrerequisites(): Promise<PrerequisiteStatus> {
  return invoke<PrerequisiteStatus>('check_prerequisites');
}

// Installation commands
export async function checkHomebrew(): Promise<boolean> {
  return invoke<boolean>('check_homebrew');
}

export async function checkNode(): Promise<boolean> {
  return invoke<boolean>('check_node');
}

export async function installHomebrew(): Promise<boolean> {
  return invoke<boolean>('install_homebrew');
}

export async function installNode(): Promise<boolean> {
  return invoke<boolean>('install_node');
}

export async function installXcode(): Promise<boolean> {
  return invoke<boolean>('install_xcode');
}

export async function installRust(): Promise<boolean> {
  return invoke<boolean>('install_rust');
}

export async function installClaudeCli(): Promise<boolean> {
  return invoke<boolean>('install_claude_cli');
}

export async function startClaudeAuth(): Promise<boolean> {
  return invoke<boolean>('start_claude_auth');
}
