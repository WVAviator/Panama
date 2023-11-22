import './Tabs.css';
import { Accessor, Setter } from 'solid-js';

export interface Tab {
  title: string;
  instanceId: number;
}

interface TabsProps {
  tabs: Accessor<Tab[]>;
  activeTab: Accessor<number>;
  setActiveTab: Setter<number>;
}

const Tabs = ({ tabs, activeTab, setActiveTab }: TabsProps) => {
  return (
    <div class={'container'}>
      {tabs().map((tab, index) => (
        <div
          class={activeTab() === index ? 'tab active' : 'tab'}
          onClick={() => setActiveTab(index)}
        >
          {tab.title}
        </div>
      ))}
    </div>
  );
};

export default Tabs;
