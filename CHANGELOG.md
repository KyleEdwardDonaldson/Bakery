# Changelog

All notable changes to Bakery will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-17

### ✨ Major Features

#### Proper OpenSpec Integration
- **Complete OpenSpec CLI Integration**: Full integration with `openspec` CLI for validation and workflow management
- **Proper Change Proposal Structure**: Creates `openspec/changes/{change-id}/` with:
  - `proposal.md` - Why, What Changes, Impact sections
  - `tasks.md` - Implementation checklist with proper formatting
  - `specs/{capability}/spec.md` - Spec deltas with ADDED/MODIFIED/REMOVED Requirements
- **Automatic Validation**: Runs `openspec validate --strict` after generation
- **Change Summary**: Shows requirement counts and validation status
- **OpenSpec Commands**: Suggests next steps (`openspec list`, `openspec view`)

#### Machine-Readable Output
- **`--print` Flag**: New flag for LLM/automation integration
- **Structured Output**: Key-value format with work item details and file paths
- **Clean Output**: No decorations or progress indicators in print mode

#### Improved User Experience
- **Clean Terminal Output**: Minimal, professional interface by default
- **Verbose Mode**: All detailed logging moved behind `-v` flag
- **ASCII Box**: Clean UI for AI generation progress
- **Path Display**: Shows OpenSpec change path after generation
- **Smart Logging**: INFO logs only in verbose mode, clean output otherwise

### 🔄 Reliability Improvements

#### Azure DevOps Retry Logic
- **Automatic Retries**: All API calls retry up to 3 times on failure
- **Exponential Backoff**: 500ms base delay with exponential increase
- **Silent Failures**: Retry attempts only logged in debug mode
- **Error Handling**: Clean error messages for end users

#### Cross-Platform Fixes
- **Windows OpenSpec Support**: Uses `openspec.cmd` on Windows
- **Platform Detection**: Automatic command resolution for npm global binaries
- **Claude CLI Integration**: Fixed stdin piping for proper prompt delivery

### 🎨 Output Improvements

#### Three Output Modes
1. **Default Mode** - Clean, minimal output
   - Single-line status updates
   - Change path display
   - Next steps suggestions

2. **Verbose Mode (`-v`)** - Detailed information
   - Full progress logs
   - File paths for all operations
   - Complete summary statistics
   - Debug logging

3. **Print Mode (`-p`)** - Machine-readable
   - Structured key-value output
   - No decorations or colors
   - Perfect for LLM parsing

### 🔧 Technical Changes

#### Code Organization
- **OpenSpec Manager**: Refactored to use OpenSpec CLI commands
- **Validation Integration**: Separate method for validate + summarize
- **Prompt Improvements**: Better AI prompts with clear structure requirements
- **Error Messages**: More concise and actionable error output

#### Configuration Updates
- **AI Command Template**: Updated default to `claude --print "{prompt}"`
- **Stdin Piping**: PowerShell script uses pipe for better compatibility
- **OpenSpec Init**: Automatic `openspec update` when already initialized

### 📚 Documentation

#### Updated README
- Complete rewrite with v0.2.0 features
- Added OpenSpec directory structure documentation
- Machine-readable output examples
- Output mode comparison
- OpenSpec CLI command examples
- Updated prerequisites and setup instructions

#### New Documentation
- CHANGELOG.md - Version history and release notes
- Updated inline code documentation
- Better error messages with actionable guidance

### 🐛 Bug Fixes
- Fixed Claude CLI invocation on Windows (stdin piping)
- Fixed OpenSpec command not found on Windows (`.cmd` extension)
- Fixed verbose logging appearing in non-verbose mode
- Fixed AI prompt formatting issues

### 🚀 Performance
- Reduced console output by ~70% in default mode
- Faster execution with minimal I/O
- Async/await optimizations maintained

---

## [0.1.3] - 2025-01-17

### Added
- AI generation loading spinner with progress messages
- Decorative border box for AI section
- `indicatif` dependency for progress indicators

### Changed
- More user-friendly AI generation feedback
- Updated version display

---

## [0.1.2] - 2025-01-17

### Added
- Automatic retry logic for Azure DevOps API calls
- Exponential backoff strategy (3 retries, 500ms base delay)
- Debug-level logging for retry attempts

### Changed
- Failures only shown to users on final attempt
- Improved reliability for flaky Azure DevOps connections

---

## [0.1.1] - 2025-01-16

### Added
- Initial OpenSpec plan generation
- Azure DevOps work item scraping
- Configuration system
- Local and centralized storage modes

---

## [0.1.0] - 2025-01-15

### Added
- Initial release
- Basic Azure DevOps integration
- Work item scraping functionality
- File system organization

---

[0.2.0]: https://github.com/KyleEdwardDonaldson/bakery/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/KyleEdwardDonaldson/bakery/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/KyleEdwardDonaldson/bakery/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/KyleEdwardDonaldson/bakery/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/KyleEdwardDonaldson/bakery/releases/tag/v0.1.0
