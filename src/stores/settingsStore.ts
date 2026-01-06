import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { AppConfig, DawPaths, CustomThemeColors } from '../types';

const defaultDawPaths: DawPaths = {
  reaper: { vst3: '~/Library/Audio/Plug-Ins/VST3', clap: '~/Library/Audio/Plug-Ins/CLAP' },
  ableton: { vst3: '~/Library/Audio/Plug-Ins/VST3', clap: '~/Library/Audio/Plug-Ins/CLAP' },
  flStudio: { vst3: '~/Library/Audio/Plug-Ins/VST3', clap: '~/Library/Audio/Plug-Ins/CLAP' },
  logic: { vst3: '~/Library/Audio/Plug-Ins/VST3', clap: '~/Library/Audio/Plug-Ins/CLAP' },
  other: { vst3: '', clap: '' },
};

const defaultCustomColors: CustomThemeColors = {
  accent: '#2DA86E',
  bgPrimary: '#0f0f0f',
  bgSecondary: '#171717',
  bgTertiary: '#1f1f1f',
  textPrimary: '#fafafa',
  textSecondary: '#a1a1aa',
};

interface SettingsState extends AppConfig {
  setSetupComplete: (complete: boolean) => void;
  setWorkspacePath: (path: string) => void;
  setTheme: (theme: 'dark' | 'light' | 'custom') => void;
  setCustomColors: (colors: CustomThemeColors) => void;
  updateCustomColor: (key: keyof CustomThemeColors, value: string) => void;
  setVendorName: (name: string) => void;
  setVendorUrl: (url: string) => void;
  setVendorEmail: (email: string) => void;
  setDawPaths: (paths: DawPaths) => void;
  updateDawPath: (daw: keyof DawPaths, format: 'vst3' | 'clap', path: string) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      workspacePath: '~/VSTWorkshop',
      outputPath: '~/VSTWorkshop/output',
      buildFormats: ['vst3', 'clap'],
      autoOpenOutput: true,
      showNotifications: true,
      theme: 'dark',
      customColors: defaultCustomColors,
      setupComplete: false,
      // Branding defaults
      vendorName: 'freqlab',
      vendorUrl: '',
      vendorEmail: '',
      // DAW paths defaults
      dawPaths: defaultDawPaths,

      setSetupComplete: (complete) => set({ setupComplete: complete }),
      setWorkspacePath: (path) => set({ workspacePath: path }),
      setTheme: (theme) => set({ theme }),
      setCustomColors: (colors) => set({ customColors: colors }),
      updateCustomColor: (key, value) =>
        set((state) => ({
          customColors: {
            ...state.customColors,
            [key]: value,
          },
        })),
      setVendorName: (name) => set({ vendorName: name }),
      setVendorUrl: (url) => set({ vendorUrl: url }),
      setVendorEmail: (email) => set({ vendorEmail: email }),
      setDawPaths: (paths) => set({ dawPaths: paths }),
      updateDawPath: (daw, format, path) =>
        set((state) => ({
          dawPaths: {
            ...state.dawPaths,
            [daw]: {
              ...state.dawPaths[daw],
              [format]: path,
            },
          },
        })),
    }),
    {
      name: 'freqlab-settings',
    }
  )
);
