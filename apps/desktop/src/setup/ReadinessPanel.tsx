import React, { useEffect, useState } from 'react';
import { getReadiness } from '../lib/readiness';
import type { ReadinessReport } from '../lib/readiness';
import { ReadinessCard } from './ReadinessCard';

interface Props {
  projectRoot?: string;
}

export const ReadinessPanel: React.FC<Props> = ({ projectRoot }) => {
  const [report, setReport] = useState<ReadinessReport | null>(null);

  useEffect(() => {
    getReadiness(projectRoot).then(setReport).catch(() => {});
  }, [projectRoot]);

  if (!report) return <div style={{ color: '#94a3b8', padding: 16 }}>Loading readiness...</div>;

  return (
    <div style={styles.panel}>
      <h2 style={styles.title}>Setup &amp; Readiness</h2>
      <p style={styles.subtitle}>Environment status for your Orqestra workspace</p>

      <div style={styles.section}>
        <h3 style={styles.sectionTitle}>Application</h3>
        <ReadinessCard id="app" label="Orqestra" status="ok"
          summary={`v${report.app.version} on ${report.app.platform}`} />
      </div>

      <div style={styles.section}>
        <h3 style={styles.sectionTitle}>Local Tools</h3>
        {report.local_tools.map(t => (
          <ReadinessCard
            key={t.tool}
            id={`tool-${t.tool}`}
            label={t.tool}
            status={t.status === 'found' ? 'ok' : 'missing'}
            summary={t.version || t.status}
            details={t.required_for.join(', ')}
          />
        ))}
      </div>

      <div style={styles.section}>
        <h3 style={styles.sectionTitle}>AI Service</h3>
        <ReadinessCard
          id="ai-service"
          label="AI Service"
          status={report.ai.service_status === 'reachable' ? 'ok' : 'warning'}
          summary={report.ai.service_status}
          details={report.ai.mode}
        />
        <ReadinessCard
          id="ai-key"
          label="API Key"
          status={report.ai.api_key_status === 'configured' ? 'ok' : 'warning'}
          summary={report.ai.api_key_status}
          details={report.ai.mode === 'real' ? 'Real AI enabled' : 'Degraded/mock mode'}
        />
      </div>

      <div style={styles.section}>
        <h3 style={styles.sectionTitle}>Credentials</h3>
        <ReadinessCard
          id="github-token"
          label="GitHub Token"
          status={report.credentials.github_token === 'stored' ? 'ok' : 'warning'}
          summary={report.credentials.github_token}
          details={`Provider: ${report.credentials.provider}`}
        />
      </div>

      <div style={styles.section}>
        <h3 style={styles.sectionTitle}>Dashboard</h3>
        <ReadinessCard
          id="dashboard-local"
          label="Local JSON"
          status={report.dashboard.local_json === 'present' ? 'ok' : 'warning'}
          summary={report.dashboard.local_json}
        />
        <ReadinessCard
          id="dashboard-live"
          label="Live Dashboard"
          status={report.dashboard.live_url_status === 'ok' ? 'ok' : 'warning'}
          summary={report.dashboard.live_url_status}
        />
      </div>

      {report.warnings.length > 0 && (
        <div style={styles.section}>
          <h3 style={styles.sectionTitle}>Warnings</h3>
          {report.warnings.map((w, i) => (
            <ReadinessCard
              key={i}
              id={`warning-${i}`}
              label={w.code}
              status={w.severity === 'error' ? 'error' : 'warning'}
              summary={w.message}
              details={w.recovery}
            />
          ))}
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: {
    padding: '16px',
    backgroundColor: '#1e293b',
    borderRadius: '8px',
    color: '#e2e8f0',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
  title: {
    fontSize: '18px',
    fontWeight: 600,
    margin: '0 0 4px 0',
    color: '#f1f5f9',
  },
  subtitle: {
    fontSize: '13px',
    color: '#94a3b8',
    margin: '0 0 16px 0',
  },
  section: {
    marginBottom: '16px',
  },
  sectionTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#94a3b8',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    margin: '0 0 8px 0',
  },
};
