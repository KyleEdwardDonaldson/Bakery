# Bakery 🥐

> A beautiful Azure DevOps work item scraper with AI-powered OpenSpec integration.

Bakery is a professional CLI tool that scrapes Azure DevOps work items and generates comprehensive OpenSpec implementation plans using AI. Features beautiful terminal output, flexible configuration, and seamless OpenSpec CLI integration with automatic validation.

## ✨ Features

- 🚀 **Clean Terminal Output** - Minimal, professional interface with verbose mode available
- 🔧 **Flexible Configuration** - User-friendly TOML configuration system
- 📁 **Smart Storage Options** - Centralized or local project-based storage
- 🤖 **AI-Powered OpenSpec Integration** - Comprehensive implementation plan generation
- ✅ **Automatic Validation** - Built-in OpenSpec CLI validation and formatting
- 📋 **Proper OpenSpec Structure** - Creates changes/proposal.md, tasks.md, and specs/
- 🎯 **Complete Azure DevOps Integration** - Full work item content extraction
- 🔄 **Retry Logic** - Automatic retries for flaky Azure DevOps API calls
- 🖨️ **Machine-Readable Output** - `--print` flag for LLM/automation integration
- ⚙️ **Professional CLI** - Built with Rust for performance and reliability

## 🚀 Quick Start

### Installation

```bash
# Install from crates.io
cargo install bakery-devops

# The binary will be available as `bakery`
bakery --help

# Or build locally
git clone https://github.com/KyleEdwardDonaldson/bakery
cd bakery
cargo build --release
```

### Basic Usage

```bash
# Scrape a work item and generate OpenSpec change proposal
bakery -t 12345

# Machine-readable output for LLM integration
bakery -t 12345 --print

# Verbose mode with detailed logging
bakery -t 12345 --verbose

# Open configuration file
bakery config
```

### 📦 Package vs Binary Name

**Important Note:**
- **Package name on crates.io:** `bakery-devops`
- **Binary name after installation:** `bakery`

```bash
# ✅ Correct: Install the package
cargo install bakery-devops

# ✅ Correct: Use the binary
bakery --help

# ❌ Incorrect: package name ≠ binary name
bakery-devops --help
```

## ⚠️ Prerequisites

### Required:
- ✅ Azure DevOps Personal Access Token (PAT)
- ✅ Network access to `dev.azure.com`

### Optional for Full OpenSpec Integration:
- 🤖 **Claude CLI** or compatible AI tool - For plan generation
- 📋 **OpenSpec CLI** (`npm install -g openspec`) - For validation and workflow

