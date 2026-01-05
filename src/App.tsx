import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from './stores/settingsStore';
import { useProjectStore } from './stores/projectStore';
import { useToastStore } from './stores/toastStore';
import { WelcomeWizard } from './components/Setup/WelcomeWizard';
import { MainLayout } from './components/Layout/MainLayout';
import { applyTheme } from './components/Settings/ThemePicker';
import type { PrerequisiteStatus } from './types';

function App() {
  const setupComplete = useSettingsStore((state) => state.setupComplete);
  const theme = useSettingsStore((state) => state.theme);
  const customColors = useSettingsStore((state) => state.customColors);
  const loadProjects = useProjectStore((state) => state.loadProjects);
  const { addToast } = useToastStore();
  const [hasCheckedPrereqs, setHasCheckedPrereqs] = useState(false);

  // Apply theme on startup and when it changes
  useEffect(() => {
    applyTheme(theme, customColors);
  }, [theme, customColors]);

  // Load projects on startup
  useEffect(() => {
    if (setupComplete) {
      loadProjects();
    }
  }, [setupComplete, loadProjects]);

  // Silent prerequisites check on startup
  useEffect(() => {
    if (!setupComplete || hasCheckedPrereqs) return;

    async function checkPrereqs() {
      try {
        const status = await invoke<PrerequisiteStatus>('check_prerequisites');

        // Check if Claude CLI or auth have issues
        const cliOk = status.claude_cli.status === 'installed';
        const authOk = status.claude_auth.status === 'installed';

        if (!cliOk) {
          addToast({
            type: 'warning',
            message: 'Claude CLI not found. Install it to use Claude features.',
          });
        } else if (!authOk) {
          addToast({
            type: 'warning',
            message: 'Claude CLI not authenticated. Run "claude login" in terminal.',
          });
        }
      } catch (err) {
        console.error('Failed to check prerequisites:', err);
      }
      setHasCheckedPrereqs(true);
    }

    checkPrereqs();
  }, [setupComplete, hasCheckedPrereqs, addToast]);

  if (!setupComplete) {
    return <WelcomeWizard />;
  }

  return <MainLayout />;
}

export default App;
