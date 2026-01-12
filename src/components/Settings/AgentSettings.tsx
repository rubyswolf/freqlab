import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../stores/settingsStore';
import { useProjectBusyStore } from '../../stores/projectBusyStore';
import type { ClaudeModel, AgentVerbosity, AgentProviderType, ProviderStatus } from '../../types';

interface ModelOption {
  id: ClaudeModel;
  label: string;
  description: string;
}

interface VerbosityOption {
  id: AgentVerbosity;
  label: string;
  description: string;
}

interface ProviderOption {
  id: AgentProviderType;
  label: string;
  description: string;
  binary: string;
}

const providerOptions: ProviderOption[] = [
  { id: 'claude', label: 'Claude Code', description: 'Anthropic\'s official CLI', binary: 'claude' },
  { id: 'opencode', label: 'OpenCode', description: '75+ LLM providers', binary: 'opencode' },
];

// Model options for Claude Code provider (native CLI)
const claudeCodeModelOptions: ModelOption[] = [
  { id: 'haiku', label: 'Haiku', description: 'Fast intern' },
  { id: 'sonnet', label: 'Sonnet', description: 'Reliable engineer' },
  { id: 'opus', label: 'Opus', description: 'Senior audio engineer' },
];

const openCodeModelOptions = [
  { id: 'anthropic/claude-sonnet-4', label: 'Claude Sonnet 4', description: 'Claude via OpenCode' },
  { id: 'anthropic/claude-opus-4', label: 'Claude Opus 4', description: 'Claude Opus via OpenCode' },
  { id: 'openai/gpt-4o', label: 'GPT-4o', description: 'OpenAI GPT-4o' },
  { id: 'openai/o1', label: 'o1', description: 'OpenAI o1 reasoning model' },
  { id: 'google/gemini-2.0-flash', label: 'Gemini 2.0 Flash', description: 'Google Gemini' },
];

const verbosityOptions: VerbosityOption[] = [
  { id: 'direct', label: 'Direct', description: 'Minimal questions, just implement' },
  { id: 'balanced', label: 'Balanced', description: 'A few key questions, then implement' },
  { id: 'thorough', label: 'Thorough', description: 'Detailed exploration before implementing' },
];

