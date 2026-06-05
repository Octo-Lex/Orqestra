/**
 * ArchitectAgentPanel — v1.9.0 read-only planning display.
 *
 * Shows architect plan output: summary, approach, risks, symbols, criteria, test strategy.
 * NO accept/reject patch buttons — architect is display-only.
 * The plan cannot be passed to apply_agent_patch_cmd.
 */

import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SymbolRef {
  name: string;
  kind: string;
  file: string;
  is_public: boolean;
}

interface RiskItem {
  risk: string;
  severity: string;
  mitigation: string;
}

interface TaskBreakdownItem {
  task: string;
  scope: string;
  complexity: string;
}

interface ArchitectPlanResult {
  plan_id: string;
  schema_version: string;
  summary: string;
  context_analysis: string;
  proposed_approach: string[];
  affected_symbols: SymbolRef[];
  risk_assessment: RiskItem[];
  dependency_warnings: string[];
  acceptance_criteria: string[];
  test_strategy: string[];
  task_breakdown: TaskBreakdownItem[];
  adr_draft: string | null;
  confidence: number;
}

interface ArchitectAgentResult {
  plan: ArchitectPlanResult;
  agent: string;
  mode: string;
  timestamp: string;
  error?: string;
}

interface Props {
  projectRoot: string;
  task: {
    id: string;
    title: string;
    labels: string[];
    source_path?: string;
  } | null;
}

