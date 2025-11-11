use anyhow::{anyhow, Result};
use crate::config::OpenSpecConfig;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn, error};

pub struct OpenSpecManager {
    base_path: String,
}

impl OpenSpecManager {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
        }
    }

    pub async fn ensure_openspec_initialized(&self) -> Result<()> {
        let openspec_dir = format!("{}/openspec", self.base_path);

        if Path::new(&openspec_dir).exists() {
            info!("OpenSpec is already initialized at {}", openspec_dir);
            return Ok(());
        }

        info!("Initializing OpenSpec at {}", openspec_dir);
        self.run_openspec_init(&openspec_dir).await
    }

    async fn run_openspec_init(&self, openspec_dir: &str) -> Result<()> {
        debug!("Running 'openspec init' in {}", self.base_path);

        let output = Command::new("openspec")
            .args(&["init"])
            .current_dir(&self.base_path)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("OpenSpec initialized successfully");
                    debug!("OpenSpec init stdout: {}", String::from_utf8_lossy(&output.stdout));
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    warn!("OpenSpec init failed. stderr: {}, stdout: {}", stderr, stdout);
                    Err(anyhow!("OpenSpec init failed: {}", stderr))
                }
            }
            Err(e) => {
                warn!("Failed to run 'openspec init' command: {}", e);
                // Try to create the openspec directory manually as a fallback
                fs::create_dir_all(openspec_dir)?;
                info!("Created openspec directory manually as fallback");
                Ok(())
            }
        }
    }

    pub async fn generate_plan_with_ai(&self, prompt: &str, config: &OpenSpecConfig) -> Result<String> {
        info!("Generating OpenSpec plan using AI command with prompt length: {}", prompt.len());

        // Replace {prompt} placeholder in the command template
        let command_with_prompt = config.ai_command_template.replace("{prompt}", prompt);

        debug!("Executing AI command: {}", command_with_prompt);
        info!("Full command being executed: {}", &command_with_prompt[..command_with_prompt.len().min(500)]);
        info!("AI command template: {}", config.ai_command_template);
        info!("Prompt preview (first 200 chars): {}", &prompt[..prompt.len().min(200)]);
        info!("Full prompt length: {} chars", prompt.len());
        debug!("FULL PROMPT CONTENT:\n{}", prompt);

        // Use heredoc approach - best for long/multi-line prompts with special characters
        let output_result = {
            if cfg!(windows) {
                // Windows: Use direct claude.cmd execution with stdin
                Command::new("claude.cmd")
                    .arg("-p")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        use std::io::Write;
                        if let Some(stdin) = child.stdin.as_mut() {
                            stdin.write_all(prompt.as_bytes())?;
                            stdin.flush()?;
                        }
                        child.wait_with_output()
                    })
                    .map_err(|e| anyhow!("Failed to execute claude.cmd: {}", e))
            } else {
                // Unix: Use heredoc approach
                let temp_file_str = format!("/tmp/bakery_prompt_{}.txt", std::process::id());
                std::fs::create_dir_all("/tmp")
                    .map_err(|e| anyhow!("Failed to create temp directory: {}", e))?;

                use std::io::Write;
                let mut file = std::fs::File::create(&temp_file_str)
                    .map_err(|e| anyhow!("Failed to create temp file: {}", e))?;

                // Write heredoc wrapper script
                let heredoc_script = format!(
                    r#"claude -p <<'EOF'
{}
EOF
"#,
                    prompt
                );

                file.write_all(heredoc_script.as_bytes())
                    .map_err(|e| anyhow!("Failed to write heredoc script: {}", e))?;
                file.flush().map_err(|e| anyhow!("Failed to flush temp file: {}", e))?;
                drop(file); // Explicitly drop the file handle to release the lock

                Command::new("sh")
                    .arg(&temp_file_str)
                    .output()
                    .map_err(|e| anyhow!("Failed to execute heredoc script: {}", e))
            }
        };

        let output = output_result?;

        // Process the output
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        debug!("AI command completed");
        debug!("Exit code: {}", exit_code);
        debug!("Stdout length: {} bytes", stdout.len());
        debug!("Stderr length: {} bytes", stderr.len());
        debug!("Stdout content: {}", stdout);
        debug!("Stderr content: {}", stderr);

        if output.status.success() {
            info!("OpenSpec plan generated successfully");
            Ok(stdout.to_string())
        } else {
            error!("AI command failed with exit code {}", exit_code);
            error!("Stderr: {}", stderr);
            error!("Stdout: {}", stdout);
            Err(anyhow!("AI command failed with exit code {}: {}", exit_code, stderr))
        }
    }

    pub fn create_feature_plan_file(&self, ticket_id: u32, plan_title: &str, plan_content: &str) -> Result<String> {
        // Generate filename from ticket number and title
        let plan_filename = format!("{}-{}.md", ticket_id, self.sanitize_filename(plan_title));
        let plan_path = format!("{}/openspec/{}", self.base_path, plan_filename);

        let content = format!(
            "# OpenSpec Implementation Plan: {}\n\n**Work Item ID**: {}\n**Generated**: {}\n\n---\n\n{}",
            plan_title,
            ticket_id,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            plan_content
        );

        fs::write(&plan_path, content)?;

        info!("OpenSpec plan saved to {}", plan_path);
        Ok(plan_path)
    }

    fn sanitize_filename(&self, title: &str) -> String {
        title
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
            .collect::<String>()
            .split_whitespace()
            .take(8) // Limit to 8 words
            .map(|word| word.to_lowercase())
            .collect::<Vec<_>>()
            .join("-")
    }
}