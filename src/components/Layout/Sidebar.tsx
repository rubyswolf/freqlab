import { ProjectList } from '../Projects';

interface SidebarProps {
  onNewPlugin: () => void;
}

export function Sidebar({ onNewPlugin }: SidebarProps) {
  return (
    <aside className="w-64 bg-bg-secondary border-r border-border flex flex-col">
      {/* New Plugin Button */}
      <div className="p-4">
        <button
          onClick={onNewPlugin}
          className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-accent hover:bg-accent-hover text-white font-medium rounded-xl transition-all duration-200 hover:shadow-lg hover:shadow-accent/25"
        >
          <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
          <span>New Plugin</span>
        </button>
      </div>

      {/* Projects Label */}
      <div className="px-4 py-2">
        <span className="text-xs font-medium text-text-muted uppercase tracking-wider">Projects</span>
      </div>

      {/* Project List */}
      <div className="flex-1 overflow-y-auto px-3 pb-3">
        <ProjectList />
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-border">
        <div className="flex items-center justify-between text-xs">
          <span className="text-text-muted">freqlab</span>
          <span className="text-text-muted px-2 py-0.5 bg-bg-tertiary rounded">v0.1.0</span>
        </div>
      </div>
    </aside>
  );
}
