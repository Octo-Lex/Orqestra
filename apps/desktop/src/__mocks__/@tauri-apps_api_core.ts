// Mock @tauri-apps/api/core for browser testing
// Replaces Tauri IPC with JavaScript implementations

// Re-export things that other Tauri plugins expect
export class Resource {
  get rid() { return 0; }
}

const MOCK_INDEX = {
  tasks: [
    {
      frontmatter: {
        id: "TASK-2026-038",
        title: "Refactor DB layer to use connection pooling",
        status: "done",
        priority: "High",
        sprint: "Sprint-13",
        epic: "Security Hardening",
        assignee: "agent-architect",
        progress: 100,
        dependencies: [],
        blocks: ["TASK-2026-042"],
        labels: ["backend", "database"],
        time_estimate: 480,
        time_logged: 420,
        due_date: "2026-05-30",
        start_date: "2026-05-15",
        created: "2026-05-15",
        updated: "2026-05-28",
      },
      body: { context: "DB connection pooling", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-038.md",
    },
    {
      frontmatter: {
        id: "TASK-2026-040",
        title: "Add caching layer with Redis",
        status: "done",
        priority: "High",
        sprint: "Sprint-13",
        epic: "Security Hardening",
        assignee: "agent-backend",
        progress: 100,
        dependencies: [],
        blocks: ["TASK-2026-042"],
        labels: ["backend", "caching"],
        time_estimate: 360,
        time_logged: 300,
        due_date: "2026-06-01",
        start_date: "2026-05-20",
        created: "2026-05-20",
        updated: "2026-05-31",
      },
      body: { context: "Redis caching layer", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-040.md",
    },
    {
      frontmatter: {
        id: "TASK-2026-042",
        title: "Implement JWT auth refresh rotation",
        status: "in-progress",
        priority: "Critical",
        sprint: "Sprint-14",
        epic: "Security Hardening",
        assignee: "agent-architect",
        progress: 37,
        dependencies: ["TASK-2026-038", "TASK-2026-040"],
        blocks: ["TASK-2026-045", "TASK-2026-050"],
        labels: ["backend", "auth"],
        time_estimate: 480,
        time_logged: 180,
        due_date: "2026-06-15",
        start_date: "2026-06-01",
        created: "2026-05-28",
        updated: "2026-06-01",
      },
      body: { context: "JWT refresh token rotation", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-042.md",
    },
    {
      frontmatter: {
        id: "TASK-2026-045",
        title: "Update API documentation for auth changes",
        status: "ready",
        priority: "Medium",
        sprint: "Sprint-14",
        epic: "Security Hardening",
        assignee: null,
        progress: 0,
        dependencies: ["TASK-2026-042"],
        blocks: [],
        labels: ["docs"],
        time_estimate: 240,
        time_logged: 0,
        due_date: "2026-06-20",
        start_date: null,
        created: "2026-06-01",
        updated: "2026-06-01",
      },
      body: { context: "API docs update", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-045.md",
    },
    {
      frontmatter: {
        id: "TASK-2026-050",
        title: "Implement rate limiting middleware",
        status: "backlog",
        priority: "High",
        sprint: "Sprint-15",
        epic: "Security Hardening",
        assignee: null,
        progress: 0,
        dependencies: ["TASK-2026-042"],
        blocks: [],
        labels: ["backend", "security"],
        time_estimate: 240,
        time_logged: 0,
        due_date: "2026-06-25",
        start_date: null,
        created: "2026-06-01",
        updated: "2026-06-01",
      },
      body: { context: "Rate limiting", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-050.md",
    },
    {
      frontmatter: {
        id: "TASK-2026-055",
        title: "Write ADR for connection pool architecture",
        status: "ready",
        priority: "High",
        sprint: "Sprint-14",
        epic: "Architecture",
        assignee: null,
        progress: 0,
        dependencies: [],
        blocks: [],
        labels: ["architecture", "adr"],
        time_estimate: 120,
        time_logged: 0,
        due_date: "2026-06-10",
        start_date: null,
        created: "2026-06-01",
        updated: "2026-06-01",
      },
      body: { context: "ADR for connection pooling", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-055.md",
    },
    {
      frontmatter: {
        id: "TASK-2026-056",
        title: "Fix null pointer dereference in handler",
        status: "ready",
        priority: "Critical",
        sprint: "Sprint-14",
        epic: "Bugfix",
        assignee: null,
        progress: 0,
        dependencies: [],
        blocks: [],
        labels: ["bug", "regression"],
        time_estimate: 60,
        time_logged: 0,
        due_date: "2026-06-08",
        start_date: null,
        created: "2026-06-01",
        updated: "2026-06-01",
      },
      body: { context: "Null pointer in handler", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-056.md",
    },
    {
      frontmatter: {
        id: "TASK-2026-057",
        title: "Update README with workspace documentation",
        status: "ready",
        priority: "Medium",
        sprint: "Sprint-14",
        epic: "Documentation",
        assignee: null,
        progress: 0,
        dependencies: [],
        blocks: [],
        labels: ["docs", "readme"],
        time_estimate: 60,
        time_logged: 0,
        due_date: "2026-06-09",
        start_date: null,
        created: "2026-06-01",
        updated: "2026-06-01",
      },
      body: { context: "README update", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-057.md",
    },
  ],
  warnings: [],
};

let mockCommitHash = "mock-" + Date.now().toString(16);

export async function invoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
  console.log("[MOCK-TAURI]", cmd, args);

  if (cmd === "index_roadmap_cmd") {
    return MOCK_INDEX;
  }
  if (cmd === "get_task") {
    return null;
  }
  if (cmd === "update_task_status_cmd") {
    const { taskId, newStatus } = args as { taskId: string; newStatus: string };
    // Update in-memory mock data
    const task = MOCK_INDEX.tasks.find((t: { frontmatter: { id: string } }) => t.frontmatter.id === taskId);
    if (task) {
      task.frontmatter.status = newStatus as never;
      console.log(`[MOCK] Updated ${taskId} to ${newStatus}`);
    }
    return { success: true, new_status: newStatus };
  }
  if (cmd === "semantic_commit_cmd") {
    mockCommitHash = "mock-" + Date.now().toString(16);
    return {
      hash: mockCommitHash,
      stub_path: `.Orqestra/graph/commits/${mockCommitHash}.json`,
    };
  }
  if (cmd === "backfill_cmd") {
    return {
      confidence: 0.95,
      intent_summary: "Advance task status for UI verification via browser test",
      reasoning_trace_id: `trace-${Date.now().toString(16)}`,
    };
  }
  if (cmd === "git_pull_roadmap" || cmd === "git_push_roadmap") {
    return { success: true, stdout: "OK", stderr: "" };
  }

  // Agent workspace commands
  if (cmd === "list_workspaces_cmd") {
    return [
      { dir: "architect", id: "agent-architect" },
      { dir: "bugfix", id: "agent-bugfix" },
      { dir: "docs", id: "agent-docs" },
    ];
  }
  if (cmd === "read_file_cmd") {
    const { path } = args as { path: string; projectRoot?: string };
    if (path.includes("workspace.yml")) {
      if (path.includes("architect")) {
        return `id: agent-architect\nmodel: glm-5.1\nskills:\n  - ./skills/documentation/SKILL.md\nconfidence_gate:\n  auto_commit: 0.95\n  propose: 0.85\n  flag: 0.70\n  breaking_change_override: always_propose`;
      }
      if (path.includes("bugfix")) {
        return `id: agent-bugfix\nmodel: glm-5.1\nskills:\n  - ./skills/debugging/SKILL.md\n  - ./skills/testing/SKILL.md\nconfidence_gate:\n  auto_commit: 0.85\n  propose: 0.65\n  flag: 0.40\n  breaking_change_override: always_propose`;
      }
      if (path.includes("docs")) {
        return `id: agent-docs\nmodel: glm-5.1\nskills:\n  - ./skills/documentation/SKILL.md\nconfidence_gate:\n  auto_commit: 0.80\n  propose: 0.60\n  flag: 0.40\n  breaking_change_override: always_propose`;
      }
    }
    if (path.includes("SKILL.md")) {
      return `# Skill\nPurpose: Mock skill for testing.\nSteps:\n1. Do the thing`; 
    }
    return "";
  }
  if (cmd === "write_file_cmd") {
    const _args = args as { path: string; projectRoot?: string; content?: string };
    console.log(`[MOCK] Write: ${JSON.stringify(_args).substring(0, 200)}`);
    return null;
  }
  if (cmd === "run_agent_cmd") {
    return JSON.stringify({
      workspace_id: (args as Record<string, unknown>).workspaceId,
      task_id: (args as Record<string, unknown>).taskId,
      status: "dispatched",
      message: "Mock dispatch",
    });
  }

  // Phase 4: Graph / History commands
  if (cmd === "index_graph_cmd") {
    return { indexed: 2, total_triples: 20 };
  }
  if (cmd === "query_graph_cmd") {
    const { predicate } = args as { predicate?: string; subject?: string; object?: string };
    if (predicate === "has_intent") {
      return [
        {
          uuid: "mock-uuid-1",
          subject: "8d9343ab95a6",
          predicate: "has_intent",
          object: "Advance the status of task TASK-2026-050 (Implement rate limiting middleware) from 'backlog' to 'in-progress' via an automated pipeline.",
          commit: "8d9343ab95a6",
          timestamp: "2026-06-01T10:15:58Z",
        },
        {
          uuid: "mock-uuid-2",
          subject: "38e5fceb600b",
          predicate: "has_intent",
          object: "Reverts task TASK-2026-045 status from 'in-review' back to 'backlog' as part of testing the semantic commit pipeline.",
          commit: "38e5fceb600b",
          timestamp: "2026-06-01T09:11:11Z",
        },
      ];
    }
    return [];
  }
  if (cmd === "query_history_cmd") {
    return {
      answer: "Best match (score 0.286):\n  Commit: 8d9343ab95a625d33dacb323f74e0b994ef25288\n  Message: feat(ui): advance TASK-2026-050 status via pipeline\n  Intent: Advance the status of task TASK-2026-050 (Implement rate limiting middleware) from 'backlog' to 'in-progress' via an automated pipeline.\n  Confidence: 1.00\n  Concepts: task management, rate limiting middleware, sprint planning, security hardening\n  Tasks: TASK-2026-050\n  Reasoning trace:\n    The commit message explicitly states this is a pipeline-driven status advancement for task TASK-2026-050. The diff confirms this by changing a single field ('status') in a task markdown file from 'backlog' to 'in-progress'. This is a metadata-only change with no code, API, or configuration alterations, making it low risk and trivially reversible.",
      supporting_commits: [
        "8d9343ab95a625d33dacb323f74e0b994ef25288",
        "38e5fceb600b401053487eaed102978731070823",
      ],
    };
  }
  if (cmd === "read_commit_stub_cmd") {
    const { hash } = args as { hash: string };
    if (hash.startsWith("8d9343ab")) {
      return {
        hash: "8d9343ab95a625d33dacb323f74e0b994ef25288",
        conventional_message: "feat(ui): advance TASK-2026-050 status via pipeline",
        timestamp: "2026-06-01T10:15:58Z",
        author: { name: "orqestra-e2e", type: "human" },
        semantic: {
          status: "complete",
          intent_summary: "Advance the status of task TASK-2026-050 (Implement rate limiting middleware) from 'backlog' to 'in-progress' via an automated pipeline.",
          affected_concepts: ["task management", "rate limiting middleware", "sprint planning", "security hardening"],
          affected_apis: [],
          confidence: 1.0,
          reasoning_trace_id: "b907d8fc-c241-4401-b86a-322c7c8fa3e4",
          task_ids: ["TASK-2026-050"],
          risk_assessment: { breaking_change: false, migration_required: null, rollback_complexity: "low" },
        },
      };
    }
    if (hash.startsWith("38e5fceb")) {
      return {
        hash: "38e5fceb600b401053487eaed102978731070823",
        conventional_message: "test(phase-1): verify semantic commit pipeline",
        timestamp: "2026-06-01T09:11:11Z",
        author: { name: "orqestra-test", type: "human" },
        semantic: {
          status: "complete",
          intent_summary: "Reverts task TASK-2026-045 status from 'in-review' back to 'backlog' as part of testing the semantic commit pipeline.",
          affected_concepts: ["task management", "semantic commit pipeline", "project workflow"],
          affected_apis: [],
          confidence: 0.95,
          reasoning_trace_id: "a61d1f98-0a76-4697-8e93-e7934495a87a",
          task_ids: ["TASK-2026-045"],
          risk_assessment: { breaking_change: false, migration_required: null, rollback_complexity: "low" },
        },
      };
    }
    throw new Error(`Commit not found: ${hash}`);
  }
  if (cmd === "read_trace_cmd") {
    const { traceId } = args as { traceId: string };
    if (traceId === "b907d8fc-c241-4401-b86a-322c7c8fa3e4") {
      return "The commit message explicitly states this is a pipeline-driven status advancement for task TASK-2026-050. The diff confirms this by changing a single field ('status') in a task markdown file from 'backlog' to 'in-progress'. This is a metadata-only change with no code, API, or configuration alterations, making it low risk and trivially reversible.";
    }
    if (traceId === "a61d1f98-0a76-4697-8e93-e7934495a87a") {
      return "The commit message explicitly states this is a test to verify a semantic commit pipeline. The diff shows a single change to a markdown file that tracks a project management task, moving its status field backward from 'in-review' to 'backlog'. This is a metadata-only change with no code or API impact, consistent with a pipeline verification test.";
    }
    throw new Error(`Trace not found: ${traceId}`);
  }

  // Docs agent (real execution path)
  if (cmd === "run_docs_agent_cmd") {
    const { task } = (args as { task: string; context_files: string });
    const taskObj = JSON.parse(task);
    return JSON.stringify({
      summary: `Documentation update for ${taskObj.title || 'task'}`,
      confidence: 0.78,
      hasBreakingChange: false,
      edits: [
        {
          path: "README.md",
          before: "## Quick Start",
          after: "## Quick Start\n\n> **Note:** This section was updated by the Orqestra docs agent.",
        },
      ],
      notes: ["Mock response — AI service not running in browser test mode."],
    });
  }

  // Phase 5: Sync / CRDT commands
  if (cmd === "init_sync_cmd") {
    const { projectRoot, masterToken } = args as { projectRoot: string; masterToken: string };
    console.log(`[MOCK] Init sync: ${projectRoot}, token=${masterToken.substring(0, 8)}...`);
    return {
      peer_id: Date.now(),
      open_docs: [],
    };
  }
  if (cmd === "open_crdt_doc_cmd") {
    const { filePath } = args as { filePath: string };
    console.log(`[MOCK] Open CRDT doc: ${filePath}`);
    return null;
  }
  if (cmd === "set_crdt_field_cmd") {
    const { payload } = args as { payload: { filePath: string; key: string; value: string } };
    console.log(`[MOCK] Set CRDT field: ${payload.filePath}/${payload.key} = ${payload.value}`);
    return null;
  }
  if (cmd === "get_crdt_field_cmd") {
    return "mock-value";
  }
  if (cmd === "get_all_fields_cmd") {
    const { filePath } = args as { filePath: string };
    if (filePath.includes("TASK-2026-042")) {
      return [
        { key: "title", value: "Rate limiter v2" },
        { key: "status", value: "in-progress" },
        { key: "assignee", value: "alice" },
        { key: "priority", value: "critical" },
      ];
    }
    return [];
  }
  if (cmd === "export_delta_cmd") {
    return Array.from(new Uint8Array([1, 2, 3, 4, 5])); // Mock delta bytes
  }
  if (cmd === "import_delta_cmd") {
    console.log("[MOCK] Import delta");
    return null;
  }
  if (cmd === "load_markdown_cmd" || cmd === "export_markdown_cmd") {
    return null;
  }
  if (cmd === "save_snapshot_cmd") {
    return null;
  }
  if (cmd === "sync_status_cmd") {
    return {
      peer_id: Date.now(),
      open_docs: [
        "roadmap/TASK-2026-038.md",
        "roadmap/TASK-2026-040.md",
        "roadmap/TASK-2026-042.md",
      ],
    };
  }
  if (cmd === "generate_token_cmd") {
    const { request } = args as { request: { scope: string; label: string } };
    const ts = Date.now().toString(16);
    return `ork_${request.scope}_${ts}`;
  }
  if (cmd === "validate_token_cmd") {
    const { request } = args as { request: { token: string } };
    const token = request.token;
    if (token.startsWith("ork_write_")) {
      return { authorized: true, scope: { Admin: false, Write: true, Read: false } };
    }
    if (token.startsWith("ork_read_")) {
      return { authorized: true, scope: { Admin: false, Write: false, Read: true } };
    }
    if (token === "master-secret") {
      return { authorized: true, scope: { Admin: true, Write: true, Read: true } };
    }
    return { authorized: false, reason: "Invalid token" };
  }

  // Credential commands (encrypted vault)
  if (cmd === "save_github_token_cmd") {
    return null;
  }
  if (cmd === "get_github_token_cmd") {
    return "mock-github-pat-token";
  }
  if (cmd === "get_github_token_status_cmd") {
    return { exists: true, provider: "encrypted-vault", label: "GitHub PAT", lastUpdated: null };
  }
  if (cmd === "delete_github_token_cmd") {
    return null;
  }
  if (cmd === "migrate_github_token_cmd") {
    return { exists: true, provider: "encrypted-vault", label: "GitHub PAT", lastUpdated: null };
  }

  return { success: true };
}
