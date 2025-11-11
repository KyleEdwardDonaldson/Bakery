# Bakery 🥐

> A beautiful Azure DevOps work item scraper with AI-powered OpenSpec integration.

Bakery is a professional CLI tool that scrapes Azure DevOps work items and generates comprehensive OpenSpec implementation plans using AI. Features beautiful terminal output, flexible configuration, and both centralized and local storage modes.

## ✨ Features

- 🚀 **Beautiful Terminal Output** - Colorful, emoji-enhanced user experience
- 🔧 **Flexible Configuration** - User-friendly TOML configuration system
- 📁 **Smart Storage Options** - Centralized or local project-based storage
- 🤖 **AI-Powered OpenSpec Integration** - Comprehensive implementation plan generation
- 🎯 **Complete Azure DevOps Integration** - Full work item content extraction
- ⚙️ **Professional CLI** - Built with Rust for performance and reliability
- 📋 **OpenSpec Methodology** - Proper three-stage workflow integration

## 🚀 Quick Start

### Installation

```bash
# Install from crates.io
cargo install bakery-devops

# The binary will be available as `bakery` (not `bakery-devops`)
bakery --help

# Or build locally
git clone https://github.com/KyleEdwardDonaldson/bakery
cd bakery
cargo build --release
```

### Basic Usage

```bash
# Scrape a work item and generate OpenSpec plan
bakery -t 12345

# Open configuration file
bakery config

# Get help
bakery --help
```

### 📦 Package vs Binary Name

**Important Note:**
- **Package name on crates.io:** `bakery-devops`
- **Binary name after installation:** `bakery`

This means you install with `cargo install bakery-devops` but run commands with `bakery`.

```bash
# ✅ Correct: Install the package
cargo install bakery-devops

# ✅ Correct: Use the binary after installation
bakery --help

# ❌ Incorrect: (package name is not the binary name)
bakery-devops --help
```

## ⚠️ Prerequisites

### Required for Basic Usage:
- ✅ Azure DevOps Personal Access Token (PAT)
- ✅ Network access to `dev.azure.com`