**If you don't have OpenSpec CLI**, Bakery will still:
- ✅ Scrape work items perfectly
- ✅ Generate AI plans (if AI configured)
- ⚠️ Skip validation (you'll see a warning)

**If you don't have AI CLI configured**:
```bash
# Disable OpenSpec generation
bakery -t 12345 --no-openspec

# Or disable in config
[openspec]
auto_generate = false
```

## 📋 Configuration

Bakery automatically creates a configuration file at:
- **Windows**: `%USERPROFILE%\.bakery\bakery-config.toml`
- **Mac/Linux**: `~/.bakery/bakery-config.toml`

Run `bakery config` to open and edit the configuration.

### Configuration Example

```toml
[azure_devops]
organization = "your-organization"
project = "YourProject"
pat_token = "your-pat-token-here"
api_version = "7.1"

[storage]
base_directory = "~/devops-data"
tickets_subdir = "Tickets"
openspec_subdir = "openspec"
local_baking = false  # Set true to use current directory

[openspec]
ai_command_template = "claude --print \"{prompt}\""
auto_generate = true
```

## 🔧 Azure DevOps Setup

1. **Create a Personal Access Token (PAT):**
   - Go to https://dev.azure.com/{organization}/_usersSettings/tokens
   - Click "New Token"
   - Name: "Bakery Scraper"
   - Scope: Work Items → Read (vso.work)
   - Copy token to configuration

2. **Configure Bakery:**
   ```bash
   bakery config
   # Update organization, project, pat_token
   ```

## 📁 OpenSpec Directory Structure

Bakery creates proper OpenSpec change proposals:

```
{base_directory}/
├── Tickets/
│   └── 12345/
│       ├── work_item.json
│       ├── attachments/
│       └── images/
└── openspec/
    ├── AGENTS.md         # Created by openspec init
    ├── project.md        # Project context
    ├── specs/            # Current specifications
    └── changes/          # Change proposals
        └── add-12345-feature-name/
            ├── proposal.md    # Why, What, Impact
            ├── tasks.md       # Implementation checklist
            └── specs/         # Spec deltas
                └── feature/
                    └── spec.md  # ADDED/MODIFIED/REMOVED Requirements
```

### Local Baking Mode
Set `local_baking = true` to create folders in current working directory.

## 🤖 AI Integration

Bakery uses stdin piping for maximum compatibility with AI CLIs:

### Claude CLI (Recommended)
```bash
# In config:
ai_command_template = "claude --print \"{prompt}\""
```

### Custom AI Tools
Any tool that accepts stdin or command-line prompts:
```bash
ai_command_template = "your-ai-tool --input \"{prompt}\""
```

## 📋 OpenSpec Integration

Bakery generates **proper OpenSpec change proposals** following the official methodology:

### What Gets Created:

#### 1. **proposal.md**
```markdown
# Change: Feature Name

## Why
[Problem/opportunity explanation]

## What Changes
- [Bullet list of changes]
- [Breaking changes marked]

## Impact
- Affected specs: [capabilities]
- Affected code: [files/systems]
```

#### 2. **tasks.md**
```markdown
## 1. Analysis and Planning
- [ ] 1.1 Review existing specs
- [ ] 1.2 Check for conflicts

## 2. Implementation
- [ ] 2.1 Implement feature
- [ ] 2.2 Write tests
- [ ] 2.3 Update docs

## 3. Verification
- [ ] 3.1 Run openspec validate --strict
- [ ] 3.2 Test scenarios
- [ ] 3.3 Get approval
```

#### 3. **specs/{capability}/spec.md** (if requirements included)
```markdown
## ADDED Requirements
### Requirement: Feature Name
The system SHALL provide...

#### Scenario: Success Case
- **WHEN** user performs action
- **THEN** expected result

## MODIFIED Requirements
[Full updated requirements]

## REMOVED Requirements
[Deprecated features]
```

### Automatic Validation

Bakery automatically runs `openspec validate --strict` on generated changes and reports results:
- ✅ **Passed**: Change is properly formatted
- ⚠️ **Issues**: Shows validation command to fix errors
- ⚙️ **CLI not found**: Continues without validation

### OpenSpec Commands

After generating a change:
```bash
# List all changes
openspec list

# View interactive dashboard
openspec view

# Show change details
openspec show add-12345-feature-name

# Validate change
openspec validate add-12345-feature-name --strict

# After implementation, archive the change
openspec archive add-12345-feature-name
```

## 📖 Command Line Options

```bash
bakery [OPTIONS] [COMMAND]

Commands:
  config  Open Bakery configuration file

Options:
  -t, --ticket-id <TICKET_ID>            Azure DevOps work item ID to scrape
      --organization <ORGANIZATION>      Override config organization
      --project <PROJECT>                Override config project
      --pat-token <PAT_TOKEN>            Override config PAT token
      --base-directory <BASE_DIRECTORY>  Override config base directory
      --no-openspec                      Skip OpenSpec plan generation
  -v, --verbose                          Enable verbose logging
  -p, --print                            Machine-readable output for LLMs
  -h, --help                             Print help
  -V, --version                          Print version
```

## 🔍 Examples

### Basic Usage
```bash
# Clean output (default)
bakery -t 12345

# Verbose output with all details
bakery -t 12345 --verbose

# Skip OpenSpec generation
bakery -t 12345 --no-openspec
```

### Machine-Readable Output
```bash
# Perfect for LLM/automation integration
bakery -t 12345 --print

# Output:
# --- BAKERY OUTPUT ---
# work_item_id: 12345
# work_item_title: Feature Name
# ticket_path: /path/to/Tickets/12345
# change_path: /path/to/openspec/changes/add-12345-feature-name
# status: success
```

### Override Configuration
```bash
# Different organization
bakery -t 12345 --organization my-org

# Custom storage location
bakery -t 12345 --base-directory ./my-tickets
```

## 🎯 Output Modes

### Default Mode (Clean & Concise)
```
🔄 Fetching work item #12345...
✓ Feature implementation

┌─────────────────────────────────────────────────────┐
│  🤖 AI Generating OpenSpec Plan...             │
└─────────────────────────────────────────────────────┘
✓ Validation passed
✓ 3 new requirement(s)
📁 /path/to/openspec/changes/add-12345-feature-name

✓ Complete

Next: openspec list  or openspec view
```

### Verbose Mode (`-v`)
- Detailed progress messages
- File paths for all operations
- Full summary with statistics
- Debug logging information

### Print Mode (`-p`)
- Machine-readable key-value output
- No decorations or progress indicators
- Perfect for parsing by LLMs or scripts

## 🔄 Reliability Features

### Automatic Retry Logic
- Azure DevOps API calls retry up to 3 times
- Exponential backoff (500ms base delay)
- Failures only shown in verbose/debug mode
- Handles flaky network connections gracefully

### Error Handling
- Clear error messages for common issues
- Graceful degradation (works without AI/OpenSpec CLI)
- Detailed logging in verbose mode

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

### Development Run
```bash
cargo run -- -t 12345 --verbose
```

## 🤝 Contributing

Contributions welcome! Please:
1. Open an issue for major changes
2. Follow Rust best practices
3. Add tests for new features
4. Update documentation

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 👨‍💻 Author

**Kyle Edward Donaldson**
- Email: kyle@ked.dev
- GitHub: @KyleEdwardDonaldson

## 🙏 Acknowledgments

### OpenSpec
This tool fully integrates with **OpenSpec**, the specification-driven development framework. Bakery generates proper OpenSpec change proposals that follow the official three-stage workflow.

### Built With
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [clap](https://clap.rs/) - Command line parsing
- [colored](https://crates.io/crates/colored) - Terminal colors
- [tokio](https://tokio.rs/) - Async runtime
- [reqwest](https://docs.rs/reqwest/) - HTTP client
- [scraper](https://crates.io/crates/scraper) - HTML parsing
- [indicatif](https://crates.io/crates/indicatif) - Progress indicators
- [serde](https://serde.rs/) - Serialization

### Integrations
- [Azure DevOps](https://dev.azure.com/) - Work item management
- [OpenSpec](https://github.com/Fission-AI/OpenSpec) - Spec-driven development
- [Claude CLI](https://claude.ai/) - AI-powered plan generation

## 📚 Resources

- [Azure DevOps REST API](https://docs.microsoft.com/en-us/rest/api/azure/devops/)
- [OpenSpec GitHub](https://github.com/Fission-AI/OpenSpec)
- [OpenSpec Methodology](https://openspec.dev/)
- [Rust Documentation](https://doc.rust-lang.org/)

---

**Bakery v0.2.0** - Transform Azure DevOps work items into comprehensive OpenSpec change proposals with AI-powered analysis and automatic validation. 🚀
