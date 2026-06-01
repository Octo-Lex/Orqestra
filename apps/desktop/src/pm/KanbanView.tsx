import { useState } from 'react';
import {
  DndContext,
  DragOverlay,
  closestCorners,
  PointerSensor,
  useSensor,
  useSensors,
  type DragStartEvent,
  type DragEndEvent,
  type DragOverEvent,
} from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy, useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import type { Task, TaskStatus } from '../lib/orqestra';

const COLUMNS: { id: TaskStatus; label: string; color: string }[] = [
  { id: 'backlog', label: 'Backlog', color: '#9ca3af' },
  { id: 'ready', label: 'Ready', color: '#3b82f6' },
  { id: 'in-progress', label: 'In Progress', color: '#f59e0b' },
  { id: 'in-review', label: 'In Review', color: '#8b5cf6' },
  { id: 'done', label: 'Done', color: '#10b981' },
  { id: 'cancelled', label: 'Cancelled', color: '#ef4444' },
];

const PRIORITY_BADGE_COLORS: Record<string, string> = {
  'Critical': '#dc2626',
  'High': '#ea580c',
  'Medium': '#ca8a04',
  'Low': '#6b7280',
};

interface KanbanViewProps {
  tasks: Task[];
  onStatusChange: (taskId: string, newStatus: TaskStatus) => void;
}

function TaskCard({ task, isDragging }: { task: Task; isDragging?: boolean }) {
  const { attributes, listeners, setNodeRef, transform, transition } = useSortable({
    id: task.frontmatter.id,
  });

  const fm = task.frontmatter;
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={{
        ...style,
        background: 'white',
        border: '1px solid #e5e7eb',
        borderRadius: '6px',
        padding: '0.5rem 0.75rem',
        marginBottom: '0.5rem',
        cursor: 'grab',
        boxShadow: '0 1px 3px rgba(0,0,0,0.06)',
      }}
      {...attributes}
      {...listeners}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.25rem' }}>
        <code style={{ fontSize: '0.75em', color: '#6b7280' }}>{fm.id}</code>
        {fm.priority && (
          <span style={{
            fontSize: '0.65em',
            padding: '0.1rem 0.4rem',
            borderRadius: '9999px',
            color: 'white',
            backgroundColor: PRIORITY_BADGE_COLORS[fm.priority] || '#6b7280',
            fontWeight: 600,
          }}>
            {fm.priority}
          </span>
        )}
      </div>
      <div style={{ fontSize: '0.85em', fontWeight: 500, marginBottom: '0.25rem' }}>
        {fm.title}
      </div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontSize: '0.7em', color: '#9ca3af' }}>
          {fm.assignee || 'Unassigned'}
        </span>
        {fm.progress > 0 && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
            <div style={{
              width: '40px', height: '4px', borderRadius: '2px',
              backgroundColor: '#e5e7eb', overflow: 'hidden',
            }}>
              <div style={{
                width: `${fm.progress}%`, height: '100%',
                backgroundColor: '#10b981', borderRadius: '2px',
              }} />
            </div>
            <span style={{ fontSize: '0.65em', color: '#9ca3af' }}>{fm.progress}%</span>
          </div>
        )}
      </div>
    </div>
  );
}

function Column({
  status: _status,
  label,
  color,
  tasks,
}: {
  status: TaskStatus;
  label: string;
  color: string;
  tasks: Task[];
}) {
  return (
    <div style={{
      flex: '1 1 0',
      minWidth: '180px',
      maxWidth: '250px',
      background: '#f9fafb',
      borderRadius: '8px',
      padding: '0.5rem',
      border: '1px solid #e5e7eb',
    }}>
      <div style={{
        display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        marginBottom: '0.5rem', padding: '0 0.25rem',
      }}>
        <span style={{ fontWeight: 600, fontSize: '0.85em' }}>
          <span style={{
            display: 'inline-block', width: 8, height: 8,
            borderRadius: '50%', backgroundColor: color, marginRight: '0.4rem',
          }} />
          {label}
        </span>
        <span style={{
          fontSize: '0.7em', backgroundColor: '#e5e7eb',
          borderRadius: '9999px', padding: '0.1rem 0.5rem',
          color: '#6b7280',
        }}>
          {tasks.length}
        </span>
      </div>

      <SortableContext items={tasks.map(t => t.frontmatter.id)} strategy={verticalListSortingStrategy}>
        {tasks.map(task => (
          <TaskCard key={task.frontmatter.id} task={task} />
        ))}
      </SortableContext>

      {tasks.length === 0 && (
        <div style={{ textAlign: 'center', padding: '1rem', color: '#d1d5db', fontSize: '0.8em' }}>
          No tasks
        </div>
      )}
    </div>
  );
}

export function KanbanView({ tasks, onStatusChange }: KanbanViewProps) {
  const [activeId, setActiveId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  );

  // Group tasks by status
  const columns = COLUMNS.map(col => ({
    ...col,
    tasks: tasks.filter(t => t.frontmatter.status === col.id),
  }));

  // Find which column a task belongs to
  function findColumn(taskId: string): TaskStatus | null {
    for (const col of columns) {
      if (col.tasks.some(t => t.frontmatter.id === taskId)) return col.id;
    }
    return null;
  }

  // Find the task data
  const activeTask = activeId ? tasks.find(t => t.frontmatter.id === activeId) : null;

  function handleDragStart(event: DragStartEvent) {
    setActiveId(event.active.id as string);
  }

  function handleDragOver(event: DragOverEvent) {
    // Optional: visual feedback during drag
    void event;
  }

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    setActiveId(null);

    if (!over) return;

    const taskId = active.id as string;
    const currentStatus = findColumn(taskId);
    if (!currentStatus) return;

    // Determine target column: either the over item's column or a column directly
    const overId = over.id as string;
    let targetStatus: TaskStatus | null = null;

    // Check if over a column directly
    const targetCol = COLUMNS.find(c => c.id === overId);
    if (targetCol) {
      targetStatus = targetCol.id;
    } else {
      // Over another task — use that task's column
      targetStatus = findColumn(overId);
    }

    if (targetStatus && targetStatus !== currentStatus) {
      onStatusChange(taskId, targetStatus);
    }
  }

  return (
    <div>
      <h3 style={{ margin: '0 0 0.5rem', fontSize: '1rem' }}>Kanban View</h3>
      <DndContext
        sensors={sensors}
        collisionDetection={closestCorners}
        onDragStart={handleDragStart}
        onDragOver={handleDragOver}
        onDragEnd={handleDragEnd}
      >
        <div style={{ display: 'flex', gap: '0.5rem', overflowX: 'auto', paddingBottom: '0.5rem' }}>
          {columns.map(col => (
            <Column key={col.id} status={col.id} label={col.label} color={col.color} tasks={col.tasks} />
          ))}
        </div>

        <DragOverlay>
          {activeTask ? <TaskCard task={activeTask} isDragging /> : null}
        </DragOverlay>
      </DndContext>
    </div>
  );
}
