import { createMemo, createSignal } from 'solid-js';
import './App.css';
import Tabs, { Tab } from './Tabs';
import Terminal from './TerminalWindow';

function App() {
  const [tabs, setTabs] = createSignal<Tab[]>([
    {
      title: 'New Tab',
      instanceId: 0,
    },
    {
      title: 'New Tab',
      instanceId: 1,
    },
  ]);
  const [activeTab, setActiveTab] = createSignal(0);
  return (
    <div>
      <Tabs tabs={tabs} activeTab={activeTab} setActiveTab={setActiveTab} />
      {tabs().map((tab, index) => {
        const derivedActiveSignal = createMemo(() => activeTab() === index);
        return (
          <Terminal instanceId={tab.instanceId} active={derivedActiveSignal} />
        );
      })}
    </div>
  );
}

export default App;
