import { ParentComponent, createContext, createMemo } from 'solid-js';
import { OpenAISession } from './OpenAISession';
import { AISession } from './AISession';

export const AIContext = createContext<AISession>();

const AIProvider: ParentComponent = (props) => {
  const openai = createMemo(() => new OpenAISession());
  return (
    <AIContext.Provider value={openai()}>{props.children}</AIContext.Provider>
  );
};

export default AIProvider;
