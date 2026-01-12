import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../stores/settingsStore';
import { Spinner } from '../Common/Spinner';
import type { AgentProviderType, ProviderStatus } from '../../types';

interface ProviderOption {
  id: AgentProviderType;
  name: string;
  description: string;
  features: string[];
  setupLink?: string;
  setupText?: string;
}

const providerOptions: ProviderOption[] = [
  {
    id: 'claude',
    name: 'Claude Code',
    description: "Anthropic's official CLI",
    features: [
      'Official Anthropic support',
      'Opus, Sonnet, and Haiku models',
      'Requires Claude Pro or Max subscription',
    ],
    setupLink: 'https://docs.anthropic.com/en/docs/claude-code',
    setupText: 'Setup guide',
  },
  {
    id: 'opencode',
    name: 'OpenCode',
    description: '75+ LLM providers',
    features: [
      'OpenAI, Google, Anthropic, and more',
      'Local models via Ollama',
      'Bring your own API keys',
    ],
    setupLink: 'https://github.com/opencode-ai/opencode',
    setupText: 'GitHub',
  },
];

interface ProviderSelectionProps {
  onComplete: () => void;
  onBack: () => void;
}

export function ProviderSelection({ onComplete, onBack }: ProviderSelectionProps) {
  const setDefaultProvider = useSettingsStore((s) => s.setDefaultProvider);
  const currentProvider = useSettingsStore((s) => s.agentSettings.defaultProvider);

  const [selectedProvider, setSelectedProvider] = useState<AgentProviderType>(currentProvider || 'claude');
  const [providerStatus, setProviderStatus] = useState<Record<AgentProviderType, ProviderStatus>>({
    claude: { installed: false, authenticated: false },
    opencode: { installed: false, authenticated: false },
  });
  const [checking, setChecking] = useState(true);

  // Check provider status on mount
  useEffect(() => {
    async function checkProviders() {
      setChecking(true);
      try {
        const status = await invoke<Record<AgentProviderType, ProviderStatus>>('check_agent_providers');
        setProviderStatus(status);

        // Auto-select a provider based on what's available
        if (!currentProvider) {
          if (status.claude.installed && status.claude.authenticated) {
            setSelectedProvider('claude');
          } else if (status.opencode.installed && status.opencode.authenticated) {
            setSelectedProvider('opencode');
          }
        }
      } catch (e) {
        console.error('Failed to check providers:', e);
      } finally {
        setChecking(false);
      }
    }
    checkProviders();
  }, [currentProvider]);

  const handleContinue = () => {
    setDefaultProvider(selectedProvider);
    onComplete();
  };

  const selectedStatus = providerStatus[selectedProvider];
  // Require both installed AND authenticated to continue
  const canContinue = selectedStatus?.installed && selectedStatus?.authenticated;

  return (
    <div className="space-y-4 animate-fade-in">
      {/* Header */}
      <div className="text-center">
        <div className="w-10 h-10 mx-auto rounded-lg bg-accent-subtle flex items-center justify-center mb-2">
          <svg className="w-5 h-5 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 5a3 3 0 1 0-5.997.125 4 4 0 0 0-2.526 5.77 4 4 0 0 0 .556 6.588A4 4 0 1 0 12 18Z" />
            <path d="M12 5a3 3 0 1 1 5.997.125 4 4 0 0 1 2.526 5.77 4 4 0 0 1-.556 6.588A4 4 0 1 1 12 18Z" />
            <path d="M15 13a4.5 4.5 0 0 1-3-4 4.5 4.5 0 0 1-3 4" />
          </svg>
        </div>
        <h2 className="text-lg font-semibold text-text-primary">Choose Your Agent</h2>
        <p className="text-xs text-text-secondary mt-0.5">Select which coding agent to use</p>
      </div>

      {/* Provider cards */}
      <div className="space-y-2">
        {providerOptions.map((provider) => {
          const status = providerStatus[provider.id];
          const isSelected = selectedProvider === provider.id;
          const isReady = status?.installed && status?.authenticated;
          const isInstalled = status?.installed;

          return (
            <button
              key={provider.id}
              onClick={() => setSelectedProvider(provider.id)}
              className={`w-full p-3 rounded-lg border text-left transition-all duration-200 ${
                isSelected
                  ? 'bg-accent/10 border-accent/30 ring-1 ring-accent/20'
                  : 'bg-bg-elevated border-border hover:border-accent/20'
              }`}
            >
              <div className="flex items-start gap-3">
                {/* Radio indicator */}
                <div className={`w-4 h-4 mt-0.5 rounded-full border-2 flex items-center justify-center flex-shrink-0 transition-colors ${
                  isSelected ? 'border-accent' : 'border-text-muted'
                }`}>
                  {isSelected && (
                    <div className="w-2 h-2 rounded-full bg-accent" />
                  )}
                </div>

                <div className="flex-1 min-w-0">
                  {/* Title and status */}
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-text-primary">{provider.name}</span>
                    {checking ? (
                      <Spinner size="xs" className="text-text-muted" />
                    ) : (
                      <span className={`text-[10px] px-1.5 py-0.5 rounded-full ${
                        isReady
                          ? 'bg-success/20 text-success'
                          : isInstalled
                            ? 'bg-warning/20 text-warning'
                            : 'bg-text-muted/20 text-text-muted'
                      }`}>
                        {isReady ? 'Ready' : isInstalled ? 'Needs auth' : 'Not installed'}
                      </span>
                    )}
                  </div>

                  {/* Description */}
                  <p className="text-xs text-text-muted mt-0.5">{provider.description}</p>

                  {/* Features */}
                  <ul className="mt-2 space-y-0.5">
                    {provider.features.map((feature, i) => (
                      <li key={i} className="text-[11px] text-text-secondary flex items-center gap-1.5">
                        <svg className="w-3 h-3 text-accent flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                          <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                        </svg>
                        {feature}
                      </li>
                    ))}
                  </ul>

                  {/* Setup link if not installed */}
                  {!isInstalled && provider.setupLink && (
                    <a
                      href={provider.setupLink}
                      target="_blank"
                      rel="noopener noreferrer"
                      onClick={(e) => e.stopPropagation()}
                      className="inline-flex items-center gap-1 mt-2 text-[11px] text-accent hover:underline"
                    >
                      {provider.setupText}
                      <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                      </svg>
                    </a>
                  )}
                </div>
              </div>
            </button>
          );
        })}
      </div>

      {/* Status message */}
      {!checking && !canContinue && (
        <div className="px-3 py-2 rounded-lg bg-warning/10 border border-warning/20">
          <p className="text-xs text-text-secondary">
            <span className="font-medium text-warning">
              {selectedStatus?.installed ? 'Authentication required' : 'Installation required'}
            </span>
            {' - '}
            {selectedStatus?.installed
              ? `Run the ${selectedProvider === 'claude' ? 'Claude' : 'OpenCode'} CLI to sign in.`
              : `Install ${providerOptions.find(p => p.id === selectedProvider)?.name} to continue.`
            }
          </p>
        </div>
      )}

      {/* Note about changing later */}
      <p className="text-center text-[11px] text-text-muted">
        You can change this anytime in Settings &gt; Agent
      </p>

      {/* Actions */}
      <div className="flex gap-2">
        <button
          onClick={onBack}
          className="flex-1 py-2 px-3 text-sm bg-bg-tertiary hover:bg-bg-elevated text-text-secondary hover:text-text-primary font-medium rounded-lg border border-border transition-all duration-200"
        >
          Back
        </button>

        <button
          onClick={handleContinue}
          disabled={!canContinue}
          className="flex-1 py-2 px-3 text-sm bg-accent hover:bg-accent-hover disabled:bg-bg-tertiary disabled:text-text-muted text-white font-medium rounded-lg transition-all duration-200 disabled:cursor-not-allowed hover:shadow-lg hover:shadow-accent/25 disabled:shadow-none"
        >
          {canContinue ? 'Continue' : 'Agent required'}
        </button>
      </div>
    </div>
  );
}
