/**
 * AgentRouter — label-based task routing to workspaces.
 *
 * Spec §4.4: "label-based task routing (`bug` → bugfix agent, `docs` → docs agent)"
 *
 * The router:
 * 1. Matches task labels to workspace IDs via configurable routing rules
 * 2. Loads the target workspace (isolated context, skills, gate)
 * 3. Delegates to AgentRunner for execution
 * 4. Returns the result with gate action applied
 */

import { AgentWorkspace, type AgentResult } from './AgentWorkspace';
import { AgentRunner } from './AgentRunner';
import type { Task } from '../lib/orqestra';

/** Routing rules: label pattern → workspace directory name */
export const ROUTING_RULES: Record<string, string> = {
  bug: 'bugfix',
  fix: 'bugfix',
  regression: 'bugfix',
  docs: 'docs',
  documentation: 'docs',
  readme: 'docs',
  changelog: 'docs',
  architecture: 'architect',
  adr: 'architect',
  design: 'architect',
  'design-decision': 'architect',
};

export interface RouteResult {
  workspace: AgentWorkspace;
  workspaceDir: string;
  matchReason: string;
}

export class AgentRouter {
  private projectRoot: string;
  private workspaceCache = new Map<string, AgentWorkspace>();

  constructor(projectRoot: string) {
    this.projectRoot = projectRoot;
  }

  /**
   * Route a task to the best-matching workspace based on labels.
   * Returns the workspace and the reason for the match.
   */
  async route(task: Task): Promise<RouteResult> {
    const labels = task.frontmatter.labels ?? [];
    let bestMatch: string | null = null;
    let bestDir: string | null = null;

    // Check labels against routing rules
    for (const label of labels) {
      const lower = label.toLowerCase();
      if (ROUTING_RULES[lower]) {
        bestMatch = label;
        bestDir = ROUTING_RULES[lower];
        break;
      }
    }

    // Fallback: check task title keywords
    if (!bestMatch) {
      const title = task.frontmatter.title.toLowerCase();
      if (title.includes('fix') || title.includes('bug')) {
        bestMatch = 'title:fix/bug';
        bestDir = 'bugfix';
      } else if (title.includes('doc') || title.includes('readme')) {
        bestMatch = 'title:doc';
        bestDir = 'docs';
      } else if (title.includes('adr') || title.includes('architect')) {
        bestMatch = 'title:adr/architect';
        bestDir = 'architect';
      }
    }

    // Default fallback: bugfix (safest workspace)
    if (!bestDir) {
      bestMatch = 'default';
      bestDir = 'bugfix';
    }

    const workspace = await this.loadWorkspace(bestDir);

    return {
      workspace,
      workspaceDir: bestDir,
      matchReason: `label "${bestMatch}" → ${bestDir}`,
    };
  }

  /**
   * Load a workspace (cached after first load).
   */
  async loadWorkspace(dir: string): Promise<AgentWorkspace> {
    if (this.workspaceCache.has(dir)) {
      return this.workspaceCache.get(dir)!;
    }
    const ws = await AgentWorkspace.load(dir, this.projectRoot);
    this.workspaceCache.set(dir, ws);
    return ws;
  }

  /**
   * Run a single task: route → execute → commit.
   */
  async runTask(task: Task): Promise<AgentResult> {
    const { workspace, matchReason } = await this.route(task);
    workspace.log(`Routed: ${matchReason}`);
    workspace.state.status = 'running';
    workspace.state.currentTaskId = task.frontmatter.id;

    try {
      const runner = new AgentRunner(this.projectRoot, workspace);
      const result = await runner.run(task);
      workspace.state.status = 'done';
      workspace.state.lastRunAt = new Date().toISOString();
      workspace.state.lastResult = result;
      workspace.log(`Done: confidence=${result.confidence}, gate=${result.gateAction}`);
      return result;
    } catch (e) {
      workspace.state.status = 'error';
      workspace.log(`Error: ${e instanceof Error ? e.message : String(e)}`);
      throw e;
    } finally {
      workspace.state.currentTaskId = null;
      await workspace.persistState();
    }
  }

  /**
   * Run multiple tasks in parallel — one per workspace.
   * Returns results keyed by task ID.
   */
  async runParallel(tasks: Task[]): Promise<Map<string, AgentResult>> {
    const results = new Map<string, AgentResult>();

    // Group tasks by workspace to avoid conflicts
    const byWorkspace = new Map<string, Task[]>();
    for (const task of tasks) {
      const { workspaceDir } = await this.route(task);
      if (!byWorkspace.has(workspaceDir)) {
        byWorkspace.set(workspaceDir, []);
      }
      byWorkspace.get(workspaceDir)!.push(task);
    }

    // Run one task per workspace simultaneously
    const promises: Promise<void>[] = [];

    for (const [_dir, wsTasks] of byWorkspace) {
      // Only take the first task per workspace for parallel run
      const task = wsTasks[0];
      promises.push(
        this.runTask(task)
          .then(result => { results.set(task.frontmatter.id, result); })
          .catch(e => {
            console.error(`Failed task ${task.frontmatter.id}:`, e);
          }),
      );
    }

    await Promise.all(promises);
    return results;
  }
}
