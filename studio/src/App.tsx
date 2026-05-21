import { useEffect } from 'react';
import { AppShell } from './components/layout/AppShell';
import { useSettingsStore } from './store/settingsStore';
import './App.css';

function App() {
  const { load } = useSettingsStore();
  useEffect(() => { load(); }, [load]);
  return <AppShell />;
}

export default App;
