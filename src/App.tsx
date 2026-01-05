import { useSettingsStore } from './stores/settingsStore';
import { WelcomeWizard } from './components/Setup/WelcomeWizard';
import { MainLayout } from './components/Layout/MainLayout';

function App() {
  const setupComplete = useSettingsStore((state) => state.setupComplete);

  if (!setupComplete) {
    return <WelcomeWizard />;
  }

  return <MainLayout />;
}

export default App;
