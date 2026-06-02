import React, { useEffect, useState } from 'react';
import { getRecoveryAdvice } from '../lib/diagnostics';
import type { RecoveryAdvice } from '../lib/diagnostics';

interface Props {
  code: string;
}

export const RecoveryCard: React.FC<Props> = ({ code }) => {
  const [advice, setAdvice] = useState<RecoveryAdvice | null>(null);

  useEffect(() => {
    getRecoveryAdvice(code).then(setAdvice).catch(() => {});
  }, [code]);

  if (!advice) return null;

  return (
    <div style={styles.card}>
      <strong style={styles.title}>{advice.title}</strong>
      <p style={styles.description}>{advice.description}</p>
      <span style={styles.action}>{advice.action_label}</span>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  card: {
    padding: '10px 12px',
    backgroundColor: '#0f172a',
    borderRadius: '6px',
    marginBottom: '6px',
  },
  title: {
    fontSize: '13px',
    color: '#e2e8f0',
  },
  description: {
    fontSize: '12px',
    color: '#94a3b8',
    margin: '4px 0',
    lineHeight: 1.4,
  },
  action: {
    fontSize: '11px',
    color: '#6366f1',
    fontWeight: 500,
  },
};
