import { useEffect } from 'react';
import { usePrerequisites } from '../../hooks/usePrerequisites';
import { Spinner } from '../Common/Spinner';
import type { CheckResult } from '../../types';

interface CheckItemProps {
  label: string;
  result: CheckResult | undefined;
  isLoading: boolean;
}

function CheckItem({ label, result, isLoading }: CheckItemProps) {
  const getStatusStyles = () => {
    if (isLoading || !result) {
      return {
        bg: 'bg-bg-elevated',
        border: 'border-border',
        iconBg: 'bg-bg-tertiary',
        iconColor: 'text-text-muted',
      };
    }

    switch (result.status) {
      case 'installed':
        return {
          bg: 'bg-success-subtle',
          border: 'border-success/20',
          iconBg: 'bg-success/20',
          iconColor: 'text-success',
        };
      case 'needsconfig':
        return {
          bg: 'bg-warning-subtle',
          border: 'border-warning/20',
          iconBg: 'bg-warning/20',
          iconColor: 'text-warning',
        };
      default:
        return {
          bg: 'bg-error-subtle',
          border: 'border-error/20',
          iconBg: 'bg-error/20',
          iconColor: 'text-error',
        };
    }
  };

  const styles = getStatusStyles();

  const StatusIcon = () => {
    if (isLoading || !result) {
      return <Spinner size="sm" className="text-text-muted" />;
    }

    switch (result.status) {
      case 'installed':
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
          </svg>
        );
      case 'needsconfig':
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        );
      default:
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        );
    }
  };

  return (
    <div className={`p-4 rounded-xl ${styles.bg} border ${styles.border} transition-all duration-300`}>
      <div className="flex items-center gap-3">
        <div className={`w-8 h-8 rounded-lg ${styles.iconBg} ${styles.iconColor} flex items-center justify-center flex-shrink-0`}>
          <StatusIcon />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-text-primary">{label}</div>
          {result && (
            <div className="text-xs text-text-muted mt-0.5 truncate">
              {result.version || result.message || ''}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

interface PrerequisitesCheckProps {
  onComplete: () => void;
}

export function PrerequisitesCheck({ onComplete }: PrerequisitesCheckProps) {
  const { status, loading, check, allInstalled } = usePrerequisites();

  useEffect(() => {
    check();
  }, [check]);

  const items = [
    { key: 'xcode_cli', label: 'Xcode Command Line Tools', result: status?.xcode_cli },
    { key: 'rust', label: 'Rust & Cargo', result: status?.rust },
    { key: 'claude_cli', label: 'Claude Code CLI', result: status?.claude_cli },
    { key: 'claude_auth', label: 'Claude Authentication', result: status?.claude_auth },
  ];

  const hasIssues = status && !allInstalled;

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Header */}
      <div className="text-center">
        <div className="w-12 h-12 mx-auto rounded-xl bg-accent-subtle flex items-center justify-center mb-4">
          <svg className="w-6 h-6 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <h2 className="text-xl font-semibold text-text-primary">System Requirements</h2>
        <p className="text-sm text-text-secondary mt-1">
          Checking your development environment
        </p>
      </div>

      {/* Checklist */}
      <div className="space-y-2">
        {items.map((item) => (
          <CheckItem
            key={item.key}
            label={item.label}
            result={item.result}
            isLoading={loading}
          />
        ))}
      </div>

      {/* Detailed help sections */}
      {hasIssues && (
        <div className="space-y-3">
          {status?.xcode_cli.status !== 'installed' && (
            <div className="p-4 rounded-xl bg-bg-tertiary border border-border">
              <div className="flex items-start gap-3">
                <div className="w-8 h-8 rounded-lg bg-error/20 flex items-center justify-center flex-shrink-0">
                  <svg className="w-4 h-4 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M11.42 15.17L17.25 21A2.652 2.652 0 0021 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655 5.653a2.548 2.548 0 11-3.586-3.586l6.837-5.63m5.108-.233c.55-.164 1.163-.188 1.743-.14a4.5 4.5 0 004.486-6.336l-3.276 3.277a3.004 3.004 0 01-2.25-2.25l3.276-3.276a4.5 4.5 0 00-6.336 4.486c.091 1.076-.071 2.264-.904 2.95l-.102.085m-1.745 1.437L5.909 7.5H4.5L2.25 3.75l1.5-1.5L7.5 4.5v1.409l4.26 4.26m-1.745 1.437l1.745-1.437m6.615 8.206L15.75 15.75M4.867 19.125h.008v.008h-.008v-.008z" />
                  </svg>
                </div>
                <div className="flex-1">
                  <h3 className="text-sm font-medium text-text-primary mb-1">Xcode Command Line Tools</h3>
                  <p className="text-xs text-text-muted mb-3">Required for compiling native code on macOS</p>
                  <div className="space-y-2">
                    <p className="text-xs text-text-secondary">Open Terminal and run:</p>
                    <code className="block px-3 py-2 bg-bg-primary rounded-lg text-accent text-xs font-mono">
                      xcode-select --install
                    </code>
                    <p className="text-xs text-text-muted">A dialog will appear. Click "Install" and wait for completion (~5 min).</p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {status?.rust.status !== 'installed' && (
            <div className="p-4 rounded-xl bg-bg-tertiary border border-border">
              <div className="flex items-start gap-3">
                <div className="w-8 h-8 rounded-lg bg-error/20 flex items-center justify-center flex-shrink-0">
                  <svg className="w-4 h-4 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5" />
                  </svg>
                </div>
                <div className="flex-1">
                  <h3 className="text-sm font-medium text-text-primary mb-1">Rust & Cargo</h3>
                  <p className="text-xs text-text-muted mb-3">The programming language used for audio plugin development</p>
                  <div className="space-y-2">
                    <p className="text-xs text-text-secondary">Open Terminal and run:</p>
                    <code className="block px-3 py-2 bg-bg-primary rounded-lg text-accent text-xs font-mono">
                      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
                    </code>
                    <p className="text-xs text-text-muted">Follow the prompts (press Enter for defaults). Then restart your terminal or run <code className="text-accent">source ~/.cargo/env</code></p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {status?.claude_cli.status !== 'installed' && (
            <div className="p-4 rounded-xl bg-bg-tertiary border border-border">
              <div className="flex items-start gap-3">
                <div className="w-8 h-8 rounded-lg bg-error/20 flex items-center justify-center flex-shrink-0">
                  <svg className="w-4 h-4 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M8.25 3v1.5M4.5 8.25H3m18 0h-1.5M4.5 12H3m18 0h-1.5m-15 3.75H3m18 0h-1.5M8.25 19.5V21M12 3v1.5m0 15V21m3.75-18v1.5m0 15V21m-9-1.5h10.5a2.25 2.25 0 002.25-2.25V6.75a2.25 2.25 0 00-2.25-2.25H6.75A2.25 2.25 0 004.5 6.75v10.5a2.25 2.25 0 002.25 2.25zm.75-12h9v9h-9v-9z" />
                  </svg>
                </div>
                <div className="flex-1">
                  <h3 className="text-sm font-medium text-text-primary mb-1">Claude Code CLI</h3>
                  <p className="text-xs text-text-muted mb-3">The AI assistant that helps write your plugin code</p>
                  <div className="space-y-2">
                    <p className="text-xs text-text-secondary">Requires Node.js 18+. Install with npm:</p>
                    <code className="block px-3 py-2 bg-bg-primary rounded-lg text-accent text-xs font-mono">
                      npm install -g @anthropic-ai/claude-code
                    </code>
                    <p className="text-xs text-text-muted">
                      Don't have Node.js? Get it from{' '}
                      <a href="https://nodejs.org" target="_blank" rel="noopener noreferrer" className="text-accent hover:underline">nodejs.org</a>
                      {' '}(LTS version recommended)
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {status?.claude_auth.status === 'needsconfig' && (
            <div className="p-4 rounded-xl bg-bg-tertiary border border-border">
              <div className="flex items-start gap-3">
                <div className="w-8 h-8 rounded-lg bg-warning/20 flex items-center justify-center flex-shrink-0">
                  <svg className="w-4 h-4 text-warning" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 5.25a3 3 0 013 3m3 0a6 6 0 01-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1121.75 8.25z" />
                  </svg>
                </div>
                <div className="flex-1">
                  <h3 className="text-sm font-medium text-text-primary mb-1">Claude Authentication</h3>
                  <p className="text-xs text-text-muted mb-3">Sign in to your Anthropic account to use Claude</p>
                  <div className="space-y-2">
                    <p className="text-xs text-text-secondary">Open Terminal and run:</p>
                    <code className="block px-3 py-2 bg-bg-primary rounded-lg text-accent text-xs font-mono">
                      claude login
                    </code>
                    <p className="text-xs text-text-muted">This will open your browser to sign in. Once authenticated, return here and click "Recheck".</p>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Actions */}
      <div className="flex gap-3">
        <button
          onClick={check}
          disabled={loading}
          className="flex-1 py-2.5 px-4 bg-bg-tertiary hover:bg-bg-elevated text-text-secondary hover:text-text-primary font-medium rounded-xl border border-border transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {loading && <Spinner size="sm" />}
          Recheck
        </button>

        <button
          onClick={onComplete}
          disabled={!allInstalled}
          className="flex-1 py-2.5 px-4 bg-accent hover:bg-accent-hover disabled:bg-bg-tertiary disabled:text-text-muted text-white font-medium rounded-xl transition-all duration-200 disabled:cursor-not-allowed hover:shadow-lg hover:shadow-accent/25 disabled:shadow-none"
        >
          {allInstalled ? 'Continue' : 'Requirements needed'}
        </button>
      </div>
    </div>
  );
}
