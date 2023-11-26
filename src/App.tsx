import { For, createMemo, createSignal } from 'solid-js';
import './App.css';
import Tabs from './Tabs';
import TerminalWindow from './TerminalWindow';

function App() {
  const [tabIds, setTabIds] = createSignal<number[]>([0, 1]);
  const [tabNames, setTabNames] = createSignal<string[]>([
    'New Tab',
    'New Tab',
  ]);

  const [activeTab, setActiveTab] = createSignal(0);

  return (
    <div>
      <Tabs
        tabNames={tabNames}
        activeTab={activeTab}
        setActiveTab={setActiveTab}
      />
      <For each={tabIds()}>
        {(tabId, index) => {
          const derivedActiveSignal = createMemo(() => activeTab() === index());
          const handleDirChange = (dir: string) => {
            const dirParts = dir.split('/');
            const lastDir = dirParts[dirParts.length - 1];

            setTabNames((prevTabNames) => {
              const newTabNames = [...prevTabNames];
              newTabNames[index()] = lastDir;
              return newTabNames;
            });
          };
          return (
            <TerminalWindow
              instanceId={tabId}
              active={derivedActiveSignal}
              onDirChange={handleDirChange}
            />
          );
        }}
      </For>
    </div>
  );
}

export default App;
