import { useState } from 'react';
import { Modal } from '../Common/Modal';
import { Spinner } from '../Common/Spinner';

interface NewProjectModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (name: string, description: string) => Promise<void>;
}

export function NewProjectModal({ isOpen, onClose, onSubmit }: NewProjectModalProps) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

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
      await onSubmit(name, description);
      setName('');
      setDescription('');
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
            You can chat with Claude to build out the features
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