export const ArchitectAgentPanel: React.FC<Props> = ({ projectRoot, task }) => {
  const [result, setResult] = useState<ArchitectAgentResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAdr, setShowAdr] = useState(false);

  if (!task) {
    return (
      <div className="text-sm text-gray-500 p-4">
        Select an architecture-labeled task to run the architect agent.
      </div>
    );
  }

  const runArchitect = async () => {
    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const taskJson = JSON.stringify({
        id: task.id,
        title: task.title,
        labels: task.labels,
        source_path: task.source_path,
      });

      const response = await invoke<string>('run_architect_agent_cmd', {
        projectRoot,
        task: taskJson,
      });

      const parsed = JSON.parse(response) as ArchitectAgentResult;
      setResult(parsed);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const severityColor = (s: string) => {
    switch (s) {
      case 'high': return '#ef4444';
      case 'medium': return '#f59e0b';
      case 'low': return '#22c55e';
      default: return '#6b7280';
    }
  };

  const complexityColor = (c: string) => {
    switch (c) {
      case 'high': return '#ef4444';
      case 'medium': return '#f59e0b';
      case 'low': return '#22c55e';
      default: return '#6b7280';
    }
  };

  return (
    <div className="border rounded-lg p-4 mb-4">
      <div className="flex items-center gap-2 mb-3">
        <h3 className="font-semibold">Architect Agent</h3>
        <span className="text-xs bg-blue-100 text-blue-700 px-2 py-0.5 rounded-full">
          read-only planner
        </span>
        <span className="text-xs bg-gray-100 text-gray-500 px-2 py-0.5 rounded-full">
          proposal — not implementation
        </span>
      </div>

      <div className="text-sm text-gray-600 mb-3">
        Task: {task.title} ({task.id})
      </div>

      {!result && !loading && !error && (
        <button
          onClick={runArchitect}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          disabled={loading}
        >
          Generate Plan
        </button>
      )}

      {loading && (
        <div className="text-sm text-gray-500">
          Architect is analyzing the task and repository context...
        </div>
      )}

      {error && (
        <div className="text-sm text-red-500 mb-3">
          Error: {error}
        </div>
      )}

      {result && (
        <div>
          {/* Header */}
          <div className="flex items-center gap-3 mb-3">
            <span className="text-xs text-gray-400">
              Plan ID: {result.plan.plan_id}
            </span>
            <span className="text-xs text-gray-400">
              Schema: {result.plan.schema_version}
            </span>
            <span style={{
              fontSize: 12,
              color: result.plan.confidence >= 0.8 ? '#22c55e' : result.plan.confidence >= 0.6 ? '#f59e0b' : '#ef4444'
            }}>
              Confidence: {(result.plan.confidence * 100).toFixed(0)}%
            </span>
          </div>

          {/* Summary */}
          <div className="mb-3">
            <h4 className="text-sm font-semibold mb-1">Summary</h4>
            <p className="text-sm text-gray-700">{result.plan.summary}</p>
          </div>

          {/* Context Analysis */}
          <div className="mb-3">
            <h4 className="text-sm font-semibold mb-1">Context Analysis</h4>
            <p className="text-sm text-gray-600">{result.plan.context_analysis}</p>
          </div>

          {/* Proposed Approach */}
          <div className="mb-3">
            <h4 className="text-sm font-semibold mb-1">Proposed Approach</h4>
            <ol className="text-sm text-gray-700 list-decimal list-inside">
              {result.plan.proposed_approach.map((step, i) => (
                <li key={i} className="mb-1">{step}</li>
              ))}
            </ol>
          </div>

          {/* Affected Symbols */}
          {result.plan.affected_symbols.length > 0 && (
            <div className="mb-3">
              <h4 className="text-sm font-semibold mb-1">Affected Symbols</h4>
              <table className="w-full text-xs">
                <thead>
                  <tr className="border-b">
                    <th className="text-left py-1">Name</th>
                    <th className="text-left py-1">Kind</th>
                    <th className="text-left py-1">File</th>
                    <th className="text-center py-1">Public</th>
                  </tr>
                </thead>
                <tbody>
                  {result.plan.affected_symbols.map((sym, i) => (
                    <tr key={i} className="border-b border-gray-100">
                      <td className="py-1 font-mono">{sym.name}</td>
                      <td className="py-1">{sym.kind}</td>
                      <td className="py-1 text-gray-500">{sym.file}</td>
                      <td className="py-1 text-center">{sym.is_public ? '✓' : '—'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* Risk Assessment */}
          {result.plan.risk_assessment.length > 0 && (
            <div className="mb-3">
              <h4 className="text-sm font-semibold mb-1">Risk Assessment</h4>
              {result.plan.risk_assessment.map((r, i) => (
                <div key={i} className="text-sm mb-1 flex gap-2">
                  <span style={{ color: severityColor(r.severity), fontWeight: 600, minWidth: 60 }}>{r.severity.toUpperCase()}</span>
                  <span className="text-gray-700">{r.risk}</span>
                  <span className="text-gray-400">— {r.mitigation}</span>
                </div>
              ))}
            </div>
          )}

          {/* Dependency Warnings */}
          {result.plan.dependency_warnings.length > 0 && (
            <div className="mb-3">
              <h4 className="text-sm font-semibold mb-1">Dependency Warnings</h4>
              {result.plan.dependency_warnings.map((w, i) => (
                <div key={i} className="text-sm text-amber-600 mb-1">⚠ {w}</div>
              ))}
            </div>
          )}

          {/* Acceptance Criteria */}
          <div className="mb-3">
            <h4 className="text-sm font-semibold mb-1">Acceptance Criteria</h4>
            {result.plan.acceptance_criteria.map((c, i) => (
              <div key={i} className="text-sm text-gray-700 mb-1">☐ {c}</div>
            ))}
          </div>

          {/* Test Strategy */}
          <div className="mb-3">
            <h4 className="text-sm font-semibold mb-1">Test Strategy</h4>
            <ol className="text-sm text-gray-700 list-decimal list-inside">
              {result.plan.test_strategy.map((s, i) => (
                <li key={i} className="mb-1">{s}</li>
              ))}
            </ol>
          </div>

          {/* Task Breakdown */}
          {result.plan.task_breakdown.length > 0 && (
            <div className="mb-3">
              <h4 className="text-sm font-semibold mb-1">Task Breakdown</h4>
              <table className="w-full text-xs">
                <thead>
                  <tr className="border-b">
                    <th className="text-left py-1">Task</th>
                    <th className="text-left py-1">Scope</th>
                    <th className="text-center py-1">Complexity</th>
                  </tr>
                </thead>
                <tbody>
                  {result.plan.task_breakdown.map((t, i) => (
                    <tr key={i} className="border-b border-gray-100">
                      <td className="py-1">{t.task}</td>
                      <td className="py-1 text-gray-500">{t.scope}</td>
                      <td className="py-1 text-center" style={{ color: complexityColor(t.complexity) }}>{t.complexity}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* ADR Draft */}
          {result.plan.adr_draft && (
            <div className="mb-3">
              <button
                onClick={() => setShowAdr(!showAdr)}
                className="text-sm text-blue-500 hover:underline"
              >
                {showAdr ? 'Hide' : 'Show'} ADR Draft
              </button>
              {showAdr && (
                <pre className="mt-2 p-3 bg-gray-50 rounded text-xs whitespace-pre-wrap font-mono">
                  {result.plan.adr_draft}
                </pre>
              )}
            </div>
          )}

          <div className="text-xs text-gray-400 mt-3">
            Architect output is a proposal. No files were modified.
          </div>
        </div>
      )}
    </div>
  );
};
