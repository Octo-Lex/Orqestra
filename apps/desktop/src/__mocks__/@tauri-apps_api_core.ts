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
        title: "Design system architecture",
        status: "in-review",
        priority: "Critical",
        sprint: "Sprint-1",
        epic: "Core",
        assignee: "architect",
        progress: 85,
        dependencies: [],
        blocks: ["TASK-2026-040", "TASK-2026-042"],
        labels: ["architecture"],
        time_estimate: 480,
        time_logged: 360,
        due_date: "2026-06-15",
        start_date: "2026-05-01",
        created: "2026-05-01",
        updated: "2026-06-01",
      },
      body: { context: "System architecture design", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-038.md",
    },
    {
      frontmatter: {
        id: "TASK-2026-050",
        title: "Implement rate limiting middleware",
        status: "in-progress",
        priority: "High",
        sprint: "Sprint-2",
        epic: "Security",
        assignee: "backend",
        progress: 30,
        dependencies: ["TASK-2026-038"],
        blocks: [],
        labels: ["security"],
        time_estimate: 240,
        time_logged: 60,
        due_date: "2026-06-20",
        start_date: "2026-06-01",
        created: "2026-06-01",
        updated: "2026-06-01",
      },
      body: { context: "Rate limiting for API protection", acceptance_criteria: [], agent_notes: null, raw: "" },
      source_path: "roadmap/TASK-2026-050.md",
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
  return { success: true };
}
