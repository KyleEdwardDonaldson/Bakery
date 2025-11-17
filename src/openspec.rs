use anyhow::{anyhow, Result};
use crate::config::OpenSpecConfig;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn, error};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use colored::Colorize;

pub struct OpenSpecManager {
    base_path: String,
}

impl OpenSpecManager {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
        }
    }

    /// Get the correct openspec command for the current platform
    fn get_openspec_command(&self) -> String {
        if cfg!(windows) {
            // On Windows, npm global binaries are .cmd files
            "openspec.cmd".to_string()
        } else {
            "openspec".to_string()
        }
    }

    pub async fn ensure_openspec_initialized(&self) -> Result<()> {
        let openspec_dir = format!("{}/openspec", self.base_path);

        if Path::new(&openspec_dir).exists() {
            info!("OpenSpec is already initialized at {}", openspec_dir);

            // Update OpenSpec instructions to ensure they're current
            self.run_openspec_update()?;

            return Ok(());
        }

        info!("Initializing OpenSpec at {}", openspec_dir);
        self.run_openspec_init(&openspec_dir).await
    }

    fn run_openspec_update(&self) -> Result<()> {
        debug!("Running 'openspec update' to refresh instructions");

        let openspec_cmd = self.get_openspec_command();
        let output = Command::new(&openspec_cmd)
            .args(&["update"])
            .current_dir(&self.base_path)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    debug!("OpenSpec instructions updated successfully");
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("OpenSpec update warning: {}", stderr);
                    Ok(()) // Don't fail on update issues
                }
            }
            Err(e) => {
                debug!("Could not run openspec update: {}", e);
                Ok(()) // Don't fail if update doesn't work
            }
        }
    }

    async fn run_openspec_init(&self, openspec_dir: &str) -> Result<()> {
        debug!("Running 'openspec init' in {}", self.base_path);

        let openspec_cmd = self.get_openspec_command();
        let output = Command::new(&openspec_cmd)
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
        debug!("Generating OpenSpec plan using AI command with prompt length: {}", prompt.len());

        // Create a minimal spinner
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message("");

        // Replace {prompt} placeholder in the command template
        let command_with_prompt = config.ai_command_template.replace("{prompt}", prompt);

        debug!("Executing AI command: {}", command_with_prompt);
        debug!("AI command template: {}", config.ai_command_template);
        debug!("Prompt preview (first 200 chars): {}", &prompt[..prompt.len().min(200)]);
        debug!("Full prompt length: {} chars", prompt.len());
        debug!("FULL PROMPT CONTENT:\n{}", prompt);

        // Use temp file approach - best for long/multi-line prompts with special characters
        let output_result = {
            if cfg!(windows) {
                // Windows: Write prompt to temp file and use PowerShell to execute
                let temp_dir = std::env::temp_dir();
                let prompt_file = temp_dir.join(format!("bakery_prompt_{}.txt", std::process::id()));
                let script_file = temp_dir.join(format!("bakery_script_{}.ps1", std::process::id()));

                use std::io::Write;

                // Write the prompt to a temp file
                std::fs::write(&prompt_file, prompt)
                    .map_err(|e| anyhow!("Failed to write prompt file: {}", e))?;

                // Create PowerShell script that reads the prompt and passes to claude via stdin
                let ps_script = format!(
                    r#"Get-Content -Path '{}' -Raw | claude.cmd --print
"#,
                    prompt_file.display().to_string().replace("\\", "\\\\")
                );

                std::fs::write(&script_file, ps_script)
                    .map_err(|e| anyhow!("Failed to write PowerShell script: {}", e))?;

                debug!("Executing PowerShell script: {}", script_file.display());

                let output = Command::new("powershell.exe")
                    .args(&[
                        "-NoProfile",
                        "-NonInteractive",
                        "-ExecutionPolicy", "Bypass",
                        "-File", script_file.to_str().unwrap()
                    ])
                    .output()
                    .map_err(|e| anyhow!("Failed to execute PowerShell script: {}", e))?;

                // Clean up temp files
                let _ = std::fs::remove_file(&prompt_file);
                let _ = std::fs::remove_file(&script_file);

                Ok(output)
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

        spinner.finish_and_clear();

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
            debug!("OpenSpec plan generated successfully");
            Ok(stdout.to_string())
        } else {
            error!("AI command failed with exit code {}", exit_code);
            error!("Stderr: {}", stderr);
            error!("Stdout: {}", stdout);
            Err(anyhow!("AI command failed with exit code {}: {}", exit_code, stderr))
        }
    }

    pub fn create_feature_plan_file(&self, ticket_id: u32, plan_title: &str, plan_content: &str) -> Result<String> {
        // Generate change ID from ticket number and title (kebab-case, verb-led)
        let change_id = format!("add-{}-{}", ticket_id, self.sanitize_filename(plan_title));
        let change_dir = format!("{}/openspec/changes/{}", self.base_path, change_id);

        // Create the change directory structure
        fs::create_dir_all(&change_dir)?;

        // Create proposal.md
        let proposal_path = format!("{}/proposal.md", change_dir);
        let proposal_content = self.extract_proposal_section(&plan_content, ticket_id, plan_title);
        fs::write(&proposal_path, proposal_content)?;

        // Create tasks.md
        let tasks_path = format!("{}/tasks.md", change_dir);
        let tasks_content = self.extract_tasks_section(&plan_content);
        fs::write(&tasks_path, tasks_content)?;

        // Create spec deltas if present in plan_content
        self.create_spec_deltas(&change_dir, &plan_content)?;

        info!("OpenSpec change proposal created at {}", change_dir);

        Ok(change_dir)
    }

    pub fn validate_and_summarize(&self, change_id: &str, print_mode: bool) -> Result<()> {
        // Validate the created change proposal
        self.validate_change(change_id, print_mode)?;

        // Show change summary if validation passed
        if !print_mode {
            self.show_change_summary(change_id);
        }

        Ok(())
    }

    fn validate_change(&self, change_id: &str, print_mode: bool) -> Result<()> {
        debug!("Validating OpenSpec change: {}", change_id);

        let openspec_cmd = self.get_openspec_command();
        let output = Command::new(&openspec_cmd)
            .args(&["validate", change_id, "--strict"])
            .current_dir(&self.base_path)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    debug!("OpenSpec validation passed for {}", change_id);
                    if !print_mode {
                        println!("{} Validation passed",
                            "✓".bright_green()
                        );
                    }
                    Ok(())
                } else {
                    debug!("OpenSpec validation failed for {}", change_id);
                    debug!("Validation stdout: {}", stdout);
                    debug!("Validation stderr: {}", stderr);

                    // Show concise validation failure
                    if !print_mode {
                        println!("{} Validation issues found - run {} for details",
                            "⚠️".bright_yellow(),
                            format!("openspec validate {} --strict", change_id).bright_cyan()
                        );
                    }

                    Ok(())
                }
            }
            Err(e) => {
                debug!("Failed to run openspec validate: {}", e);
                if !print_mode {
                    println!("{} OpenSpec CLI not found - validation skipped",
                        "⚠️".bright_yellow()
                    );
                }
                Ok(())
            }
        }
    }

    fn show_change_summary(&self, change_id: &str) {
        debug!("Showing summary for change: {}", change_id);

        // Try to get JSON output for structured display
        let openspec_cmd = self.get_openspec_command();
        let output = Command::new(&openspec_cmd)
            .args(&["show", change_id, "--json", "--deltas-only", "--no-interactive"])
            .current_dir(&self.base_path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug!("Change summary JSON: {}", stdout);

                // Parse JSON to show a clean summary
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(deltas) = json.get("deltas").and_then(|d| d.as_array()) {
                        let mut has_deltas = false;
                        for delta in deltas {
                            if let Some(added) = delta.get("added").and_then(|a| a.as_array()) {
                                if !added.is_empty() {
                                    println!("{} {} new requirement(s)",
                                        "✓".bright_green(),
                                        added.len()
                                    );
                                    has_deltas = true;
                                }
                            }
                            if let Some(modified) = delta.get("modified").and_then(|m| m.as_array()) {
                                if !modified.is_empty() {
                                    println!("{} {} modified requirement(s)",
                                        "✓".bright_green(),
                                        modified.len()
                                    );
                                    has_deltas = true;
                                }
                            }
                        }
                        if !has_deltas {
                            debug!("No deltas found in change");
                        }
                    }
                }
            }
            _ => {
                debug!("Could not retrieve change summary");
            }
        }
    }

    fn extract_proposal_section(&self, plan_content: &str, ticket_id: u32, plan_title: &str) -> String {
        // Parse the AI-generated content to extract proposal information
        // This is a simple extraction - the AI should generate content with clear sections

        let mut why_section = String::new();
        let mut what_changes = String::new();
        let mut impact = String::new();

        // Try to find sections in the AI response
        if let Some(why_start) = plan_content.find("## Why") {
            if let Some(why_end) = plan_content[why_start..].find("\n## ") {
                why_section = plan_content[why_start + 6..why_start + why_end].trim().to_string();
            }
        }

        if let Some(what_start) = plan_content.find("## What") {
            if let Some(what_end) = plan_content[what_start..].find("\n## ") {
                what_changes = plan_content[what_start + 7..what_start + what_end].trim().to_string();
            }
        }

        if let Some(impact_start) = plan_content.find("## Impact") {
            if let Some(impact_end) = plan_content[impact_start..].find("\n## ") {
                impact = plan_content[impact_start + 9..impact_start + impact_end].trim().to_string();
            } else {
                impact = plan_content[impact_start + 9..].trim().to_string();
            }
        }

        // Fallback: use the entire content if sections not found
        if why_section.is_empty() && what_changes.is_empty() {
            why_section = format!("Implement Azure DevOps work item #{}: {}", ticket_id, plan_title);
            what_changes = "See AI-generated plan below for detailed changes.".to_string();
        }

        format!(
            "# Change: {}\n\n## Why\n{}\n\n## What Changes\n{}\n\n## Impact\n{}\n\n---\n\n**Work Item ID**: {}\n**Generated**: {}\n\n## AI-Generated Plan\n\n{}",
            plan_title,
            why_section,
            what_changes,
            impact,
            ticket_id,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            plan_content
        )
    }

    fn extract_tasks_section(&self, plan_content: &str) -> String {
        // Try to extract tasks from the AI-generated content
        if let Some(tasks_start) = plan_content.find("## Tasks") {
            if let Some(tasks_end) = plan_content[tasks_start..].find("\n## ") {
                return plan_content[tasks_start..tasks_start + tasks_end].to_string();
            } else {
                return plan_content[tasks_start..].to_string();
            }
        }

        // If no tasks section found, look for Implementation or checklist items
        if let Some(impl_start) = plan_content.find("## Implementation") {
            if let Some(impl_end) = plan_content[impl_start..].find("\n## ") {
                return plan_content[impl_start..impl_start + impl_end].to_string();
            }
        }

        // Fallback: create a basic tasks structure
        format!(
            "## 1. Implementation\n- [ ] 1.1 Review work item requirements\n- [ ] 1.2 Implement core functionality\n- [ ] 1.3 Write tests\n- [ ] 1.4 Update documentation\n\n## AI-Generated Content\n\n{}",
            plan_content
        )
    }

    fn create_spec_deltas(&self, change_dir: &str, plan_content: &str) -> Result<()> {
        // Look for spec sections in the AI-generated content
        // This is optional - only create if the AI generated proper spec deltas

        if plan_content.contains("## ADDED Requirements") ||
           plan_content.contains("## MODIFIED Requirements") ||
           plan_content.contains("## REMOVED Requirements") {

            // Create specs directory
            let specs_dir = format!("{}/specs", change_dir);
            fs::create_dir_all(&specs_dir)?;

            // For now, create a generic capability spec
            // In the future, we could parse multiple capabilities from the AI response
            let spec_path = format!("{}/feature/spec.md", specs_dir);
            fs::create_dir_all(format!("{}/feature", specs_dir))?;

            // Extract only the spec delta sections
            let mut spec_content = String::new();
            for section in ["## ADDED Requirements", "## MODIFIED Requirements", "## REMOVED Requirements"] {
                if let Some(section_start) = plan_content.find(section) {
                    if let Some(section_end) = plan_content[section_start..].find("\n## ") {
                        spec_content.push_str(&plan_content[section_start..section_start + section_end]);
                        spec_content.push_str("\n\n");
                    } else {
                        spec_content.push_str(&plan_content[section_start..]);
                    }
                }
            }

            if !spec_content.is_empty() {
                fs::write(&spec_path, spec_content)?;
                info!("Created spec delta at {}", spec_path);
            }
        }

        Ok(())
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