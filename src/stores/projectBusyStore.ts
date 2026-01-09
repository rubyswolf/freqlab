import { create } from 'zustand';

interface ProjectBusyState {
  // Track which projects are busy with Claude (supports multiple concurrent)
  claudeBusyPaths: Set<string>;
  // Track when each project started being busy (for elapsed time display)
  claudeStartTimes: Map<string, number>;
  setClaudeBusy: (path: string) => void;
  clearClaudeBusy: (path: string) => void;
  isClaudeBusy: (path: string) => boolean;
  getClaudeBusyPaths: () => string[];
  getClaudeStartTime: (path: string) => number | undefined;

  // Track which project is building (only one at a time)
  buildingPath: string | null;
  setBuildingPath: (path: string | null) => void;
  clearBuildingIfMatch: (path: string) => void;

  // Check if a specific project is busy (either Claude or building)
  isProjectBusy: (path: string) => boolean;

  // Check if ANY project is busy
  isAnyBusy: () => boolean;

  // Check if any project (other than the given one) has Claude busy
  hasOtherClaudeBusy: (currentPath: string) => boolean;
}

export const useProjectBusyStore = create<ProjectBusyState>((set, get) => ({
  claudeBusyPaths: new Set<string>(),
  claudeStartTimes: new Map<string, number>(),

  setClaudeBusy: (path) => set((state) => {
    const newPaths = new Set(state.claudeBusyPaths);
    newPaths.add(path);
    const newStartTimes = new Map(state.claudeStartTimes);
    if (!newStartTimes.has(path)) {
      newStartTimes.set(path, Date.now());
    }
    return { claudeBusyPaths: newPaths, claudeStartTimes: newStartTimes };
  }),

  clearClaudeBusy: (path) => set((state) => {
    const newPaths = new Set(state.claudeBusyPaths);
    newPaths.delete(path);
    const newStartTimes = new Map(state.claudeStartTimes);
    newStartTimes.delete(path);
    return { claudeBusyPaths: newPaths, claudeStartTimes: newStartTimes };
  }),

  isClaudeBusy: (path) => {
    return get().claudeBusyPaths.has(path);
  },

  getClaudeBusyPaths: () => {
    return Array.from(get().claudeBusyPaths);
  },

  getClaudeStartTime: (path) => {
    return get().claudeStartTimes.get(path);
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
    return state.claudeBusyPaths.has(path) || state.buildingPath === path;
  },

  isAnyBusy: () => {
    const state = get();
    return state.claudeBusyPaths.size > 0 || state.buildingPath !== null;
  },

  hasOtherClaudeBusy: (currentPath) => {
    const state = get();
    for (const path of state.claudeBusyPaths) {
      if (path !== currentPath) {
        return true;
      }
    }
    return false;
  },
}));
