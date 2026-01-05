import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export function DevSettings() {
  const [showConfirm, setShowConfirm] = useState(false);
  const [showClearLogConfirm, setShowClearLogConfirm] = useState(false);
  const [logPath, setLogPath] = useState<string>('');
  const [logSize, setLogSize] = useState<number>(0);
  const [logContent, setLogContent] = useState<string | null>(null);

  // Load log file info on mount
  useEffect(() => {
    async function loadLogInfo() {
      try {
        const path = await invoke<string>('get_log_file_path');
        const size = await invoke<number>('get_log_file_size');
        setLogPath(path);
        setLogSize(size);
      } catch (err) {
        console.error('Failed to load log info:', err);
      }
    }
    loadLogInfo();
  }, []);

  const handleReset = () => {
    // Clear all localStorage
    localStorage.clear();
    // Reload the app
    window.location.reload();
  };

  const handleViewLog = async () => {
    try {
      const content = await invoke<string>('read_log_file');
      setLogContent(content);
    } catch (err) {
      setLogContent(`Error reading log: ${err}`);
    }
  };

  const handleClearLog = async () => {
    try {
      await invoke('clear_log_file');
      setLogSize(0);
      setLogContent(null);
      setShowClearLogConfirm(false);
    } catch (err) {
      console.error('Failed to clear log:', err);
    }
  };

  const handleCopyLog = async () => {
    if (logContent) {
      await navigator.clipboard.writeText(logContent);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-text-primary mb-1">Developer Tools</h3>
        <p className="text-sm text-text-muted">
          Debugging utilities and log file management.
        </p>
      </div>

      {/* Log File Section */}
      <div className="border border-border rounded-lg p-4">
        <h4 className="font-medium text-text-primary mb-2">Log File</h4>
        <p className="text-sm text-text-muted mb-3">
          Application logs for debugging issues. Share this file when reporting bugs.
        </p>

        <div className="space-y-3">
          <div className="flex items-center justify-between text-sm">
            <span className="text-text-secondary">Location:</span>
            <span className="text-text-muted font-mono text-xs truncate max-w-[250px]" title={logPath}>
              {logPath}
            </span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-text-secondary">Size:</span>
            <span className="text-text-muted">{formatBytes(logSize)}</span>
          </div>

          <div className="flex gap-2 pt-2">
            <button
              onClick={handleViewLog}
              className="px-3 py-1.5 text-sm font-medium text-accent bg-accent/10 hover:bg-accent/20 rounded-lg border border-accent/30 transition-colors"
            >
              View Log
            </button>
            {!showClearLogConfirm ? (
              <button
                onClick={() => setShowClearLogConfirm(true)}
                disabled={logSize === 0}
                className="px-3 py-1.5 text-sm font-medium text-text-secondary bg-bg-tertiary hover:bg-bg-elevated disabled:opacity-50 disabled:cursor-not-allowed rounded-lg border border-border transition-colors"
              >
                Clear Log
              </button>
            ) : (
              <div className="flex items-center gap-2">
                <button
                  onClick={handleClearLog}
                  className="px-3 py-1.5 text-sm font-medium text-error bg-error/10 hover:bg-error/20 rounded-lg border border-error/30 transition-colors"
                >
                  Confirm Clear
                </button>
                <button
                  onClick={() => setShowClearLogConfirm(false)}
                  className="px-3 py-1.5 text-sm font-medium text-text-secondary bg-bg-tertiary hover:bg-bg-elevated rounded-lg border border-border transition-colors"
                >
                  Cancel
                </button>
              </div>
            )}
          </div>
        </div>

        {/* Log Content Viewer */}
        {logContent !== null && (
          <div className="mt-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-text-secondary">Log Contents</span>
              <div className="flex gap-2">
                <button
                  onClick={handleCopyLog}
                  className="text-xs text-accent hover:text-accent-hover"
                >
                  Copy to Clipboard
                </button>
                <button
                  onClick={() => setLogContent(null)}
                  className="text-xs text-text-muted hover:text-text-secondary"
                >
                  Close
                </button>
              </div>
            </div>
            <pre className="bg-bg-primary border border-border rounded-lg p-3 text-xs text-text-muted font-mono max-h-64 overflow-auto whitespace-pre-wrap">
              {logContent || 'Log file is empty.'}
            </pre>
          </div>
        )}
      </div>

      {/* Reset App State Section */}
      <div className="border border-border rounded-lg p-4">
        <h4 className="font-medium text-text-primary mb-2">Reset App State</h4>
        <p className="text-sm text-text-muted mb-4">
          Clears all settings and shows the first-run experience again. Projects are not deleted.
        </p>

        {!showConfirm ? (
          <button
            onClick={() => setShowConfirm(true)}
            className="px-4 py-2 text-sm font-medium text-warning bg-warning/10 hover:bg-warning/20 rounded-lg border border-warning/30 transition-colors"
          >
            Reset App State
          </button>
        ) : (
          <div className="flex items-center gap-3">
            <span className="text-sm text-text-secondary">Are you sure?</span>
            <button
              onClick={handleReset}
              className="px-4 py-2 text-sm font-medium text-error bg-error/10 hover:bg-error/20 rounded-lg border border-error/30 transition-colors"
            >
              Yes, Reset
            </button>
            <button
              onClick={() => setShowConfirm(false)}
              className="px-4 py-2 text-sm font-medium text-text-secondary bg-bg-tertiary hover:bg-bg-elevated rounded-lg border border-border transition-colors"
            >
              Cancel
            </button>
          </div>
        )}
      </div>

      <div className="bg-bg-tertiary rounded-lg p-4">
        <div className="flex items-start gap-3">
          <svg className="w-5 h-5 text-text-muted flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div className="text-sm text-text-muted">
            <p>
              Reset clears settings like theme, branding, and DAW paths. Your plugin projects in ~/VSTWorkshop/projects remain untouched.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
