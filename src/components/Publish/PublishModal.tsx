import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Modal } from '../Common/Modal';
import { Spinner } from '../Common/Spinner';
import { useSettingsStore } from '../../stores/settingsStore';
import type { ProjectMeta, DawPaths } from '../../types';

interface PublishModalProps {
  isOpen: boolean;
  onClose: () => void;
  project: ProjectMeta;
  onSuccess?: () => void;
}

interface AvailableFormats {
  vst3: boolean;
  clap: boolean;
}

interface DawPublishTarget {
  daw: string;
  vst3_path: string;
  clap_path: string;
}

interface CopiedFile {
  format: string;
  daw: string;
  path: string;
}

interface PublishResult {
  success: boolean;
  copied: CopiedFile[];
  errors: string[];
}

const DAW_LABELS: Record<keyof DawPaths, string> = {
  reaper: 'REAPER',
  ableton: 'Ableton Live',
  flStudio: 'FL Studio',
  logic: 'Logic Pro',
  other: 'Custom Location',
};

export function PublishModal({ isOpen, onClose, project, onSuccess }: PublishModalProps) {
  const { dawPaths } = useSettingsStore();
  const [selectedDaws, setSelectedDaws] = useState<Set<keyof DawPaths>>(new Set());
  const [formats, setFormats] = useState<AvailableFormats | null>(null);
  const [isPublishing, setIsPublishing] = useState(false);
  const [result, setResult] = useState<PublishResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [currentVersion, setCurrentVersion] = useState<number>(1);

  // Check available formats when modal opens
  useEffect(() => {
    if (isOpen) {
      setResult(null);
      setError(null);
      setFormats(null);

      // First get the current version, then check available formats
      invoke<number>('get_current_version', { projectPath: project.path })
        .then((version) => {
          setCurrentVersion(version);
          return invoke<AvailableFormats>('check_available_formats', {
            projectName: project.name,
            version,
          });
        })
        .then(setFormats)
        .catch((err) => setError(String(err)));
    }
  }, [isOpen, project.name, project.path]);

  // Get DAWs that have at least one path configured
  const configuredDaws = Object.entries(dawPaths).filter(
    ([, paths]) => paths.vst3.trim() !== '' || paths.clap.trim() !== ''
  ) as [keyof DawPaths, { vst3: string; clap: string }][];

  // Check if "other" (Custom Location) is configured
  const isOtherConfigured = dawPaths.other.vst3.trim() !== '' || dawPaths.other.clap.trim() !== '';

  const handleToggleDaw = (daw: keyof DawPaths) => {
    setSelectedDaws((prev) => {
      const next = new Set(prev);
      if (next.has(daw)) {
        next.delete(daw);
      } else {
        next.add(daw);
      }
      return next;
    });
  };

  const handleSelectAll = () => {
    setSelectedDaws(new Set(configuredDaws.map(([daw]) => daw)));
  };

  const handleSelectNone = () => {
    setSelectedDaws(new Set());
  };

  const handlePublish = async () => {
    if (selectedDaws.size === 0) return;

    setIsPublishing(true);
    setError(null);
    setResult(null);

    try {
      const targets: DawPublishTarget[] = Array.from(selectedDaws).map((daw) => ({
        daw: DAW_LABELS[daw],
        vst3_path: dawPaths[daw].vst3,
        clap_path: dawPaths[daw].clap,
      }));

      const publishResult = await invoke<PublishResult>('publish_to_daw', {
        projectName: project.name,
        version: currentVersion,
        targets,
      });

      setResult(publishResult);
      if (publishResult.success) {
        onSuccess?.();
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setIsPublishing(false);
    }
  };

  const handleClose = () => {
    setSelectedDaws(new Set());
    setResult(null);
    setError(null);
    onClose();
  };

  const noFormatsAvailable = formats && !formats.vst3 && !formats.clap;
  const noDawsConfigured = configuredDaws.length === 0;

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Publish to DAW">
      <div className="space-y-5">
        {/* Version indicator */}
        <div className="flex items-center gap-2">
          <span className="text-sm text-text-secondary">Publishing version:</span>
          <span className="text-xs px-2 py-0.5 rounded-full bg-accent/20 text-accent font-medium">
            v{currentVersion}
          </span>
        </div>

        {/* Available formats */}
        <div>
          <h4 className="text-sm font-medium text-text-secondary mb-2">Available Formats</h4>
          {!formats ? (
            <div className="flex items-center gap-2 text-text-muted">
              <Spinner size="sm" />
              Checking...
            </div>
          ) : noFormatsAvailable ? (
            <p className="text-sm text-warning">
              No built plugins found. Build the project first.
            </p>
          ) : (
            <div className="flex gap-3">
              {formats.vst3 && (
                <span className="px-3 py-1 rounded-lg bg-accent/20 text-accent text-sm font-medium">
                  VST3
                </span>
              )}
              {formats.clap && (
                <span className="px-3 py-1 rounded-lg bg-accent/20 text-accent text-sm font-medium">
                  CLAP
                </span>
              )}
            </div>
          )}
        </div>

        {/* DAW selection */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-medium text-text-secondary">Select DAWs</h4>
            {configuredDaws.length > 1 && (
              <div className="flex gap-2 text-xs">
                <button
                  onClick={handleSelectAll}
                  className="text-accent hover:text-accent-hover"
                >
                  Select All
                </button>
                <span className="text-text-muted">/</span>
                <button
                  onClick={handleSelectNone}
                  className="text-accent hover:text-accent-hover"
                >
                  None
                </button>
              </div>
            )}
          </div>

          {noDawsConfigured && !isOtherConfigured ? (
            <p className="text-sm text-text-muted">
              No DAW paths configured. Set up paths in Settings &rarr; DAW Paths.
            </p>
          ) : (
            <>
              <div className="flex flex-wrap gap-2">
                {configuredDaws.filter(([daw]) => daw !== 'other').map(([daw]) => (
                  <label
                    key={daw}
                    className={`flex items-center gap-2 px-3 py-2 rounded-lg border cursor-pointer transition-colors ${
                      selectedDaws.has(daw)
                        ? 'bg-accent/10 border-accent/30 text-text-primary'
                        : 'bg-bg-tertiary border-border hover:border-accent/30 text-text-secondary'
                    }`}
                  >
                    <input
                      type="checkbox"
                      checked={selectedDaws.has(daw)}
                      onChange={() => handleToggleDaw(daw)}
                      className="rounded border-border text-accent focus:ring-accent"
                    />
                    <span className="text-sm font-medium">{DAW_LABELS[daw]}</span>
                  </label>
                ))}
                {/* Always show Custom Location option */}
                {isOtherConfigured ? (
                  <label
                    className={`flex items-center gap-2 px-3 py-2 rounded-lg border cursor-pointer transition-colors ${
                      selectedDaws.has('other')
                        ? 'bg-accent/10 border-accent/30 text-text-primary'
                        : 'bg-bg-tertiary border-border hover:border-accent/30 text-text-secondary'
                    }`}
                  >
                    <input
                      type="checkbox"
                      checked={selectedDaws.has('other')}
                      onChange={() => handleToggleDaw('other')}
                      className="rounded border-border text-accent focus:ring-accent"
                    />
                    <span className="text-sm font-medium">{DAW_LABELS.other}</span>
                  </label>
                ) : (
                  <div
                    className="flex items-center gap-2 px-3 py-2 rounded-lg border border-dashed border-border text-text-muted cursor-not-allowed"
                    title="Configure in Settings → DAW Paths"
                  >
                    <span className="text-sm">{DAW_LABELS.other}</span>
                    <span className="text-xs">(not configured)</span>
                  </div>
                )}
              </div>
              <p className="text-xs text-text-muted mt-2">
                Configure paths in Settings &rarr; DAW Paths
              </p>
            </>
          )}
        </div>

        {/* Error */}
        {error && (
          <div className="p-3 rounded-lg bg-error-subtle border border-error/20 text-error text-sm">
            {error}
          </div>
        )}

        {/* Result */}
        {result && (
          <div
            className={`p-3 rounded-lg border text-sm ${
              result.success
                ? 'bg-success/10 border-success/30 text-success'
                : 'bg-warning/10 border-warning/30 text-warning'
            }`}
          >
            {result.success ? (
              <>
                <p className="font-medium mb-1">Published successfully!</p>
                <ul className="text-xs space-y-0.5">
                  {result.copied.map((file, i) => (
                    <li key={i}>
                      {file.format} &rarr; {file.daw}
                    </li>
                  ))}
                </ul>
              </>
            ) : (
              <>
                <p className="font-medium mb-1">Publish completed with errors:</p>
                <ul className="text-xs space-y-0.5">
                  {result.errors.map((err, i) => (
                    <li key={i}>{err}</li>
                  ))}
                </ul>
              </>
            )}
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-3 pt-2">
          <button
            type="button"
            onClick={handleClose}
            className="flex-1 py-2.5 px-4 bg-bg-tertiary hover:bg-bg-elevated text-text-secondary hover:text-text-primary font-medium rounded-xl border border-border transition-all duration-200"
          >
            {result ? 'Close' : 'Cancel'}
          </button>
          {!result && (
            <button
              type="button"
              onClick={handlePublish}
              disabled={isPublishing || noFormatsAvailable || selectedDaws.size === 0}
              className="flex-1 py-2.5 px-4 bg-accent hover:bg-accent-hover disabled:bg-bg-tertiary disabled:text-text-muted text-white font-medium rounded-xl transition-all duration-200 disabled:cursor-not-allowed hover:shadow-lg hover:shadow-accent/25 disabled:shadow-none flex items-center justify-center gap-2"
            >
              {isPublishing && <Spinner size="sm" />}
              Publish
            </button>
          )}
        </div>
      </div>
    </Modal>
  );
}
