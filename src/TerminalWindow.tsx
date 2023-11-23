import { invoke } from '@tauri-apps/api';
import { appWindow } from '@tauri-apps/api/window';
import { FitAddon } from '@xterm/addon-fit';
import { Accessor } from 'solid-js';
import { Terminal, XTerm } from 'solid-xterm';

interface ReadResponse {
  output: string;
}

interface TerminalProps {
  instanceId: number;
  active: Accessor<boolean>;
  onDirChange?: (dir: string) => void;
}

const TerminalWindow = ({ instanceId, active, onDirChange }: TerminalProps) => {
  const handleMount = async (terminal: Terminal) => {
    try {
      await invoke('create', {
        cols: terminal.cols,
        rows: terminal.rows,
        instanceId,
        command: 'zsh',
      });

      const unlisten = await appWindow.listen(`read:${instanceId}`, (event) => {
        const { output } = event.payload as ReadResponse;
        terminal.write(output);
      });

      return async () => {
        console.log('Terminal unmounted. Sending interrupt to pty.');
        await invoke('destroy', {
          instanceId,
        });

        unlisten();
      };
    } catch (e) {
      console.error(e);
      return () => {};
    }
  };

  const handleData = async (data: string) => {
    try {
      await invoke('write', {
        instanceId,
        input: data,
      });
    } catch (e) {
      console.error(e);
      return;
    }
  };

  return (
    <div style={{ display: active() ? 'block' : 'none' }}>
      <XTerm
        onData={handleData}
        onMount={handleMount}
        onTitleChange={(title) => {
          if (title.includes(':')) {
            const titleParts = title.split(':');
            onDirChange?.(titleParts[titleParts.length - 1]);
          }
        }}
        options={{
          fontFamily: '"JetBrains Mono", "Roboto Mono", monospace',
        }}
        addons={[FitAddon]}
      />
    </div>
  );
};

export default TerminalWindow;
