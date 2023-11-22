import './Tabs.css';
import { Accessor, For, Setter } from 'solid-js';

interface TabsProps {
  tabNames: Accessor<string[]>;
  activeTab: Accessor<number>;
  setActiveTab: Setter<number>;
}

const Tabs = ({ tabNames, activeTab, setActiveTab }: TabsProps) => {
  return (
    <div class={'container'}>
      <For each={tabNames()}>
        {(tab, index) => (
          <div
            class={activeTab() === index() ? 'tab active' : 'tab'}
            onClick={() => setActiveTab(index())}
          >
            {tab}
          </div>
        )}
      </For>
    </div>
  );
};

export default Tabs;
