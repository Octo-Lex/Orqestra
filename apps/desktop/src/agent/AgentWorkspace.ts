/**
 * AgentWorkspace — loads, isolates, and persists a single agent workspace.
 *
 * Spec §4.4: each workspace has its own personality, skills, tools, memory,
 * and confidence gate. State is persisted to `.Orqestra/agents/<id>/`.
 *
 * Key invariant: no workspace bleeds context into another.
 */

import { invoke } from '@tauri-apps/api/core';
import { ConfidenceGate, type ConfidenceGateConfig } from '../lib/ConfidenceGate';
import { SkillLoader, type Skill } from './SkillLoader';

export interface WorkspaceConfig {
  id: string;
  personality: string;
  model: string;
  skills: string[];
  tools: string[];
  memory: {
    type: 'session' | 'persistent' | 'episodic';
    max_tokens: number;
  };
  secrets: string[];
  confidence_gate: Partial<ConfidenceGateConfig>;
}

export interface WorkspaceState {
  workspaceId: string;
  status: 'idle' | 'running' | 'done' | 'error' | 'unavailable';
  currentTaskId: string | null;
  lastRunAt: string | null;
  lastResult: AgentResult | null;
  log: string[];
}

export interface AgentResult {
  workspaceId: string;
  taskId: string;
  changes: FileChange[];
  confidence: number;
  hasBreakingChange: boolean;
  commitHash: string | null;
  gateAction: string;
  summary: string;
}

export interface FileChange {
  path: string;
  action: 'create' | 'edit' | 'delete';
  content: string;
}

export class AgentWorkspace {
  readonly config: WorkspaceConfig;
  readonly gate: ConfidenceGate;
  readonly skills: Skill[];
  readonly state: WorkspaceState;

  private constructor(config: WorkspaceConfig, skills: Skill[], state: WorkspaceState) {
    this.config = config;
    this.gate = new ConfidenceGate(config.confidence_gate);
    this.skills = skills;
    this.state = state;
  }

  get id(): string {
    return this.config.id;
  }

  /**
   * Load a workspace from YAML config + skill files.
   * Resolves skill paths and builds the isolated context.
   */
  static async load(
    workspaceDir: string,
    projectRoot: string,
  ): Promise<AgentWorkspace> {
    // Read workspace.yml
    const yamlContent = await invoke<string>('read_file_cmd', {
      path: `${projectRoot}/agents/workspaces/${workspaceDir}/workspace.yml`,
    });

    const config = parseWorkspaceYaml(yamlContent);

    // Load skills
    const skillLoader = new SkillLoader(projectRoot);
    const skills = await skillLoader.loadSkills(config.skills);

    // Load persisted state
    let state: WorkspaceState;
    try {
      const stateJson = await invoke<string>('read_file_cmd', {
        path: `${projectRoot}/.Orqestra/agents/${config.id}/state.json`,
      });
      state = JSON.parse(stateJson);
    } catch {
      state = {
        workspaceId: config.id,
        status: 'idle',
        currentTaskId: null,
        lastRunAt: null,
        lastResult: null,
        log: [],
      };
    }

    return new AgentWorkspace(config, skills, state);
  }

  /**
   * Build the full prompt context for this workspace.
   * Includes personality + skills — nothing from other workspaces.
   */
  buildPrompt(taskTitle: string, taskContext: string): string {
    const skillContext = SkillLoader.buildSkillContext(this.skills);

    return `# Agent: ${this.config.id}

## Personality
${this.config.personality}

## Skills
${skillContext || '(no skills loaded)'}

## Task
${taskTitle}

## Context
${taskContext}

## Instructions
Using your personality and skills above, complete the task.
Return a JSON object with:
- "changes": array of {path, action, content} for each file to create/edit
- "confidence": number 0.0-1.0
- "hasBreakingChange": boolean
- "summary": one-line description of what you did`;
  }

  /**
   * Persist workspace state to .Orqestra/agents/<id>/state.json
   */
  async persistState(): Promise<void> {
    await invoke('write_file_cmd', {
      path: `${this.config.id}`,
      content: JSON.stringify(this.state, null, 2),
    });
  }

  /**
   * Add a log entry to this workspace's state.
   */
  log(message: string): void {
    const timestamp = new Date().toISOString();
    this.state.log.push(`[${timestamp}] ${message}`);
    // Keep last 100 entries
    if (this.state.log.length > 100) {
      this.state.log = this.state.log.slice(-100);
    }
  }
}

/**
 * Minimal YAML parser for workspace.yml files.
 * Handles the flat structure we need (no nested maps beyond confidence_gate).
 */
function parseWorkspaceYaml(yaml: string): WorkspaceConfig {
  const lines = yaml.split('\n');
  const config: Record<string, unknown> = {};
  let currentKey = '';

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();
    if (!line || line.startsWith('#')) continue;

    // Top-level key
    const topMatch = line.match(/^(\w+):\s*(.*)$/);
    if (topMatch && !rawLine.startsWith(' ')) {
      const [, key, value] = topMatch;
      currentKey = key;

      if (value) {
        // Simple scalar
        if (value === '[]') {
          config[key] = [];
        } else {
          config[key] = parseScalar(value);
        }
      }
      continue;
    }

    // List item
    const listMatch = line.match(/^\s+-\s+(.*)$/);
    if (listMatch) {
      const val = listMatch[1];
      if (!Array.isArray(config[currentKey])) {
        config[currentKey] = [];
      }
      (config[currentKey] as unknown[]).push(parseScalar(val));
      continue;
    }

    // Nested key (e.g. confidence_gate.auto_commit)
    const nestedMatch = line.match(/^\s+(\w+):\s*(.*)$/);
    if (nestedMatch && currentKey) {
      const [, subKey, value] = nestedMatch;
      if (typeof config[currentKey] !== 'object' || Array.isArray(config[currentKey])) {
        config[currentKey] = {};
      }
      if (value) {
        (config[currentKey] as Record<string, unknown>)[subKey] = parseScalar(value);
      }
    }
  }

  return {
    id: (config.id as string) ?? 'unknown',
    personality: (config.personality as string) ?? '',
    model: (config.model as string) ?? 'glm-5.1',
    skills: (config.skills as string[]) ?? [],
    tools: (config.tools as string[]) ?? [],
    memory: {
      type: ((config.memory as Record<string, unknown>)?.type as 'session') ?? 'session',
      max_tokens: ((config.memory as Record<string, unknown>)?.max_tokens as number) ?? 16000,
    },
    secrets: (config.secrets as string[]) ?? [],
    confidence_gate: (config.confidence_gate as Partial<ConfidenceGateConfig>) ?? {},
  };
}

function parseScalar(value: string): unknown {
  // Remove surrounding quotes
  const stripped = value.replace(/^["']|["']$/g, '');
  if (stripped === 'true') return true;
  if (stripped === 'false') return false;
  if (/^\d+$/.test(stripped)) return parseInt(stripped, 10);
  if (/^\d+\.\d+$/.test(stripped)) return parseFloat(stripped);
  return stripped;
}
