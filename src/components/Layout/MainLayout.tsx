import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Header } from './Header';
import { Sidebar } from './Sidebar';
import { OutputPanel } from './OutputPanel';
import { NewProjectModal } from '../Projects';
import { ChatPanel } from '../Chat';
import { useProjectStore } from '../../stores/projectStore';

export function MainLayout() {
  const [isNewProjectModalOpen, setIsNewProjectModalOpen] = useState(false);
  const { activeProject, createProject } = useProjectStore();

  const handleCreateProject = async (name: string, description: string) => {
    await createProject({ name, description });
  };

  const handleOpenFolder = async () => {
    if (activeProject?.path) {
      await invoke('open_project_folder', { path: activeProject.path });
    }
  };

  return (
    <div className="h-screen flex flex-col bg-bg-primary">
      <Header title={activeProject?.name || 'freqlab'} />
      <div className="flex-1 flex overflow-hidden">
        <Sidebar onNewPlugin={() => setIsNewProjectModalOpen(true)} />
        <main className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-auto p-6">
            {activeProject ? (
              <div className="h-full flex flex-col">
                <div className="mb-4 flex items-start justify-between">
                  <div>
                    <h2 className="text-xl font-semibold text-text-primary">{activeProject.name}</h2>
                    <p className="text-sm text-text-muted mt-1">{activeProject.description || 'No description'}</p>
                  </div>
                  <button
                    onClick={handleOpenFolder}
                    className="flex items-center gap-2 px-3 py-1.5 text-sm text-text-secondary hover:text-text-primary bg-bg-tertiary hover:bg-bg-elevated rounded-lg border border-border transition-colors"
                    title="Open project folder"
                  >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
                    </svg>
                    Open Folder
                  </button>
                </div>
                <div className="flex-1 min-h-0">
                  <ChatPanel project={activeProject} />
                </div>
              </div>
            ) : (
              <div className="h-full flex items-center justify-center">
                <div className="text-center">
                  <div className="w-20 h-20 mx-auto rounded-2xl bg-accent-subtle flex items-center justify-center mb-6">
                    <svg className="w-10 h-10 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
                    </svg>
                  </div>
                  <h2 className="text-2xl font-semibold text-text-primary mb-2">Welcome to freqlab</h2>
                  <p className="text-text-secondary mb-6 max-w-md">
                    Create VST plugins by describing what you want. Click "New Plugin" to get started.
                  </p>
                  <button
                    onClick={() => setIsNewProjectModalOpen(true)}
                    className="px-6 py-3 bg-accent hover:bg-accent-hover text-white font-medium rounded-xl transition-all duration-200 hover:shadow-lg hover:shadow-accent/25"
                  >
                    Create Your First Plugin
                  </button>
                </div>
              </div>
            )}
          </div>
          <OutputPanel />
        </main>
      </div>

      <NewProjectModal
        isOpen={isNewProjectModalOpen}
        onClose={() => setIsNewProjectModalOpen(false)}
        onSubmit={handleCreateProject}
      />
    </div>
  );
}
