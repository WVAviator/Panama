import { createSignal } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';

interface ReadResponse {
  output: string;
  instanceId: number;
  error?: string;
}

interface WriteResponse {
  instanceId: number;
  error?: string;
}

function App() {
  const [lines, setLines] = createSignal<string[]>([]);
  const [input, setInput] = createSignal<string>('');

  const readStdout = async () => {
    const { output, error } = (await invoke('read', {
      instanceId: 0,
    })) as ReadResponse;
    if (error) {
      console.error(error);
      const errorLines = error.split('\n');
      setLines((lines) => [...lines, ...errorLines]);
      return;
    }
    const outputLines = output.split('\n');
    setLines((lines) => [...lines, ...outputLines]);
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    console.log(`Submitting ${input()}`);
    const { error } = (await invoke('write', {
      input: input(),
      instanceId: 0,
    })) as WriteResponse;
    if (error) {
      console.error(error);
      const errorLines = error.split('\n');
      setLines((lines) => [...lines, ...errorLines]);
      return;
    }
    setInput('');
    window.scrollTo(0, document.body.scrollHeight);
    await readStdout();
  };

  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    setInput(target.value);
  };

  return (
    <div class="container">
      {lines().map((line) => (
        <p>{line}</p>
      ))}
      <form onSubmit={handleSubmit}>
        {`> `}
        <input value={input()} onChange={handleInput} autocorrect="off" />
        <button type="submit">Submit</button>
      </form>
    </div>
  );
}

export default App;
