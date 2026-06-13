//! Environment and integration readiness checks.
//!
//! Probes local tools, AI service, credentials, dashboard state, and release
//! artifacts. Returns a structured report for the UI to render as status cards.
//!
//! Security rule: Readiness DTOs must never include raw tokens, PATs, API keys,
//! secrets, password fields, or unlock secrets.

use serde::Serialize;
use std::process::Stdio;
use tauri::command;

use super::roadmap::CommandError;

type CommandResult<T> = Result<T, CommandError>;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ReadinessReport {
    pub generated_at: String,
    pub app: AppReadiness,
    pub project: Option<ProjectReadiness>,
    pub local_tools: Vec<ToolReadiness>,
    pub ai: AiReadiness,
    pub credentials: CredentialReadiness,
    pub dashboard: DashboardReadiness,
    pub release_artifacts: Vec<ReleaseArtifactReadiness>,
    pub warnings: Vec<ReadinessWarning>,
}

#[derive(Debug, Serialize)]
pub struct AppReadiness {
    pub version: String,
    pub git_sha: Option<String>,
    pub tauri_commands_registered: Option<usize>,
    pub platform: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectReadiness {
    pub root: String,
    pub status: String,
    pub task_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ToolReadiness {
    pub tool: String,
    pub status: String,
    pub version: Option<String>,
    pub required_for: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AiReadiness {
    pub service_status: String,
    pub health_url: String,
    pub api_key_status: String,
    pub mode: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CredentialReadiness {
    pub github_token: String,
    pub provider: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DashboardReadiness {
    pub local_json: String,
    pub live_url_status: String,
    pub source_commit: Option<String>,
    pub cloudflare_secrets: String,
}

#[derive(Debug, Serialize)]
pub struct ReleaseArtifactReadiness {
    pub platform: String,
    pub status: String,
    pub artifact_name: Option<String>,
    pub limitation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReadinessWarning {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub recovery: String,
}

// ---------------------------------------------------------------------------
// Readiness command
// ---------------------------------------------------------------------------

#[command]
pub async fn get_readiness_cmd(project_root: Option<String>) -> CommandResult<ReadinessReport> {
    // v2.14.11: Run on blocking thread pool to avoid freezing the UI.
    // The readiness checks spawn subprocesses and make HTTP requests
    // that can take 5-10 seconds total.
    tokio::task::spawn_blocking(move || {
        get_readiness_impl(project_root)
    })
    .await
    .map_err(|e| CommandError {
        code: "READINESS_THREAD_ERROR",
        message: format!("Readiness check thread failed: {}", e),
    })?}

pub fn get_readiness_impl(project_root: Option<String>) -> CommandResult<ReadinessReport> {
    let generated_at = chrono::Utc::now().to_rfc3339();

    // App info
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let app = AppReadiness {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha: option_env!("GIT_SHA").map(|s| s.to_string()),
        tauri_commands_registered: None,
        platform: platform.to_string(),
    };

    // Project readiness
    let project = project_root.as_ref().map(|root| {
        let roadmap_dir = std::path::Path::new(root).join("roadmap");
        let (status, task_count) = if roadmap_dir.is_dir() {
            match md_indexer::index_roadmap(&roadmap_dir) {
                Ok(result) => ("valid".to_string(), result.tasks.len()),
                Err(_) => ("error".to_string(), 0),
            }
        } else {
            ("not_loaded".to_string(), 0)
        };
        ProjectReadiness {
            root: root.clone(),
            status,
            task_count,
        }
    });

    // Local tools
    let local_tools = vec![
        check_tool("git", &["Git sync", "Semantic commits"]),
        check_tool("node", &["Dashboard build"]),
        check_tool("npm", &["Desktop build"]),
        check_tool("rustc", &["Core crate development"]),
        check_tool("cargo", &["Core crate development"]),
        check_tool("python", &["AI service"]),
    ];

    // AI readiness
    let ai = check_ai_readiness();

    // Credential readiness
    let credentials = check_credential_readiness();

    // Dashboard readiness
    let dashboard = check_dashboard_readiness(&project_root);

    // Release artifacts
    let release_artifacts = vec![
        ReleaseArtifactReadiness {
            platform: "windows".to_string(),
            status: "available".to_string(),
            artifact_name: Some("Orqestra_1.0.3_x64-setup.exe".to_string()),
            limitation: Some("Unsigned beta artifact".to_string()),
        },
        ReleaseArtifactReadiness {
            platform: "macos".to_string(),
            status: "not_checked".to_string(),
            artifact_name: None,
            limitation: Some("Requires macOS CI runner".to_string()),
        },
        ReleaseArtifactReadiness {
            platform: "linux".to_string(),
            status: "not_checked".to_string(),
            artifact_name: None,
            limitation: Some("Requires Linux CI runner with Tauri dependencies".to_string()),
        },
    ];

    // Warnings
    let mut warnings = Vec::new();

    if ai.mode != "real" {
        warnings.push(ReadinessWarning {
            code: "AI_DEGRADED".to_string(),
            severity: "info".to_string(),
            message: "AI service is in degraded or mock mode".to_string(),
            recovery: "Set ZAI_API_KEY environment variable and start the AI service".to_string(),
        });
    }

    if credentials.github_token == "missing" {
        warnings.push(ReadinessWarning {
            code: "GITHUB_TOKEN_MISSING".to_string(),
            severity: "warning".to_string(),
            message: "GitHub credential not stored".to_string(),
            recovery: "Save a GitHub PAT in the Credentials panel".to_string(),
        });
    }

    Ok(ReadinessReport {
        generated_at,
        app,
        project,
        local_tools,
        ai,
        credentials,
        dashboard,
        release_artifacts,
        warnings,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn check_tool(name: &str, required_for: &[&str]) -> ToolReadiness {
    let output = std::process::Command::new(name)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let version = stdout.lines().next().map(|s| s.trim().to_string());
            ToolReadiness {
                tool: name.to_string(),
                status: "found".to_string(),
                version,
                required_for: required_for.iter().map(|s| s.to_string()).collect(),
            }
        }
        Err(_) => ToolReadiness {
            tool: name.to_string(),
            status: "missing".to_string(),
            version: None,
            required_for: required_for.iter().map(|s| s.to_string()).collect(),
        },
    }
}

fn check_ai_readiness() -> AiReadiness {
    // Check if AI service is reachable
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build();

    let health_url = "http://localhost:8000/health".to_string();

    match client {
        Ok(client) => {
            let resp = client.get(&health_url).send();
            match resp {
                Ok(r) if r.status().is_success() => {
                    // Service is up — check if API key is configured
                    let has_key = std::env::var("ZAI_API_KEY")
                        .map(|v| !v.is_empty())
                        .unwrap_or(false);

                    if has_key {
                        AiReadiness {
                            service_status: "reachable".to_string(),
                            health_url,
                            api_key_status: "configured".to_string(),
                            mode: "real".to_string(),
                            last_error: None,
                        }
                    } else {
                        AiReadiness {
                            service_status: "reachable".to_string(),
                            health_url,
                            api_key_status: "missing".to_string(),
                            mode: "degraded_mock".to_string(),
                            last_error: Some("ZAI_API_KEY not set".to_string()),
                        }
                    }
                }
                Ok(r) => AiReadiness {
                    service_status: "reachable".to_string(),
                    health_url,
                    api_key_status: "unknown".to_string(),
                    mode: "degraded_mock".to_string(),
                    last_error: Some(format!("Health check returned status {}", r.status())),
                },
                Err(e) => AiReadiness {
                    service_status: "unreachable".to_string(),
                    health_url,
                    api_key_status: if std::env::var("ZAI_API_KEY").is_ok() {
                        "configured".to_string()
                    } else {
                        "missing".to_string()
                    },
                    mode: "unavailable".to_string(),
                    last_error: Some(mask_connection_error(&e.to_string())),
                },
            }
        }
        Err(_) => AiReadiness {
            service_status: "not_checked".to_string(),
            health_url,
            api_key_status: "unknown".to_string(),
            mode: "unavailable".to_string(),
            last_error: Some("HTTP client initialization failed".to_string()),
        },
    }
}

fn check_credential_readiness() -> CredentialReadiness {
    // Check keyring availability without exposing token values
    match crate::security::is_keyring_available() {
        true => {
            // Try to check if a GitHub token exists
            match crate::security::has_github_token() {
                Ok(true) => CredentialReadiness {
                    github_token: "stored".to_string(),
                    provider: "keyring".to_string(),
                    last_error: None,
                },
                Ok(false) => CredentialReadiness {
                    github_token: "missing".to_string(),
                    provider: "keyring".to_string(),
                    last_error: None,
                },
                Err(e) => CredentialReadiness {
                    github_token: "error".to_string(),
                    provider: "none".to_string(),
                    last_error: Some(mask_error(&e.to_string())),
                },
            }
        }
        false => CredentialReadiness {
            github_token: "not_checked".to_string(),
            provider: "none".to_string(),
            last_error: Some("OS keyring unavailable".to_string()),
        },
    }
}

fn check_dashboard_readiness(project_root: &Option<String>) -> DashboardReadiness {
    let local_json = match project_root {
        Some(root) => {
            let json_path = std::path::Path::new(root)
                .join("apps")
                .join("dashboard")
                .join("public")
                .join("roadmap.json");
            if json_path.exists() {
                "present".to_string()
            } else {
                "missing".to_string()
            }
        }
        None => "not_checked".to_string(),
    };

    // Check live URL
    let live_url_status = {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build();
        match client {
            Ok(client) => {
                match client.head("https://orqestra.pages.dev/").send() {
                    Ok(r) if r.status().is_success() => "ok".to_string(),
                    _ => "unreachable".to_string(),
                }
            }
            Err(_) => "not_checked".to_string(),
        }
    };

    DashboardReadiness {
        local_json,
        live_url_status,
        source_commit: None,
        cloudflare_secrets: "unknown".to_string(),
    }
}

/// Mask connection errors to avoid leaking local network details.
fn mask_connection_error(err: &str) -> String {
    // Replace IP addresses and ports in error messages
    let re = regex_lite::Regex::new(r"\d+\.\d+\.\d+\.\d+(:\d+)?").unwrap_or_else(|_| regex_lite::Regex::new(r"NEVER_MATCH_").unwrap());
    let masked = re.replace_all(err, "[REDACTED:ADDRESS]");
    masked.to_string()
}

/// Generic error masker — removes any token-like patterns.
fn mask_error(err: &str) -> String {
    let patterns = ["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "sk-", "Bearer "];
    let mut result = err.to_string();
    for pat in &patterns {
        if result.contains(pat) {
            result = "[REDACTED]".to_string();
            break;
        }
    }
    result
}
