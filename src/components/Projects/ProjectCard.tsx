import type { ProjectMeta } from '../../types';

interface ProjectCardProps {
  project: ProjectMeta;
  isActive: boolean;
  onClick: () => void;
}

export function ProjectCard({ project, isActive, onClick }: ProjectCardProps) {
  const timeAgo = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    if (minutes > 0) return `${minutes}m ago`;
    return 'Just now';
  };

  return (
    <button
      onClick={onClick}
      className={`w-full text-left p-3 rounded-xl transition-all duration-200 ${
        isActive
          ? 'bg-accent-subtle border border-accent/30'
          : 'hover:bg-bg-tertiary border border-transparent'
      }`}
    >
      <div className="flex items-start gap-3">
        <div className={`w-9 h-9 rounded-lg flex items-center justify-center text-lg flex-shrink-0 ${
          isActive ? 'bg-accent/20' : 'bg-bg-elevated'
        }`}>
          <svg className={`w-5 h-5 ${isActive ? 'text-accent' : 'text-text-muted'}`} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <div className={`font-medium truncate ${isActive ? 'text-accent' : 'text-text-primary'}`}>
            {project.name}
          </div>
          <div className="text-xs text-text-muted truncate mt-0.5">
            {project.description || 'No description'}
          </div>
          <div className="text-xs text-text-muted mt-1">
            {timeAgo(project.updated_at)}
          </div>
        </div>
      </div>
    </button>
  );
}
