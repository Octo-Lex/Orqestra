/**
 * AgentRunner — executes a single task within an isolated workspace.
 *
 * v2.14.1: Removed mock/fabricated fallback. When the AI service is
 * unavailable, returns a structured unavailable error instead of
 * fabricating realistic changes. A governed AI tool must never invent
 * agent results.
 *
 * Flow:
 * 1. Build prompt from workspace personality + skills + task context
 * 2. Call Rust command to dispatch to AI service
 * 3. Receive structured response (changes, confidence, summary)
 * 4. Return AgentResult with gate action resolved
 */

import { invoke } from '@tauri-apps/api/core';
import { AgentWorkspace, type AgentResult, type FileChange } from './AgentWorkspace';
import type { Task } from '../lib/orqestra';

export interface AgentRunResponse {
  changes: FileChange[];
  confidence: number;
  hasBreakingChange: boolean;
  summary: string;
}

export interface AgentUnavailableResult {
  workspaceId: string;
  taskId: string;
  available: false;
  reason: string;
}

export class AgentRunner {
  private projectRoot: string;
  private workspace: AgentWorkspace;

  constructor(projectRoot: string, workspace: AgentWorkspace) {
    this.projectRoot = projectRoot;
    this.workspace = workspace;
  }

  /**
   * Execute the agent loop for a task.
   * Returns AgentResult on success, or AgentUnavailableResult if the
   * AI service is down or returns a non-real response.
   */
  async run(task: Task): Promise<AgentResult | AgentUnavailableResult> {
    // 1. Build isolated prompt (personality + skills + task — no other workspace context)
    const taskContext = `Title: ${task.frontmatter.title}\n` +
      `ID: ${task.frontmatter.id}\n` +
      `Status: ${task.frontmatter.status}\n` +
      `Labels: ${task.frontmatter.labels?.join(', ') ?? 'none'}\n` +
      `Source: ${task.source_path}`;

    const prompt = this.workspace.buildPrompt(task.frontmatter.title, taskContext);

    this.workspace.log(`Running agent: ${this.workspace.id} on ${task.frontmatter.id}`);

    // 2. Call the AI service via Rust command
    let response: AgentRunResponse;
    try {
      const rawResponse = await invoke<string>('run_agent_cmd', {
        projectRoot: this.projectRoot,
        workspaceId: this.workspace.id,
        model: this.workspace.config.model,
        prompt,
        taskId: task.frontmatter.id,
      });
      const parsed = typeof rawResponse === 'string' ? JSON.parse(rawResponse) : rawResponse;

      // Check if response has the expected AgentRunResponse shape
      if (parsed && Array.isArray(parsed.changes) && typeof parsed.confidence === 'number') {
        response = parsed as AgentRunResponse;
      } else {
        // Service returned dispatch confirmation but no real result — unavailable
        this.workspace.log('Service returned dispatch without real result — agent unavailable');
        return {
          workspaceId: this.workspace.id,
          taskId: task.frontmatter.id,
          available: false,
          reason: 'AI service dispatched but did not return a structured response. Agent unavailable.',
        };
      }
    } catch (e) {
      // AI service error — return unavailable, never fabricate
      this.workspace.log(`AI service error: ${e}`);
      return {
        workspaceId: this.workspace.id,
        taskId: task.frontmatter.id,
        available: false,
        reason: `AI service unavailable: ${e instanceof Error ? e.message : String(e)}`,
      };
    }

    // 3. Resolve gate action
    const gateAction = this.workspace.gate.resolve(
      response.confidence,
      response.hasBreakingChange,
    );

    this.workspace.log(`Gate: ${gateAction.type} (confidence=${response.confidence})`);

    // 4. Return result — no auto-commit, no auto-apply
    // All patch application must go through apply_agent_patch_cmd
    // which validates, writes atomically, and records audit trail.
    return {
      workspaceId: this.workspace.id,
      taskId: task.frontmatter.id,
      changes: response.changes,
      confidence: response.confidence,
      hasBreakingChange: response.hasBreakingChange,
      commitHash: null,
      gateAction: gateAction.type,
      summary: response.summary,
    };
  }
}
