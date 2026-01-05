import { create } from 'zustand';

interface OutputState {
  lines: string[];
  isActive: boolean;

  // Actions
  addLine: (line: string) => void;
  addLines: (lines: string[]) => void;
  clear: () => void;
  setActive: (active: boolean) => void;
}

export const useOutputStore = create<OutputState>((set) => ({
  lines: [],
  isActive: false,

  addLine: (line: string) => {
    set((state) => ({
      lines: [...state.lines, line].slice(-500), // Keep last 500 lines
    }));
  },

  addLines: (newLines: string[]) => {
    set((state) => ({
      lines: [...state.lines, ...newLines].slice(-500),
    }));
  },

  clear: () => {
    set({ lines: [] });
  },

  setActive: (active: boolean) => {
    set({ isActive: active });
  },
}));
