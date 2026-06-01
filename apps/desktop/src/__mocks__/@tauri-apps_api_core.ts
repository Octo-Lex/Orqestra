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
    const { path } = args as { path: string };
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
    console.log(`[MOCK] Write: ${JSON.stringify(args).substring(0, 200)}`);
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

  return { success: true };
}