### Optional for OpenSpec Integration:
- 🤖 **AI CLI Tool** (any of the following):
  - [Claude CLI](https://claude.ai/cli) - Recommended
  - OpenAI CLI
  - Custom AI tools that accept prompts via command line

**If you don't have an AI CLI configured**, Bakery will still scrape work items perfectly:

1. **Disable OpenSpec generation:**
   ```bash
   bakery -t 12345 --no-openspec
   ```

2. **Disable in configuration:**
   ```toml
   [openspec]
   auto_generate = false
   ```

## 📋 Configuration

Bakery automatically creates a configuration file at:
- **Windows**: `%USERPROFILE%\.bakery\bakery-config.toml`
- **Mac/Linux**: `~/.bakery/bakery-config.toml`

Run `bakery config` to open the configuration file in your default editor.

### Configuration Example

```toml
[azure_devops]
# Azure DevOps organization name
organization = "your-organization"

# Azure DevOps project name
project = "YourProject"

# Personal Access Token (PAT) for Azure DevOps API access
pat_token = "your-pat-token-here"

# Azure DevOps REST API version (usually don't need to change this)
api_version = "7.1"

[storage]
# Base directory where Bakery stores all data
# Windows example: "C:/DevOpsData"
# Mac/Linux example: "~/devops-data"
base_directory = "~/devops-data"

# Subdirectory for scraped tickets
tickets_subdir = "Tickets"

# Subdirectory for OpenSpec plans
openspec_subdir = "openspec"

# Local baking mode - creates folders in current working directory
local_baking = false

[openspec]
# AI command template for generating OpenSpec plans
# Use {prompt} as a placeholder for the generated prompt
ai_command_template = "claude -p \"{prompt}\""

# Automatically generate OpenSpec plans after scraping tickets
auto_generate = true
```

## 🔧 Azure DevOps Setup

1. **Create a Personal Access Token (PAT):**
   - Go to https://dev.azure.com/{organization}/_usersSettings/tokens
   - Click "Create New Token"
   - Give it a name (e.g., "Bakery Scraper")
   - Select scopes: "Work Items" → "Read" (vso.work)
   - Copy the token to your configuration

2. **Configure Bakery:**
   - Run `bakery config` to open configuration
   - Update `organization`, `project`, and `pat_token`
   - Set your preferred `base_directory`

## 📁 Storage Modes

### Centralized Storage (Default)
```
{base_directory}/
├── Tickets/
│   ├── 12345/
│   │   ├── work_item.json
│   │   ├── attachments/
│   │   └── images/
│   └── 12346/
└── openspec/
    ├── 12345-concise-title.md
    └── 12346-another-title.md
```

### Local Baking Mode
Set `local_baking = true` to create folders in your current working directory:
```
current-project/
├── Tickets/
│   └── 12345/
└── openspec/
```

## 🤖 AI Integration

Bakery supports any AI CLI that accepts prompts as command line arguments:

### Claude (Recommended)
```bash
claude -p "{prompt}"
```

### OpenAI CLI
```bash
openai api chat.complete --messages "{prompt}"
```

### Custom AI Tools
```bash
your-ai-tool --prompt "{prompt}"
```

Configure your preferred AI tool in the `ai_command_template` setting.

## 📋 OpenSpec Integration

Bakery generates comprehensive OpenSpec implementation plans that follow the proper three-stage workflow:

### Generated Plans Include:
- **Change Analysis**: Scope identification and directory structure planning
- **Proposal Structure**: Complete proposal.md with Why, What Changes, and Impact
- **Delta Specifications**: Proper ADDED/MODIFIED/REMOVED Requirements with scenarios
- **Implementation Tasks**: Detailed tasks.md with analysis, implementation, and verification
- **Design Documentation**: design.md when needed for complex changes
- **Three-Stage Workflow**: Creating Changes → Implementing Changes → Archiving Changes
- **Quality Gates**: Validation, testing, and approval requirements

### OpenSpec Features:
- ✅ Proper methodology integration from OpenSpec AGENTS.md
- ✅ Scenario-driven requirements with WHEN/THEN format
- ✅ Delta operations (ADDED/MODIFIED/REMOVED/RENAMED)
- ✅ Implementation task breakdowns
- ✅ Testing strategies and validation steps
- ✅ Quality gates and completion criteria

## 🎯 Features

### Work Item Scraping
- ✅ Full work item details (title, description, state, type)
- ✅ Acceptance criteria extraction
- ✅ Comments and attachments
- ✅ Embedded images and media
- ✅ Related work items
- ✅ HTML content cleaning and formatting

### OpenSpec Integration
- ✅ Comprehensive plan generation (130+ lines typical)
- ✅ Proper OpenSpec methodology
- ✅ AI-powered analysis and task creation
- ✅ Integration with multiple AI platforms
- ✅ Customizable filename formats

### Terminal Experience
- 🎨 Beautiful colored output
- 📊 Detailed progress information
- ✅ Success/error messaging
- 📁 Storage location indicators

## 📖 Command Line Options

```bash
bakery [OPTIONS] [COMMAND]

Commands:
  config  Open Bakery configuration file
  help    Print this message or the help of the given subcommands

Options:
  -t, --ticket-id <TICKET_ID>            The Azure DevOps work item ID to scrape
      --organization <ORGANIZATION>      Azure DevOps organization name (overrides config)
      --project <PROJECT>                Azure DevOps project name (overrides config)
      --pat-token <PAT_TOKEN>            Personal Access Token for authentication (overrides config)
      --base-directory <BASE_DIRECTORY>  Base directory for storing tickets (overrides config)
      --no-openspec                      Skip OpenSpec plan generation
  -v, --verbose                          Enable verbose logging
  -h, --help                             Print help
  -V, --version                          Print version
```

## 🔍 Examples

### Basic Scraping
```bash
# Scrape work item with default settings
bakery -t 12345

# Scrape with verbose logging
bakery -t 12345 --verbose

# Skip OpenSpec generation
bakery -t 12345 --no-openspec
```

### Override Configuration
```bash
# Use different organization
bakery -t 12345 --organization my-org

# Use different project
bakery -t 12345 --project MyProject

# Use custom storage directory
bakery -t 12345 --base-directory ./my-tickets
```

### Local Baking Mode
```bash
# Enable local baking in config, then run:
cd /path/to/your/project
bakery -t 12345
# Creates folders in current directory
```

## 🔧 Development

### Building from Source
```bash
git clone https://github.com/KyleEdwardDonaldson/bakery
cd bakery
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Development Setup
```bash
# Install development dependencies
cargo build

# Run with debug output
cargo run -- -t 12345 --verbose
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Guidelines
- Follow Rust best practices and conventions
- Add tests for new features
- Update documentation as needed
- Ensure the code builds with `cargo build`
- Run tests with `cargo test`

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👨‍💻 Author

**Kyle Edward Donaldson**
- Email: kyle@ked.dev
- GitHub: @KyleEdwardDonaldson

## 🙏 Acknowledgments

### OpenSpec Integration
This tool integrates methodology and concepts from **OpenSpec**, a comprehensive specification-driven development framework. The OpenSpec workflow provides the structured approach to change management, requirement specification, and implementation planning that makes Bakery-generated plans so effective.

### Built With
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [clap](https://clap.rs/) - Command line argument parsing
- [colored](https://crates.io/crates/colored) - Terminal colors
- [tokio](https://tokio.rs/) - Async runtime
- [reqwest](https://docs.rs/reqwest/) - HTTP client
- [scraper](https://crates.io/crates/scraper) - HTML parsing
- [serde](https://serde.rs/) - Serialization/deserialization
- [anyhow](https://docs.rs/anyhow/) - Error handling

### Integrations
- [Azure DevOps](https://dev.azure.com/) - Work item management
- [OpenSpec](https://openspec.dev/) - Specification-driven development
- [Claude](https://claude.ai/) - AI-powered plan generation

## 📚 Additional Resources

- [Azure DevOps REST API Documentation](https://docs.microsoft.com/en-us/rest/api/azure/devops/)
- [OpenSpec Methodology](https://openspec.dev/)
- [Claude CLI Documentation](https://claude.ai/cli)
- [Rust Documentation](https://doc.rust-lang.org/)

---

**Bakery** - Transform your Azure DevOps work items into comprehensive implementation plans with the power of AI and OpenSpec methodology. 🚀
