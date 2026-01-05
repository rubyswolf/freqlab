import { create } from 'zustand';

interface ProjectOutput {
  lines: string[];
  isActive: boolean;
}

interface OutputState {
  // Map of project path -> output state
  projectOutputs: Record<string, ProjectOutput>;

  // Currently active project path (for the output panel to know which to show)
  activeProjectPath: string | null;

  // Actions - all take projectPath to scope to that project
  addLine: (projectPath: string, line: string) => void;
  addLines: (projectPath: string, lines: string[]) => void;
  clear: (projectPath: string) => void;
  setActive: (projectPath: string, active: boolean) => void;
  setActiveProject: (projectPath: string | null) => void;

  // Getters
  getOutput: (projectPath: string) => ProjectOutput;
}

const defaultOutput: ProjectOutput = {
  lines: [],
  isActive: false,
};

export const useOutputStore = create<OutputState>((set, get) => ({
  projectOutputs: {},
  activeProjectPath: null,

  addLine: (projectPath: string, line: string) => {
    set((state) => {
      const current = state.projectOutputs[projectPath] || defaultOutput;
      return {
        projectOutputs: {
          ...state.projectOutputs,
          [projectPath]: {
            ...current,
            lines: [...current.lines, line].slice(-500), // Keep last 500 lines
          },
        },
      };
    });
  },

  addLines: (projectPath: string, newLines: string[]) => {
    set((state) => {
      const current = state.projectOutputs[projectPath] || defaultOutput;
      return {
        projectOutputs: {
          ...state.projectOutputs,
          [projectPath]: {
            ...current,
            lines: [...current.lines, ...newLines].slice(-500),
          },
        },
      };
    });
  },

  clear: (projectPath: string) => {
    set((state) => {
      const current = state.projectOutputs[projectPath] || defaultOutput;
      return {
        projectOutputs: {
          ...state.projectOutputs,
          [projectPath]: {
            ...current,
            lines: [],
          },
        },
      };
    });
  },

  setActive: (projectPath: string, active: boolean) => {
    set((state) => {
      const current = state.projectOutputs[projectPath] || defaultOutput;
      return {
        projectOutputs: {
          ...state.projectOutputs,
          [projectPath]: {
            ...current,
            isActive: active,
          },
        },
      };
    });
  },

  setActiveProject: (projectPath: string | null) => {
    set({ activeProjectPath: projectPath });
  },

  getOutput: (projectPath: string) => {
    return get().projectOutputs[projectPath] || defaultOutput;
  },
}));

// Helper hook for components that need output for the current project
export function useProjectOutput(projectPath: string | null) {
  // Subscribe to the specific project's output - this ensures re-renders on changes
  const output = useOutputStore((state) =>
    projectPath ? state.projectOutputs[projectPath] || defaultOutput : defaultOutput
  );
  const addLine = useOutputStore((state) => state.addLine);
  const addLines = useOutputStore((state) => state.addLines);
  const clear = useOutputStore((state) => state.clear);
  const setActive = useOutputStore((state) => state.setActive);

  if (!projectPath) {
    return {
      lines: [],
      isActive: false,
      addLine: () => {},
      addLines: () => {},
      clear: () => {},
      setActive: () => {},
    };
  }

  return {
    lines: output.lines,
    isActive: output.isActive,
    addLine: (line: string) => addLine(projectPath, line),
    addLines: (lines: string[]) => addLines(projectPath, lines),
    clear: () => clear(projectPath),
    setActive: (active: boolean) => setActive(projectPath, active),
  };
}
