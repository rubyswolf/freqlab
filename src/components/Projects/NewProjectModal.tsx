import { useState } from 'react';
import { Modal } from '../Common/Modal';
import { Spinner } from '../Common/Spinner';
import { useSettingsStore } from '../../stores/settingsStore';
import type { CreateProjectInput, PluginTemplate } from '../../types';

interface NewProjectModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (input: CreateProjectInput) => Promise<void>;
}

export function NewProjectModal({ isOpen, onClose, onSubmit }: NewProjectModalProps) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [template, setTemplate] = useState<PluginTemplate>('effect');
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { vendorName, vendorUrl, vendorEmail } = useSettingsStore();

  const validateName = (value: string): string | null => {
    if (!value) return 'Name is required';
    if (value.length > 50) return 'Name too long (max 50 chars)';
    if (!/^[a-z]/.test(value)) return 'Must start with a lowercase letter';
    if (!/^[a-z][a-z0-9_-]*$/.test(value)) {
      return 'Only lowercase letters, numbers, hyphens, and underscores allowed';
    }
    return null;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const nameError = validateName(name);
    if (nameError) {
      setError(nameError);
      return;
    }

    setError(null);
    setIsSubmitting(true);

    try {
      await onSubmit({
        name,
        description,
        template,
        vendorName: vendorName || 'freqlab',
        vendorUrl: vendorUrl || '',
        vendorEmail: vendorEmail || '',
      });
      setName('');
      setDescription('');
      setTemplate('effect');
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value.toLowerCase().replace(/[^a-z0-9_-]/g, '');
    setName(value);
    setError(null);
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="New Plugin">
      <form onSubmit={handleSubmit} className="space-y-5">
        <div>
          <label htmlFor="name" className="block text-sm font-medium text-text-secondary mb-2">
            Plugin Name
          </label>
          <input
            type="text"
            id="name"
            value={name}
            onChange={handleNameChange}
            placeholder="my-plugin"
            className="w-full px-4 py-2.5 bg-bg-primary border border-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
            autoFocus
          />
          <p className="mt-1.5 text-xs text-text-muted">
            Lowercase letters, numbers, hyphens, and underscores only
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-text-secondary mb-2">
            Plugin Type
          </label>
          <div className="grid grid-cols-2 gap-3">
            <button
              type="button"
              onClick={() => setTemplate('effect')}
              className={`p-3 rounded-xl border-2 transition-all text-left ${
                template === 'effect'
                  ? 'border-accent bg-accent/5'
                  : 'border-border hover:border-text-muted'
              }`}
            >
              <div className="flex items-center gap-2 mb-1">
                <svg className={`w-4 h-4 ${template === 'effect' ? 'text-accent' : 'text-text-muted'}`} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <path strokeLinecap="round" d="M6 4v16M12 4v16M18 4v16" />
                  <rect x="4" y="6" width="4" height="3" rx="1" fill="currentColor" />
                  <rect x="10" y="12" width="4" height="3" rx="1" fill="currentColor" />
                  <rect x="16" y="9" width="4" height="3" rx="1" fill="currentColor" />
                </svg>
                <span className={`text-sm font-medium ${template === 'effect' ? 'text-accent' : 'text-text-primary'}`}>
                  Effect
                </span>
              </div>
              <p className="text-xs text-text-muted">Processes audio (EQ, delay)</p>
            </button>
            <button
              type="button"
              onClick={() => setTemplate('instrument')}
              className={`p-3 rounded-xl border-2 transition-all text-left ${
                template === 'instrument'
                  ? 'border-accent bg-accent/5'
                  : 'border-border hover:border-text-muted'
              }`}
            >
              <div className="flex items-center gap-2 mb-1">
                <svg className={`w-4 h-4 ${template === 'instrument' ? 'text-accent' : 'text-text-muted'}`} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <rect x="2" y="4" width="20" height="16" rx="2" />
                  <path d="M6 4v10M10 4v10M14 4v10M18 4v10" />
                  <rect x="5" y="4" width="2" height="6" fill="currentColor" />
                  <rect x="9" y="4" width="2" height="6" fill="currentColor" />
                  <rect x="13" y="4" width="2" height="6" fill="currentColor" />
                  <rect x="17" y="4" width="2" height="6" fill="currentColor" />
                </svg>
                <span className={`text-sm font-medium ${template === 'instrument' ? 'text-accent' : 'text-text-primary'}`}>
                  Instrument
                </span>
              </div>
              <p className="text-xs text-text-muted">Generates sound from MIDI (synth)</p>
            </button>
          </div>
        </div>

        <div>
          <label htmlFor="description" className="block text-sm font-medium text-text-secondary mb-2">
            Description
          </label>
          <textarea
            id="description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe your plugin idea..."
            rows={3}
            className="w-full px-4 py-2.5 bg-bg-primary border border-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent transition-colors resize-none"
          />
          <p className="mt-1.5 text-xs text-text-muted">
            You can refine and add features through conversation
          </p>
        </div>

        {error && (
          <div className="p-3 rounded-lg bg-error-subtle border border-error/20 text-error text-sm">
            {error}
          </div>
        )}

        <div className="flex gap-3 pt-2">
          <button
            type="button"
            onClick={onClose}
            disabled={isSubmitting}
            className="flex-1 py-2.5 px-4 bg-bg-tertiary hover:bg-bg-elevated text-text-secondary hover:text-text-primary font-medium rounded-xl border border-border transition-all duration-200 disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSubmitting || !name}
            className="flex-1 py-2.5 px-4 bg-accent hover:bg-accent-hover disabled:bg-bg-tertiary disabled:text-text-muted text-white font-medium rounded-xl transition-all duration-200 disabled:cursor-not-allowed hover:shadow-lg hover:shadow-accent/25 disabled:shadow-none flex items-center justify-center gap-2"
          >
            {isSubmitting && <Spinner size="sm" />}
            Create
          </button>
        </div>
      </form>
    </Modal>
  );
}
