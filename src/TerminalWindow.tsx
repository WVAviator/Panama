import { invoke, window } from '@tauri-apps/api';
import { appWindow } from '@tauri-apps/api/window';
import { FitAddon } from '@xterm/addon-fit';
import { Accessor, createMemo } from 'solid-js';
import { Terminal, XTerm } from 'solid-xterm';
import './TerminalWindow.css';

interface ReadResponse {
  output: string;
}

interface TerminalProps {
  instanceId: number;
  active: Accessor<boolean>;
  onDirChange?: (dir: string) => void;
}

const TerminalWindow = ({ instanceId, active, onDirChange }: TerminalProps) => {
  const fitAddon = createMemo(() => new FitAddon());

  const handleMount = async (terminal: Terminal) => {
    terminal.refresh(0, terminal.rows - 1);

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

      const resizeUnlisten = await appWindow.onResized(() => {
        fitAddon().fit();
      });

      return async () => {
        console.log('Terminal unmounted. Sending interrupt to pty.');
        await invoke('destroy', {
          instanceId,
        });

        unlisten();
        resizeUnlisten();
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

  const handleResize = async ({
    cols,
    rows,
  }: {
    cols: number;
    rows: number;
  }) => {
    try {
      await invoke('resize', {
        instanceId,
        cols,
        rows,
      });
    } catch (e) {
      console.error(e);
      return;
    }
  };

  return (
    <div
      style={{ display: active() ? 'block' : 'none' }}
      class="terminal-container"
    >
      <XTerm
        onData={handleData}
        onMount={handleMount}
        onResize={handleResize}
        onTitleChange={(title) => {
          if (title.includes(':')) {
            const titleParts = title.split(':');
            onDirChange?.(titleParts[titleParts.length - 1]);
          }
        }}
        options={{
          fontFamily: '"JetBrains Mono", "Roboto Mono", monospace',
        }}
        addons={[fitAddon()]}
      />
    </div>
  );
};

export default TerminalWindow;
