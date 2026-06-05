//! Operational Risk Classification — v2.4.0.
//!
//! Path-based classifier for high-leverage operational files.
//! Not a scanner: no file content parsing, no registry lookups.
//! Deterministic: same path always produces same risks.
//!
//! Key rules:
//! - classify_path returns Vec<OperationalRisk> (multi-risk per file)
//! - Highest severity determines enforcement
//! - CredentialOrSecretConfig: reject write outright
//! - blocks_auto_apply: future auto-apply forbidden, not human-apply impossible
//! - UnknownSensitiveConfig escalated in sensitive directories

use serde::Serialize;
use std::path::Path;

// ---------------------------------------------------------------------------
// Stable reason codes
// ---------------------------------------------------------------------------

pub const RISK_DEPENDENCY_VERSION_CHANGE: &str = "RISK_DEPENDENCY_VERSION_CHANGE";
pub const RISK_LOCKFILE_MODIFIED: &str = "RISK_LOCKFILE_MODIFIED";
pub const RISK_CI_WORKFLOW_MODIFIED: &str = "RISK_CI_WORKFLOW_MODIFIED";
pub const RISK_CLOUDFLARE_CONFIG_MODIFIED: &str = "RISK_CLOUDFLARE_CONFIG_MODIFIED";
pub const RISK_TAURI_CONFIG_MODIFIED: &str = "RISK_TAURI_CONFIG_MODIFIED";
pub const RISK_RELEASE_MANIFEST_MODIFIED: &str = "RISK_RELEASE_MANIFEST_MODIFIED";
pub const RISK_CREDENTIAL_OR_SECRET_PROXIMITY: &str = "RISK_CREDENTIAL_OR_SECRET_PROXIMITY";
pub const RISK_BUILD_CONFIG_MODIFIED: &str = "RISK_BUILD_CONFIG_MODIFIED";
pub const RISK_REPO_POLICY_MODIFIED: &str = "RISK_REPO_POLICY_MODIFIED";
pub const RISK_TOOLCHAIN_MODIFIED: &str = "RISK_TOOLCHAIN_MODIFIED";
pub const RISK_PACKAGE_MANAGER_CONFIG_MODIFIED: &str = "RISK_PACKAGE_MANAGER_CONFIG_MODIFIED";
pub const RISK_CONTAINER_CONFIG_MODIFIED: &str = "RISK_CONTAINER_CONFIG_MODIFIED";
pub const RISK_UNKNOWN_SENSITIVE_CONFIG: &str = "RISK_UNKNOWN_SENSITIVE_CONFIG";

