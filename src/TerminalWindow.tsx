import { invoke } from '@tauri-apps/api';
import { XTerm, Terminal } from 'solid-xterm';

interface WriteResponse {
  instanceId: number;
  output: string;
  error?: string;
}

const TerminalWindow = () => {
  const handleData = async (data: string, terminal: Terminal) => {
    try {
      const { output } = (await invoke('write', {
        input: data,
        instanceId: 0,
      })) as WriteResponse;
      terminal.write(output);
    } catch (e) {
      console.error(e);
      return;
    }
  };

  return (
    <div>
      <XTerm onData={handleData} />
    </div>
  );
};

export default TerminalWindow;
