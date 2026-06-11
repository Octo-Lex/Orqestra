import React from 'react';

interface ExternalBetaEvidenceData {
  schema_version?: number;
  status?: string;
  external_beta_user_data?: boolean;
  intake_mechanism?: string;
  automatic_upload?: boolean;
  consent_required?: boolean;
  redaction_required?: boolean;
}

interface ExternalBetaReviewData {
  schema_version?: number;
  status?: string;
  external_beta_user_data?: boolean;
  reviewed_bundle_count?: number;
  accepted_bundle_count?: number;
  rejected_bundle_count?: number;
  needs_follow_up_count?: number;
  aggregate_only?: boolean;
}

interface Props {
  externalBetaEvidence?: ExternalBetaEvidenceData | null;
  externalBetaReview?: ExternalBetaReviewData | null;
}

export const ExternalBetaEvidencePanel: React.FC<Props> = ({ externalBetaEvidence, externalBetaReview }) => {
  if (!externalBetaEvidence && !externalBetaReview) {
    return (
      <div style={styles.panel}>
        <h3 style={styles.title}>External Beta Evidence</h3>
        <div style={styles.unavailable}>
          External beta data unavailable
        </div>
        <div style={styles.hint}>
          Evidence intake mechanism not yet deployed.
        </div>
      </div>
    );
  }

  const evidenceStatus = externalBetaEvidence?.status || 'none';
  const reviewStatus = externalBetaReview?.status || 'none';
  const hasData = externalBetaEvidence?.external_beta_user_data === true;

  return (
    <div style={styles.panel}>
      <h3 style={styles.title}>External Beta Evidence</h3>

      <div style={styles.row}>
        <span style={styles.label}>Status</span>
        <span style={{
          ...styles.value,
          color: hasData ? '#22c55e' : '#94a3b8',
        }}>
          {hasData ? 'Present' : 'None'}
        </span>
      </div>

      <div style={styles.row}>
        <span style={styles.label}>Collection mode</span>
        <span style={styles.value}>
          {externalBetaEvidence?.intake_mechanism || 'local_export_only'}
        </span>
      </div>

      <div style={styles.row}>
        <span style={styles.label}>Consent required</span>
        <span style={{
          ...styles.value,
          color: externalBetaEvidence?.consent_required ? '#22c55e' : '#ef4444',
        }}>
          {externalBetaEvidence?.consent_required ? 'Yes' : 'No'}
        </span>
      </div>

      <div style={styles.row}>
        <span style={styles.label}>Automatic upload</span>
        <span style={{
          ...styles.value,
          color: externalBetaEvidence?.automatic_upload ? '#ef4444' : '#22c55e',
        }}>
          {externalBetaEvidence?.automatic_upload ? 'Yes' : 'No'}
        </span>
      </div>

      <div style={styles.row}>
        <span style={styles.label}>Redaction required</span>
        <span style={{
          ...styles.value,
          color: externalBetaEvidence?.redaction_required ? '#22c55e' : '#ef4444',
        }}>
          {externalBetaEvidence?.redaction_required ? 'Yes' : 'No'}
        </span>
      </div>

      <div style={styles.row}>
        <span style={styles.label}>Public data</span>
        <span style={styles.value}>Aggregate only</span>
      </div>

      {/* Review section */}
      {externalBetaReview && hasData && (
        <>
          <div style={styles.sectionTitle}>Evidence Review</div>
          <div style={styles.row}>
            <span style={styles.label}>Reviewed bundles</span>
            <span style={styles.value}>{externalBetaReview.reviewed_bundle_count ?? 0}</span>
          </div>
          <div style={styles.row}>
            <span style={styles.label}>Accepted bundles</span>
            <span style={styles.value}>{externalBetaReview.accepted_bundle_count ?? 0}</span>
          </div>
          <div style={styles.row}>
            <span style={styles.label}>Rejected bundles</span>
            <span style={styles.value}>{externalBetaReview.rejected_bundle_count ?? 0}</span>
          </div>
          <div style={styles.row}>
            <span style={styles.label}>Needs follow-up</span>
            <span style={styles.value}>{externalBetaReview.needs_follow_up_count ?? 0}</span>
          </div>
        </>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: {
    backgroundColor: '#1e293b',
    borderRadius: '8px',
    padding: '16px',
  },
  title: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#f1f5f9',
    margin: '0 0 12px 0',
  },
  sectionTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#64748b',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    margin: '12px 0 8px 0',
  },
  row: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '4px 0',
    fontSize: '13px',
  },
  label: {
    color: '#94a3b8',
  },
  value: {
    color: '#e2e8f0',
    fontWeight: 500,
  },
  unavailable: {
    color: '#94a3b8',
    fontSize: '13px',
  },
  hint: {
    color: '#64748b',
    fontSize: '12px',
    marginTop: '4px',
  },
};