/// All stable reason codes. Test covers this to prevent accidental renames.
pub const ALL_REASON_CODES: &[&str] = &[
    RISK_DEPENDENCY_VERSION_CHANGE,
    RISK_LOCKFILE_MODIFIED,
    RISK_CI_WORKFLOW_MODIFIED,
    RISK_CLOUDFLARE_CONFIG_MODIFIED,
    RISK_TAURI_CONFIG_MODIFIED,
    RISK_RELEASE_MANIFEST_MODIFIED,
    RISK_CREDENTIAL_OR_SECRET_PROXIMITY,
    RISK_BUILD_CONFIG_MODIFIED,
    RISK_REPO_POLICY_MODIFIED,
    RISK_TOOLCHAIN_MODIFIED,
    RISK_PACKAGE_MANAGER_CONFIG_MODIFIED,
    RISK_CONTAINER_CONFIG_MODIFIED,
    RISK_UNKNOWN_SENSITIVE_CONFIG,
];

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    DependencyManifest,
    DependencyLockfile,
    CiWorkflow,
    CloudflareConfig,
    TauriConfig,
    ReleaseManifest,
    CredentialOrSecretConfig,
    BuildConfig,
    RepoPolicyConfig,
    ToolchainConfig,
    PackageManagerConfig,
    ContainerConfig,
    UnknownSensitiveConfig,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RiskSeverity {
    Info = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OperationalRisk {
    pub path: String,
    pub category: RiskCategory,
    pub severity: RiskSeverity,
    pub reason_code: &'static str,
    pub requires_human_review: bool,
    pub blocks_auto_apply: bool,
}

impl OperationalRisk {
    /// Whether this path should be rejected outright (no human override).
    pub fn reject_outright(&self) -> bool {
        self.category == RiskCategory::CredentialOrSecretConfig
    }
}

// ---------------------------------------------------------------------------
// Sensitive directories for escalation
// ---------------------------------------------------------------------------

const SENSITIVE_DIRS: &[&str] = &[
    ".github",
    ".cloudflare",
    ".vscode",
    "scripts",
    "deploy",
    "infra",
];

fn is_in_sensitive_dir(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    for dir in SENSITIVE_DIRS {
        if normalized.starts_with(&format!("{}/", dir)) || normalized.starts_with(&format!("./{}/", dir)) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Classifier
// ---------------------------------------------------------------------------

/// Classify a file path into zero or more operational risks.
/// Multi-risk: a single path may match multiple categories.
/// Deterministic: same path always produces same risks.
pub fn classify_path(path: &str) -> Vec<OperationalRisk> {
    let normalized = path.replace('\\', "/");
    let filename = Path::new(&normalized)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    let lower = filename.to_lowercase();
    let mut risks = Vec::new();

    // --- Credential/Secret (Critical, reject outright) ---
    if lower.starts_with(".env")
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower.contains("credentials")
        || lower.ends_with(".secret")
    {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::CredentialOrSecretConfig,
            severity: RiskSeverity::Critical,
            reason_code: RISK_CREDENTIAL_OR_SECRET_PROXIMITY,
            requires_human_review: true,
            blocks_auto_apply: true,
        });
        return risks; // Credential paths don't produce other risks
    }

    // --- Release manifest (Critical) ---
    if lower == "release-manifest.json" {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::ReleaseManifest,
            severity: RiskSeverity::Critical,
            reason_code: RISK_RELEASE_MANIFEST_MODIFIED,
            requires_human_review: true,
            blocks_auto_apply: true,
        });
    }

    // --- CI workflows (High) ---
    if normalized.contains(".github/workflows/") || normalized.contains(".github/actions/") {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::CiWorkflow,
            severity: RiskSeverity::High,
            reason_code: RISK_CI_WORKFLOW_MODIFIED,
            requires_human_review: true,
            blocks_auto_apply: true,
        });
    }

    // --- Lockfiles (High) ---
    if lower == "cargo.lock"
        || lower.ends_with("-lock.json")
        || lower == "pnpm-lock.yaml"
        || lower == "yarn.lock"
    {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::DependencyLockfile,
            severity: RiskSeverity::High,
            reason_code: RISK_LOCKFILE_MODIFIED,
            requires_human_review: true,
            blocks_auto_apply: true,
        });
    }

    // --- Dependency manifests (Medium) ---
    if lower == "cargo.toml" || lower == "package.json" {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::DependencyManifest,
            severity: RiskSeverity::Medium,
            reason_code: RISK_DEPENDENCY_VERSION_CHANGE,
            requires_human_review: true,
            blocks_auto_apply: false,
        });
    }

    // --- Cloudflare config (Medium) ---
    if lower == "wrangler.toml" || lower == "wrangler.json" {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::CloudflareConfig,
            severity: RiskSeverity::Medium,
            reason_code: RISK_CLOUDFLARE_CONFIG_MODIFIED,
            requires_human_review: true,
            blocks_auto_apply: false,
        });
    }

    // --- Tauri config (Medium) ---
    if lower == "tauri.conf.json" {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::TauriConfig,
            severity: RiskSeverity::Medium,
            reason_code: RISK_TAURI_CONFIG_MODIFIED,
            requires_human_review: true,
            blocks_auto_apply: false,
        });
    }

    // --- Repo policy (Medium) ---
    if lower == "dependabot.yml"
        || lower == "dependabot.yaml"
        || lower == "codeowners"
        || normalized.ends_with(".github/codeowners")
        || normalized.ends_with(".github/CODEOWNERS")
    {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::RepoPolicyConfig,
            severity: RiskSeverity::Medium,
            reason_code: RISK_REPO_POLICY_MODIFIED,
            requires_human_review: true,
            blocks_auto_apply: false,
        });
    }

    // --- Toolchain config (Medium) ---
    if lower == "rust-toolchain.toml"
        || lower == "rust-toolchain"
        || normalized.contains(".cargo/config")
        || lower == "deny.toml"
        || lower == "clippy.toml"
    {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::ToolchainConfig,
            severity: RiskSeverity::Medium,
            reason_code: RISK_TOOLCHAIN_MODIFIED,
            requires_human_review: true,
            blocks_auto_apply: false,
        });
    }

    // --- Container config (Medium) ---
    if lower == "dockerfile"
        || lower.starts_with("dockerfile.")
        || lower == "docker-compose.yml"
        || lower == "docker-compose.yaml"
        || lower == "compose.yml"
        || lower == "compose.yaml"
    {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::ContainerConfig,
            severity: RiskSeverity::Medium,
            reason_code: RISK_CONTAINER_CONFIG_MODIFIED,
            requires_human_review: true,
            blocks_auto_apply: false,
        });
    }

    // --- Package manager config (Low) ---
    if lower == ".npmrc" || lower == ".yarnrc" || lower == ".yarnrc.yml" {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::PackageManagerConfig,
            severity: RiskSeverity::Low,
            reason_code: RISK_PACKAGE_MANAGER_CONFIG_MODIFIED,
            requires_human_review: false,
            blocks_auto_apply: false,
        });
    }

    // --- Build config (Low) ---
    if lower == "tsconfig.json"
        || lower.starts_with("tsconfig.")
        || lower.starts_with("vite.config.")
        || lower.starts_with("webpack.config.")
        || lower.starts_with("rollup.config.")
        || lower.starts_with("esbuild.")
    {
        risks.push(OperationalRisk {
            path: path.to_string(),
            category: RiskCategory::BuildConfig,
            severity: RiskSeverity::Low,
            reason_code: RISK_BUILD_CONFIG_MODIFIED,
            requires_human_review: false,
            blocks_auto_apply: false,
        });
    }

    // --- Unknown sensitive config (fallback) ---
    if risks.is_empty() {
        let is_config = lower.ends_with(".yml")
            || lower.ends_with(".yaml")
            || lower.ends_with(".toml")
            || lower.ends_with(".json")
            || lower.ends_with(".ini")
            || lower.ends_with(".conf")
            || lower.ends_with(".config");

        if is_config {
            let in_sensitive = is_in_sensitive_dir(path);
            risks.push(OperationalRisk {
                path: path.to_string(),
                category: RiskCategory::UnknownSensitiveConfig,
                severity: if in_sensitive { RiskSeverity::Medium } else { RiskSeverity::Low },
                reason_code: RISK_UNKNOWN_SENSITIVE_CONFIG,
                requires_human_review: in_sensitive,
                blocks_auto_apply: false,
            });
        }
    }

    risks
}