export function AgentSettings() {
  const {
    agentSettings,
    setChatStyle,
    setModel,
    setCustomInstructions,
    setAgentVerbosity,
    setDefaultProvider,
    setProviderModel,
  } = useSettingsStore();

  const [providerStatus, setProviderStatus] = useState<Record<AgentProviderType, ProviderStatus>>({
    claude: { installed: false, authenticated: false },
    opencode: { installed: false, authenticated: false },
  });
  const [checkingProviders, setCheckingProviders] = useState(true);

  // Check if any project has an active agent - lock settings during active chats
  const isAnyAgentBusy = useProjectBusyStore((s) => s.isAnyBusy());

  // Check provider installation status on mount
  useEffect(() => {
    async function checkProviders() {
      setCheckingProviders(true);
      try {
        const status = await invoke<Record<AgentProviderType, ProviderStatus>>('check_agent_providers');
        setProviderStatus(status);
      } catch (e) {
        console.error('Failed to check providers:', e);
      } finally {
        setCheckingProviders(false);
      }
    }
    checkProviders();
  }, []);

  const currentProvider = agentSettings.defaultProvider || 'claude';
  const currentProviderStatus = providerStatus[currentProvider];

  // Get model options based on selected provider
  const modelOptions = currentProvider === 'claude' ? claudeCodeModelOptions : openCodeModelOptions;
  const currentModel = currentProvider === 'claude'
    ? agentSettings.providerModels?.claude || agentSettings.model
    : agentSettings.providerModels?.opencode || 'anthropic/claude-opus-4';

  return (
    <div className="space-y-5">
      <div>
        <h3 className="text-lg font-medium text-text-primary mb-1">Agent Settings</h3>
        <p className="text-sm text-text-muted">Configure your coding agent</p>
      </div>

      {/* Provider Selection */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <div>
            <label className="block text-sm font-medium text-text-primary">Agent Provider</label>
            <p className="text-xs text-text-muted">
              {providerOptions.find(p => p.id === currentProvider)?.description}
            </p>
          </div>
          <div className="flex rounded-lg border border-border overflow-hidden">
            {providerOptions.map((option, index) => {
              const status = providerStatus[option.id];
              const isAvailable = status?.installed;
              const isDisabled = isAnyAgentBusy || (!isAvailable && !checkingProviders);
              return (
                <button
                  key={option.id}
                  onClick={() => isAvailable && !isAnyAgentBusy && setDefaultProvider(option.id)}
                  disabled={isDisabled}
                  className={`px-3 py-1.5 text-sm transition-colors ${
                    index > 0 ? 'border-l border-border' : ''
                  } ${
                    currentProvider === option.id
                      ? 'bg-accent text-white'
                      : isDisabled
                        ? 'bg-bg-primary text-text-muted cursor-not-allowed opacity-50'
                        : 'bg-bg-primary text-text-secondary hover:text-text-primary'
                  }`}
                  title={
                    isAnyAgentBusy
                      ? 'Cannot change provider while agent is working'
                      : !isAvailable && !checkingProviders
                        ? `${option.label} is not installed`
                        : undefined
                  }
                >
                  {option.label}
                </button>
              );
            })}
          </div>
        </div>

        {/* Provider status indicator */}
        {!checkingProviders && (
          <div className="flex items-center gap-2 text-xs">
            <span className={`w-2 h-2 rounded-full ${
              currentProviderStatus?.installed
                ? currentProviderStatus?.authenticated
                  ? 'bg-green-500'
                  : 'bg-yellow-500'
                : 'bg-red-500'
            }`} />
            <span className="text-text-muted">
              {currentProviderStatus?.installed
                ? currentProviderStatus?.authenticated
                  ? `${providerOptions.find(p => p.id === currentProvider)?.label} ready${currentProviderStatus.version ? ` (v${currentProviderStatus.version})` : ''}`
                  : `${providerOptions.find(p => p.id === currentProvider)?.label} needs authentication`
                : `${providerOptions.find(p => p.id === currentProvider)?.label} not installed`
              }
            </span>
          </div>
        )}
      </div>

      {/* Chat Style - inline toggle */}
      <div className="flex items-center justify-between">
        <div>
          <label className="block text-sm font-medium text-text-primary">Chat Style</label>
          <p className="text-xs text-text-muted">How responses are displayed</p>
        </div>
        <div className="flex rounded-lg border border-border overflow-hidden">
          <button
            onClick={() => setChatStyle('conversational')}
            className={`px-3 py-1.5 text-sm transition-colors ${
              agentSettings.chatStyle === 'conversational'
                ? 'bg-accent text-white'
                : 'bg-bg-primary text-text-secondary hover:text-text-primary'
            }`}
          >
            Conversational
          </button>
          <button
            onClick={() => setChatStyle('minimal')}
            className={`px-3 py-1.5 text-sm transition-colors border-l border-border ${
              agentSettings.chatStyle === 'minimal'
                ? 'bg-accent text-white'
                : 'bg-bg-primary text-text-secondary hover:text-text-primary'
            }`}
          >
            Minimal
          </button>
        </div>
      </div>

      {/* Agent Verbosity - toggle */}
      <div className="flex items-center justify-between">
        <div>
          <label className="block text-sm font-medium text-text-primary">Response Style</label>
          <p className="text-xs text-text-muted">
            {verbosityOptions.find(v => v.id === agentSettings.agentVerbosity)?.description}
          </p>
        </div>
        <div className="flex rounded-lg border border-border overflow-hidden">
          {verbosityOptions.map((option, index) => (
            <button
              key={option.id}
              onClick={() => setAgentVerbosity(option.id)}
              className={`px-3 py-1.5 text-sm transition-colors ${
                index > 0 ? 'border-l border-border' : ''
              } ${
                agentSettings.agentVerbosity === option.id
                  ? 'bg-accent text-white'
                  : 'bg-bg-primary text-text-secondary hover:text-text-primary'
              }`}
            >
              {option.label}
            </button>
          ))}
        </div>
      </div>

      {/* Model Selection - dynamic based on provider */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <div>
            <label className="block text-sm font-medium text-text-primary">Model</label>
            <p className="text-xs text-text-muted">
              {modelOptions.find(m => m.id === currentModel)?.description || 'Select a model'}
            </p>
          </div>
          {isAnyAgentBusy && (
            <span className="text-xs text-warning">Locked while agent is working</span>
          )}
        </div>
        {currentProvider === 'claude' ? (
          <div className="flex rounded-lg border border-border overflow-hidden">
            {claudeCodeModelOptions.map((option, index) => (
              <button
                key={option.id}
                disabled={isAnyAgentBusy}
                onClick={() => {
                  if (isAnyAgentBusy) return;
                  // Atomic update: set both model and providerModel together
                  setModel(option.id);
                  setProviderModel('claude', option.id);
                }}
                className={`px-3 py-1.5 text-sm transition-colors ${
                  index > 0 ? 'border-l border-border' : ''
                } ${
                  currentModel === option.id
                    ? 'bg-accent text-white'
                    : isAnyAgentBusy
                      ? 'bg-bg-primary text-text-muted cursor-not-allowed opacity-50'
                      : 'bg-bg-primary text-text-secondary hover:text-text-primary'
                }`}
              >
                {option.label}
              </button>
            ))}
          </div>
        ) : (
          <select
            value={currentModel}
            disabled={isAnyAgentBusy}
            onChange={(e) => {
              if (isAnyAgentBusy) return;
              setProviderModel('opencode', e.target.value);
            }}
            className={`w-full px-3 py-2 rounded-lg border border-border bg-bg-primary text-text-primary focus:outline-none focus:ring-2 focus:ring-accent/50 text-sm ${
              isAnyAgentBusy ? 'opacity-50 cursor-not-allowed' : ''
            }`}
          >
            {openCodeModelOptions.map(option => (
              <option key={option.id} value={option.id}>
                {option.label}
              </option>
            ))}
          </select>
        )}
      </div>

      {/* Custom Instructions - textarea */}
      <div className="space-y-2">
        <label className="block text-sm font-medium text-text-primary">
          Custom Instructions
        </label>
        <textarea
          value={agentSettings.customInstructions}
          onChange={(e) => setCustomInstructions(e.target.value)}
          placeholder="e.g., Keep code concise. Always add safety limiters..."
          className="w-full h-24 px-3 py-2 rounded-lg border border-border bg-bg-primary text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-accent/50 resize-none text-sm"
        />
      </div>
    </div>
  );
}
