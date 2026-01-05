import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ProjectMeta, CreateProjectInput } from '../types';

interface ProjectState {
  projects: ProjectMeta[];
  activeProject: ProjectMeta | null;
  loading: boolean;
  error: string | null;

  // Actions
  loadProjects: () => Promise<void>;
  createProject: (input: CreateProjectInput) => Promise<ProjectMeta>;
  selectProject: (project: ProjectMeta | null) => void;
  deleteProject: (name: string) => Promise<void>;
}

export const useProjectStore = create<ProjectState>((set) => ({
  projects: [],
  activeProject: null,
  loading: false,
  error: null,

  loadProjects: async () => {
    set({ loading: true, error: null });
    try {
      const projects = await invoke<ProjectMeta[]>('list_projects');
      set({ projects, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  createProject: async (input: CreateProjectInput) => {
    set({ loading: true, error: null });
    try {
      const project = await invoke<ProjectMeta>('create_project', { input });
      set((state) => ({
        projects: [project, ...state.projects],
        activeProject: project,
        loading: false,
      }));
      return project;
    } catch (err) {
      set({ error: String(err), loading: false });
      throw err;
    }
  },

  selectProject: (project) => {
    set({ activeProject: project });
  },

  deleteProject: async (name: string) => {
    set({ loading: true, error: null });
    try {
      await invoke('delete_project', { name });
      set((state) => ({
        projects: state.projects.filter((p) => p.name !== name),
        activeProject: state.activeProject?.name === name ? null : state.activeProject,
        loading: false,
      }));
    } catch (err) {
      set({ error: String(err), loading: false });
      throw err;
    }
  },
}));
