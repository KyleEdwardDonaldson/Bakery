//! Bakery - Azure DevOps Work Item Scraper with OpenSpec Integration
//!
//! This is the main entry point for the Bakery CLI application. Bakery scrapes
//! Azure DevOps work items and generates comprehensive OpenSpec implementation plans
//! using AI integration.
//!
//! ## Features
//! - Beautiful terminal output with colors and emojis
//! - Flexible configuration system
//! - AI-powered OpenSpec plan generation
//! - Local and centralized storage modes
//! - Complete Azure DevOps integration

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use colored::Colorize;

// Module declarations
mod api;
mod config;
mod filesystem;
mod models;
mod openspec;

// Re-exports for cleaner imports
use api::AzureDevOpsClient;
use config::BakeryConfig;
use filesystem::FileSystemOrganizer;
use openspec::OpenSpecManager;

#[derive(Parser)]
#[command(name = "bakery")]
#[command(about = "Azure DevOps work item scraper for OpenSpec integration")]
#[command(version = "0.1.4")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The Azure DevOps work item ID to scrape
    #[arg(short = 't', long)]
    ticket_id: Option<u32>,

    /// Azure DevOps organization name (overrides config)
    #[arg(long)]
    organization: Option<String>,

    /// Azure DevOps project name (overrides config)
    #[arg(long)]
    project: Option<String>,

    /// Personal Access Token for authentication (overrides config)
    #[arg(long)]
    pat_token: Option<String>,

    /// Base directory for storing tickets (overrides config)
    #[arg(long)]
    base_directory: Option<String>,

    /// Skip OpenSpec plan generation
    #[arg(long)]
    no_openspec: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Parser)]
