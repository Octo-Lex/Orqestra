// View modes for the PM view switcher

export type ViewMode = 'table' | 'gantt' | 'kanban';

interface ViewSwitcherProps {
  current: ViewMode;
  onChange: (view: ViewMode) => void;
}

const VIEWS: { id: ViewMode; label: string; icon: string }[] = [
  { id: 'table', label: 'Table', icon: '☰' },
  { id: 'gantt', label: 'Gantt', icon: '━' },
  { id: 'kanban', label: 'Kanban', icon: '▦' },
];

export function ViewSwitcher({ current, onChange }: ViewSwitcherProps) {
  return (
    <div style={{
      display: 'inline-flex',
      border: '1px solid #d1d5db',
      borderRadius: '6px',
      overflow: 'hidden',
      marginBottom: '0.75rem',
    }}>
      {VIEWS.map(view => (
        <button
          key={view.id}
          onClick={() => onChange(view.id)}
          style={{
            padding: '0.35rem 0.75rem',
            border: 'none',
            borderRight: view.id !== 'kanban' ? '1px solid #e5e7eb' : 'none',
            background: current === view.id ? '#2563eb' : 'white',
            color: current === view.id ? 'white' : '#374151',
            cursor: 'pointer',
            fontSize: '0.85em',
            fontWeight: current === view.id ? 600 : 400,
            transition: 'all 0.15s',
          }}
        >
          {view.icon} {view.label}
        </button>
      ))}
    </div>
  );
}
