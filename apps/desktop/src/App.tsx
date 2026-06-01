import { useState } from 'react';
import { TaskTable } from './components/TaskTable';
import { open } from '@tauri-apps/plugin-dialog';

export default function App() {
  const [projectRoot, setProjectRoot] = useState<string | null>(null);

  async function openProject() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') setProjectRoot(selected);
  }

  return (
    <div style={{ padding: '1rem' }}>
      {!projectRoot ? (
        <button onClick={openProject}>Open project folder</button>
      ) : (
        <>
          <div style={{ marginBottom: '0.5rem', color: '#666' }}>
            {projectRoot}
            <button onClick={() => setProjectRoot(null)} style={{ marginLeft: '1rem' }}>
              Close
            </button>
          </div>
          <TaskTable projectRoot={projectRoot} />
        </>
      )}
    </div>
  );
}