/// Compute the highest severity from a list of risks.
pub fn highest_severity(risks: &[OperationalRisk]) -> Option<RiskSeverity> {
    risks.iter().map(|r| r.severity).max()
}

/// Check if any risk requires outright rejection (credential/secret).
pub fn any_reject_outright(risks: &[OperationalRisk]) -> bool {
    risks.iter().any(|r| r.reject_outright())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Basic classification ---

    #[test]
    fn test_classify_cargo_toml() {
        let risks = classify_path("Cargo.toml");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::DependencyManifest);
        assert_eq!(risks[0].severity, RiskSeverity::Medium);
        assert_eq!(risks[0].reason_code, RISK_DEPENDENCY_VERSION_CHANGE);
        assert!(risks[0].requires_human_review);
        assert!(!risks[0].blocks_auto_apply);
    }

    #[test]
    fn test_classify_cargo_lock() {
        let risks = classify_path("Cargo.lock");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::DependencyLockfile);
        assert_eq!(risks[0].severity, RiskSeverity::High);
        assert_eq!(risks[0].reason_code, RISK_LOCKFILE_MODIFIED);
    }

    #[test]
    fn test_classify_package_json() {
        let risks = classify_path("package.json");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::DependencyManifest);
        assert_eq!(risks[0].severity, RiskSeverity::Medium);
    }

    #[test]
    fn test_classify_pnpm_lock() {
        let risks = classify_path("pnpm-lock.yaml");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::DependencyLockfile);
        assert_eq!(risks[0].severity, RiskSeverity::High);
    }

    #[test]
    fn test_classify_ci_workflow() {
        let risks = classify_path(".github/workflows/ci.yml");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::CiWorkflow);
        assert_eq!(risks[0].severity, RiskSeverity::High);
        assert_eq!(risks[0].reason_code, RISK_CI_WORKFLOW_MODIFIED);
        assert!(risks[0].blocks_auto_apply);
    }

    #[test]
    fn test_classify_wrangler() {
        let risks = classify_path("wrangler.toml");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::CloudflareConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Medium);
    }

    #[test]
    fn test_classify_tauri_config() {
        let risks = classify_path("tauri.conf.json");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::TauriConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Medium);
    }

    #[test]
    fn test_classify_release_manifest() {
        let risks = classify_path("release-manifest.json");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::ReleaseManifest);
        assert_eq!(risks[0].severity, RiskSeverity::Critical);
        assert!(risks[0].blocks_auto_apply);
    }

    #[test]
    fn test_classify_env_secret() {
        let risks = classify_path(".env");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::CredentialOrSecretConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Critical);
        assert!(risks[0].reject_outright());
    }

    #[test]
    fn test_classify_pem_key() {
        let risks = classify_path("server.pem");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::CredentialOrSecretConfig);
        assert!(risks[0].reject_outright());
    }

    #[test]
    fn test_classify_tsconfig() {
        let risks = classify_path("tsconfig.json");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::BuildConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Low);
    }

    #[test]
    fn test_classify_codeowners() {
        let risks = classify_path("CODEOWNERS");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::RepoPolicyConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Medium);
    }

    #[test]
    fn test_classify_dockerfile() {
        let risks = classify_path("Dockerfile");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::ContainerConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Medium);
    }

    #[test]
    fn test_classify_npmrc() {
        let risks = classify_path(".npmrc");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::PackageManagerConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Low);
    }

    #[test]
    fn test_classify_rust_toolchain() {
        let risks = classify_path("rust-toolchain.toml");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::ToolchainConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Medium);
    }

    // --- Unknown in sensitive dir ---

    #[test]
    fn test_classify_unknown_in_github_dir() {
        let risks = classify_path(".github/some-config.yml");
        assert!(risks.len() >= 1);
        // Should be at least UnknownSensitiveConfig Medium (or a specific category)
        let has_medium = risks.iter().any(|r| r.severity >= RiskSeverity::Medium);
        assert!(has_medium);
    }

    #[test]
    fn test_classify_unknown_config() {
        let risks = classify_path("random.yml");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::UnknownSensitiveConfig);
        assert_eq!(risks[0].severity, RiskSeverity::Low);
        assert!(!risks[0].requires_human_review);
    }

    // --- Multi-risk ---

    #[test]
    fn test_multi_risk_ci_workflow() {
        // .github/workflows/release.yml → CiWorkflow (High)
        let risks = classify_path(".github/workflows/release.yml");
        assert!(!risks.is_empty());
        assert!(risks.iter().any(|r| r.category == RiskCategory::CiWorkflow));
    }

    #[test]
    fn test_highest_severity_enforcement() {
        let risks = vec![
            OperationalRisk {
                path: "Cargo.toml".to_string(),
                category: RiskCategory::DependencyManifest,
                severity: RiskSeverity::Medium,
                reason_code: RISK_DEPENDENCY_VERSION_CHANGE,
                requires_human_review: true,
                blocks_auto_apply: false,
            },
            OperationalRisk {
                path: "Cargo.toml".to_string(),
                category: RiskCategory::BuildConfig,
                severity: RiskSeverity::Low,
                reason_code: RISK_BUILD_CONFIG_MODIFIED,
                requires_human_review: false,
                blocks_auto_apply: false,
            },
        ];
        assert_eq!(highest_severity(&risks), Some(RiskSeverity::Medium));
    }

    // --- Credential reject outright ---

    #[test]
    fn test_credential_reject_outright() {
        let risks = classify_path(".env.production");
        assert!(any_reject_outright(&risks));
    }

    #[test]
    fn test_blocks_auto_apply_not_human_reject() {
        let risks = classify_path(".github/workflows/ci.yml");
        assert!(!any_reject_outright(&risks)); // CI workflow blocks auto-apply but NOT outright reject
        assert!(risks[0].blocks_auto_apply); // But auto-apply is blocked
    }

    // --- Determinism ---

    #[test]
    fn test_risk_deterministic() {
        let r1 = classify_path("Cargo.toml");
        let r2 = classify_path("Cargo.toml");
        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.category, b.category);
            assert_eq!(a.severity, b.severity);
            assert_eq!(a.reason_code, b.reason_code);
        }
    }

    // --- No secret contents in DTO ---

    #[test]
    fn test_operational_risk_no_secret_contents() {
        let risk = OperationalRisk {
            path: ".env".to_string(),
            category: RiskCategory::CredentialOrSecretConfig,
            severity: RiskSeverity::Critical,
            reason_code: RISK_CREDENTIAL_OR_SECRET_PROXIMITY,
            requires_human_review: true,
            blocks_auto_apply: true,
        };
        let json = serde_json::to_string(&risk).unwrap();
        assert!(!json.contains("secret_value"));
        assert!(!json.contains("password"));
        assert!(!json.contains("token"));
    }

    // --- Reason-code stability ---

    #[test]
    fn test_reason_codes_are_stable() {
        assert_eq!(ALL_REASON_CODES.len(), 13);
        assert!(ALL_REASON_CODES.contains(&RISK_DEPENDENCY_VERSION_CHANGE));
        assert!(ALL_REASON_CODES.contains(&RISK_LOCKFILE_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_CI_WORKFLOW_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_CLOUDFLARE_CONFIG_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_TAURI_CONFIG_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_RELEASE_MANIFEST_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_CREDENTIAL_OR_SECRET_PROXIMITY));
        assert!(ALL_REASON_CODES.contains(&RISK_BUILD_CONFIG_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_REPO_POLICY_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_TOOLCHAIN_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_PACKAGE_MANAGER_CONFIG_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_CONTAINER_CONFIG_MODIFIED));
        assert!(ALL_REASON_CODES.contains(&RISK_UNKNOWN_SENSITIVE_CONFIG));
    }

    // --- Non-config files produce no risks ---

    #[test]
    fn test_non_config_no_risk() {
        let risks = classify_path("src/main.rs");
        assert!(risks.is_empty());
    }

    // --- Docker Compose variants ---

    #[test]
    fn test_classify_compose_yaml() {
        let risks = classify_path("compose.yaml");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::ContainerConfig);
    }

    // --- .cargo/config.toml ---

    #[test]
    fn test_classify_cargo_config() {
        let risks = classify_path(".cargo/config.toml");
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].category, RiskCategory::ToolchainConfig);
    }
}
