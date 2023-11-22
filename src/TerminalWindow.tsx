import { invoke } from '@tauri-apps/api';
import { appWindow } from '@tauri-apps/api/window';
import { FitAddon } from '@xterm/addon-fit';
import { createSignal } from 'solid-js';
import { Terminal, XTerm } from 'solid-xterm';

interface CreateResponse {
  instance_id: number;
}

interface ReadResponse {
  output: string;
}

const TerminalWindow = () => {
  const [instanceId, setInstanceId] = createSignal<number>(0);

  const handleMount = async (terminal: Terminal) => {
    try {
      const response = await invoke<CreateResponse>('create', {
        cols: terminal.cols,
        rows: terminal.rows,
      });
      const { instance_id: id } = response;
      setInstanceId(id);

      const unlisten = await appWindow.listen(`read:${id}`, (event) => {
        const { output } = event.payload as ReadResponse;
        terminal.write(output);
      });

      return () => {
        unlisten();
      };
    } catch (e) {
      console.error(e);
      return () => {};
    }
  };

  const handleData = async (data: string) => {
    const id = instanceId();
    try {
      await invoke('write', {
        instanceId: id,
        input: data,
      });
    } catch (e) {
      console.error(e);
      return;
    }
  };

  return (
    <div>
      <XTerm
        onData={handleData}
        onMount={handleMount}
        options={{
          fontFamily: '"JetBrains Mono", "Roboto Mono", monospace',
        }}
        addons={[FitAddon]}
      />
    </div>
  );
};

export default TerminalWindow;
