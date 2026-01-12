import { create } from 'zustand';

interface ProjectBusyState {
  // Track which projects are busy with the agent (supports multiple concurrent)
  agentBusyPaths: Set<string>;
  // Track when each project started being busy (for elapsed time display)
  agentStartTimes: Map<string, number>;
  setAgentBusy: (path: string) => void;
  clearAgentBusy: (path: string) => void;
  isAgentBusy: (path: string) => boolean;
  getAgentBusyPaths: () => string[];
  getAgentStartTime: (path: string) => number | undefined;

  // Track which project is building (only one at a time)
  buildingPath: string | null;
  setBuildingPath: (path: string | null) => void;
  clearBuildingIfMatch: (path: string) => void;

  // Check if a specific project is busy (either agent or building)
  isProjectBusy: (path: string) => boolean;

  // Check if ANY project is busy
  isAnyBusy: () => boolean;

  // Check if any project (other than the given one) has agent busy
  hasOtherAgentBusy: (currentPath: string) => boolean;
}

export const useProjectBusyStore = create<ProjectBusyState>((set, get) => ({
  agentBusyPaths: new Set<string>(),
  agentStartTimes: new Map<string, number>(),

  setAgentBusy: (path) => set((state) => {
    const newPaths = new Set(state.agentBusyPaths);
    newPaths.add(path);
    const newStartTimes = new Map(state.agentStartTimes);
    if (!newStartTimes.has(path)) {
      newStartTimes.set(path, Date.now());
    }
    return { agentBusyPaths: newPaths, agentStartTimes: newStartTimes };
  }),

  clearAgentBusy: (path) => set((state) => {
    const newPaths = new Set(state.agentBusyPaths);
    newPaths.delete(path);
    const newStartTimes = new Map(state.agentStartTimes);
    newStartTimes.delete(path);
    return { agentBusyPaths: newPaths, agentStartTimes: newStartTimes };
  }),

  isAgentBusy: (path) => {
    return get().agentBusyPaths.has(path);
  },

  getAgentBusyPaths: () => {
    return Array.from(get().agentBusyPaths);
  },

  getAgentStartTime: (path) => {
    return get().agentStartTimes.get(path);
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
    return state.agentBusyPaths.has(path) || state.buildingPath === path;
  },

  isAnyBusy: () => {
    const state = get();
    return state.agentBusyPaths.size > 0 || state.buildingPath !== null;
  },

  hasOtherAgentBusy: (currentPath) => {
    const state = get();
    for (const path of state.agentBusyPaths) {
      if (path !== currentPath) {
        return true;
      }
    }
    return false;
  },
}));