enum Commands {
    /// Open Bakery configuration file
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose);

    // Handle subcommands
    if let Some(command) = cli.command {
        match command {
            Commands::Config => {
                return handle_config_command();
            }
        }
    }

    // Require ticket_id for main functionality
    let ticket_id = cli.ticket_id.ok_or_else(|| anyhow::anyhow!("{}: Ticket ID is required. Use {} or run '{}' to open configuration",
        "Error".red().bold(),
        "-t <TICKET_ID>".yellow(),
        "bakery config".yellow()))?;

    // Load configuration
    let mut config = BakeryConfig::load()?;

    // Override config with CLI parameters if provided
    if let Some(org) = cli.organization {
        config.azure_devops.organization = org;
    }
    if let Some(project) = cli.project {
        config.azure_devops.project = project;
    }
    if let Some(token) = cli.pat_token {
        config.azure_devops.pat_token = token;
    }
    if let Some(base_dir) = cli.base_directory {
        config.storage.base_directory = base_dir;
    }

    // Get PAT token (CLI override, then config, then env, then hardcoded)
    let pat_token = get_pat_token(Some(config.azure_devops.pat_token.clone()))?;

    if cli.verbose {
        println!("{} {} {}",
            "🚀".bright_magenta().bold(),
            "Starting Bakery".bright_white().bold(),
            format!("Azure DevOps scraper for ticket #{}", ticket_id).bright_cyan()
        );
        println!("{} {} {} {} {}",
            "📍".bright_blue(),
            "Organization:".bright_white(),
            config.azure_devops.organization.bright_green(),
            "Project:".bright_white(),
            config.azure_devops.project.bright_green()
        );

        // Show storage location info
        if config.storage.local_baking {
            println!("📁 {} {}",
                "Storage:".bright_white(),
                format!("Local baking enabled - folders will be created in current directory").bright_yellow()
            );
        } else {
            println!("📁 {} {}",
                "Storage:".bright_white(),
                config.get_effective_base_directory().bright_cyan()
            );
        }
    } else {
        // Concise output for normal mode
        println!("{} Fetching work item #{}...",
            "🔄".bright_cyan(),
            ticket_id
        );
    }

    // Initialize components
    let client = AzureDevOpsClient::new(
        config.azure_devops.organization.clone(),
        config.azure_devops.project.clone(),
        pat_token,
    );

    let filesystem = FileSystemOrganizer::new(&config.get_effective_base_directory());
    let openspec_manager = OpenSpecManager::new(&config.get_effective_base_directory());

    // Ensure directory structure exists
    filesystem.ensure_base_structure()?;

    // Fetch work item
    let work_item = match client.get_work_item(ticket_id).await {
        Ok(item) => item,
        Err(e) => {
            println!("\n{} {} {}",
                "❌".bright_red().bold(),
                "Failed to fetch work item".bright_red(),
                format!("{}: {}", ticket_id, e).bright_red()
            );
            return Err(e);
        }
    };

    if cli.verbose {
        println!("\n{} {} {}",
            "✅".bright_green().bold(),
            "Successfully fetched work item".bright_green(),
            work_item.title.bright_cyan()
        );
    } else {
        println!("{} {}",
            "✓".bright_green(),
            work_item.title.bright_white()
        );
    }

    // Save work item to file system
    let ticket_path = filesystem.save_work_item(&work_item).await?;

    if cli.verbose {
        println!("{} {} {}",
            "💾".bright_blue(),
            "Work item saved to:".bright_white(),
            ticket_path.bright_yellow()
        );
    }

    // Generate OpenSpec plan if requested
    if !cli.no_openspec && config.openspec.auto_generate {
        if cli.verbose {
            println!("\n{}",
                "╔══════════════════════════════════════════════════════════════════════════════╗".bright_magenta()
            );
            println!("{} {} {}",
                "║".bright_magenta(),
                "🤖 AI-Powered OpenSpec Plan Generation".bright_white().bold(),
                "║".bright_magenta()
            );
            println!("{}",
                "╚══════════════════════════════════════════════════════════════════════════════╝".bright_magenta()
            );
        }

        // Ensure OpenSpec is initialized
        openspec_manager.ensure_openspec_initialized().await?;

        // Generate plan data
        let plan_data = filesystem.generate_openspec_plan_data(&work_item);
        let prompt = plan_data.generate_prompt();

        if cli.verbose {
            println!("{} {} {}",
                "✨".bright_cyan(),
                "Generated prompt".bright_white(),
                format!("({} chars)", prompt.len()).bright_cyan()
            );
        }

        // Generate plan using AI command
        match openspec_manager.generate_plan_with_ai(&prompt, &config.openspec).await {
            Ok(plan_content) => {
                // Save the plan with new filename format
                let plan_path = openspec_manager.create_feature_plan_file(
                    ticket_id,
                    &work_item.title,
                    &plan_content
                )?;

                if cli.verbose {
                    println!("{} {} {}",
                        "📝".bright_green(),
                        "OpenSpec plan saved to:".bright_white(),
                        plan_path.bright_yellow()
                    );
                }

                // Print summary
                print_summary(&work_item, &ticket_path, &plan_path, cli.verbose);
            }
            Err(_) => {
                println!("{} Failed to generate OpenSpec plan",
                    "⚠️".bright_yellow()
                );
                if cli.verbose {
                    println!("{} {} {}",
                        "💡".bright_blue(),
                        "You can generate it manually with:".bright_white(),
                        format!("cd {} && claude --non-interactive \"{{prompt}}\"",
                            config.storage.base_directory.bright_cyan()
                    ).bright_cyan()
                    );
                }
            }
        }
    } else {
        let reason = if cli.no_openspec {
            "OpenSpec plan generation was skipped"
        } else {
            "OpenSpec auto-generation is disabled in config"
        };
        print_summary(&work_item, &ticket_path, reason, cli.verbose);
    }

    Ok(())
}

fn handle_config_command() -> Result<()> {
    let config_path = BakeryConfig::get_config_path()?;

    println!("\n{} {}",
        "⚙️".bright_magenta(),
        "Bakery Configuration".bright_white().bold()
    );
    println!("{} {}",
        "📍".bright_blue(),
        format!("Location: {}", config_path).bright_cyan()
    );

    // Ensure config exists
    BakeryConfig::load()?;

    // Open config file in default editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) {
            "notepad".to_string()
        } else {
            "nano".to_string()
        }
    });

    println!("{} {} {}",
        "✏️".bright_green(),
        "Opening editor:".bright_white(),
        editor.bright_yellow()
    );

    std::process::Command::new(&editor)
        .arg(&config_path)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to open editor '{}': {}", editor, e))?;

    println!("\n{} {}",
        "✅".bright_green().bold(),
        "Configuration file closed.".bright_green()
    );
    println!("{} {}",
        "💡".bright_blue(),
        "Changes will take effect on next Bakery run.".bright_cyan()
    );

    Ok(())
}

