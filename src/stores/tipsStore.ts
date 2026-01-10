import { create } from 'zustand';
import { persist } from 'zustand/middleware';

// Tip IDs - add new ones here as we create more tips
export type TipId =
  | 'collapse-sidebar'      // Hint to collapse sidebar for more space
  | 'preview-controls'      // Hint about preview panel controls
  | 'version-revert'        // Hint about reverting to previous versions
  | 'auto-build'            // Hint about auto-build feature
  | 'plugin-editor';        // Hint about opening plugin editor

interface TipsState {
  // Set of tip IDs that have been shown
  shownTips: string[];

  // Check if a tip has been shown
  hasTipBeenShown: (tipId: TipId) => boolean;

  // Mark a tip as shown
  markTipShown: (tipId: TipId) => void;

  // Reset all tips (for dev settings)
  resetAllTips: () => void;
}

export const useTipsStore = create<TipsState>()(
  persist(
    (set, get) => ({
      shownTips: [],

      hasTipBeenShown: (tipId) => {
        return get().shownTips.includes(tipId);
      },

      markTipShown: (tipId) => {
        const current = get().shownTips;
        if (!current.includes(tipId)) {
          set({ shownTips: [...current, tipId] });
        }
      },

      resetAllTips: () => {
        set({ shownTips: [] });
      },
    }),
    {
      name: 'freqlab-tips',
    }
  )
);
