import { create } from 'zustand';

interface ProjectBusyState {
  // Track which project is busy with Claude
  claudeBusyPath: string | null;
  setClaudeBusy: (path: string | null) => void;
  // Clear Claude busy only if it matches the given path (prevents race conditions)
  clearClaudeBusyIfMatch: (path: string) => void;

  // Track which project is building
  buildingPath: string | null;
  setBuildingPath: (path: string | null) => void;
  // Clear build busy only if it matches the given path
  clearBuildingIfMatch: (path: string) => void;

  // Check if a specific project is busy (either Claude or building)
  isProjectBusy: (path: string) => boolean;

  // Check if ANY project is busy
  isAnyBusy: () => boolean;
}

export const useProjectBusyStore = create<ProjectBusyState>((set, get) => ({
  claudeBusyPath: null,
  setClaudeBusy: (path) => set({ claudeBusyPath: path }),
  clearClaudeBusyIfMatch: (path) => {
    const state = get();
    if (state.claudeBusyPath === path) {
      set({ claudeBusyPath: null });
    }
  },

  buildingPath: null,
  setBuildingPath: (path) => set({ buildingPath: path }),
  clearBuildingIfMatch: (path) => {
    const state = get();
    if (state.buildingPath === path) {
      set({ buildingPath: null });
    }
  },

  isProjectBusy: (path) => {
    const state = get();
    return state.claudeBusyPath === path || state.buildingPath === path;
  },

  isAnyBusy: () => {
    const state = get();
    return state.claudeBusyPath !== null || state.buildingPath !== null;
  },
}));