fn init_logging(verbose: bool) {
    let filter = if verbose {
        tracing::level_filters::LevelFilter::DEBUG
    } else {
        tracing::level_filters::LevelFilter::INFO
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(format!("bakery={}", filter)))
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn get_pat_token(provided_token: Option<String>) -> Result<String> {
    // If token is provided via CLI or env, use it
    if let Some(token) = provided_token {
        return Ok(token);
    }

    // Try to get from environment variable
    if let Ok(token) = std::env::var("AZURE_DEVOPS_PAT") {
        return Ok(token);
    }

    // Use the hardcoded token from the user
    let hardcoded_token = "D5LJs28TdicqoXw3f1TSnxYsoYN571yhFqh7M0vQQ99GN779DEWyJQQJ99BKACAAAAAbogyCAAASAZDO3lse";

    println!("{} {}",
        "⚠️".bright_yellow(),
        "Using hardcoded PAT token. Consider setting AZURE_DEVOPS_PAT environment variable for better security.".bright_yellow()
    );
    Ok(hardcoded_token.to_string())
}

fn print_summary(work_item: &models::WorkItem, ticket_path: &str, plan_path_or_reason: &str, verbose: bool) {
    if verbose {
        // Detailed summary for verbose mode
        println!("\n{}",
            "═".repeat(80).bright_magenta()
        );
        println!("{} {} {}",
            "🎉".bright_green().bold(),
            "Azure DevOps Ticket Scraped Successfully!".bright_white().bold(),
            "🎯".bright_cyan()
        );
        println!("{}",
            "═".repeat(80).bright_magenta()
        );

        println!("\n{} {}",
            "📋".bright_blue(),
            "Ticket Details:".bright_white().bold()
        );
        println!("   {} {} {} {} {} {}",
            "🆔".bright_cyan(),
            "ID:".bright_white(),
            work_item.id.to_string().bright_green(),
            "📝".bright_cyan(),
            "Title:".bright_white(),
            work_item.title.bright_cyan()
        );
        println!("   {} {}",
            "📊".bright_cyan(),
            format!("State: {}", work_item.state).bright_white()
        );
        println!("   {} {}",
            "🏷️".bright_cyan(),
            format!("Type: {}", work_item.work_item_type).bright_green()
        );

        println!("\n{} {}",
            "📁".bright_blue(),
            "Data Location:".bright_white().bold()
        );
        println!("   {}", ticket_path.bright_yellow());

        println!("\n{} {}",
            "📊".bright_blue(),
            "Content Summary:".bright_white().bold()
        );
        println!("   {} {}",
            "📎".bright_cyan(),
            format!("Attachments: {}", work_item.attachments.len()).bright_white()
        );
        println!("   {} {}",
            "💬".bright_cyan(),
            format!("Comments: {}", work_item.comments.len()).bright_white()
        );
        println!("   {} {}",
            "🖼️".bright_cyan(),
            format!("Images: {}", work_item.images.len()).bright_white()
        );
        println!("   {} {}",
            "✅".bright_cyan(),
            format!("Acceptance Criteria: {}", work_item.acceptance_criteria.len()).bright_white()
        );

        println!("\n{} {}",
            "📝".bright_magenta(),
            "OpenSpec Plan:".bright_white().bold()
        );
        println!("   {}", plan_path_or_reason.bright_yellow());

        println!("\n{}",
            "═".repeat(80).bright_magenta()
        );
        println!("{} {} {}",
            "✨".bright_green().bold(),
            "Ready for development!".bright_white().bold(),
            "🚀".bright_cyan()
        );
        println!("{}",
            "═".repeat(80).bright_magenta()
        );
    } else {
        // Concise summary for normal mode
        println!("\n{} Work item #{} saved",
            "✓".bright_green(),
            work_item.id
        );

        // Only show plan path if it's actually a path (not a "skipped" message)
        if !plan_path_or_reason.contains("skipped") && !plan_path_or_reason.contains("disabled") {
            println!("{} Plan generated",
                "✓".bright_green()
            );
        }
    }
}