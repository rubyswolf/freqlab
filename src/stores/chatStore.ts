import { create } from 'zustand';

interface ChatState {
  pendingMessage: string | null;

  // Actions
  queueMessage: (message: string) => void;
  clearPendingMessage: () => void;
}

export const useChatStore = create<ChatState>((set) => ({
  pendingMessage: null,

  queueMessage: (message) => {
    set({ pendingMessage: message });
  },

  clearPendingMessage: () => {
    set({ pendingMessage: null });
  },
}));
