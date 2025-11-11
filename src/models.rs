use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use scraper::{Html, Selector};

/// Clean HTML content by removing tags and extracting readable text
pub fn clean_html_content(html_content: &str) -> String {
    if html_content.is_empty() {
        return String::new();
    }

    let fragment = Html::parse_fragment(html_content);
    let text_selectors = Selector::parse("p, div, li, span, h1, h2, h3, h4, h5, h6").unwrap();

    let mut cleaned_text = String::new();

    // Extract text from relevant elements
    for element in fragment.select(&text_selectors) {
        let text = element.text().collect::<String>().trim().to_string();
        if !text.is_empty() {
            // Add appropriate formatting based on element type
            if element.value().name() == "li" {
                cleaned_text.push_str(&format!("• {}\n", text));
            } else if element.value().name().starts_with('h') {
                cleaned_text.push_str(&format!("\n**{}**\n", text));
            } else {
                cleaned_text.push_str(&format!("{}\n", text));
            }
        }
    }

    // Clean up extra whitespace and format
    cleaned_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\n\n\n", "\n\n")
        .trim()
        .to_string()
}

/// Clean a vector of HTML/Markdown content strings
pub fn clean_text_content_list(content_list: &[String]) -> Vec<String> {
    content_list
        .iter()
        .map(|content| clean_html_content(content))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub comments: Vec<Comment>,
    pub attachments: Vec<Attachment>,
    pub images: Vec<ImageReference>,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub created_by: User,
    pub assigned_to: Option<User>,
    pub state: String,
    pub work_item_type: String,
    pub area_path: String,
    pub iteration_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: u32,
    pub author: User,
    pub created_date: DateTime<Utc>,
    pub updated_date: Option<DateTime<Utc>>,
    pub text: String,
    pub images: Vec<ImageReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: u32,
    pub filename: String,
    pub url: String,
    pub local_path: String,
    pub content_type: String,
    pub size: u64,
    pub created_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageReference {
    pub placeholder: String,
    pub original_url: String,
    pub local_path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub alt_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub display_name: String,
    pub email: String,
    pub url: String,
}

// Azure DevOps API Response Models
#[derive(Debug, Clone, Deserialize)]
pub struct AzureWorkItemResponse {
    pub id: u32,
    #[serde(rename = "rev")]
    pub revision: u32,
    pub fields: HashMap<String, serde_json::Value>,
    #[serde(rename = "relations")]
    pub relations: Option<Vec<AzureRelation>>,
    pub url: String,
    #[serde(rename = "_links")]
    pub links: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AzureRelation {
    pub rel: String,
    pub url: String,
    pub attributes: Option<AzureRelationAttributes>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AzureRelationAttributes {
    pub name: Option<String>,
    pub comment: Option<String>,
    #[serde(rename = "authorized-date")]
    pub authorized_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AzureCommentsResponse {
    pub count: u32,
    pub value: Vec<AzureComment>,
}

#[derive(Debug, Deserialize)]
pub struct AzureComment {
    pub id: u32,
    pub version: u32,
    pub text: String,
    #[serde(rename = "createdDate")]
    pub created_date: String,
    #[serde(rename = "updatedDate")]
    pub updated_date: Option<String>,
    pub author: AzureUser,
}

#[derive(Debug, Deserialize)]
pub struct AzureUser {
    pub displayName: String,
    pub url: String,
    #[serde(rename = "_links")]
    pub links: serde_json::Value,
}

// OpenSpec Plan Generation Models
#[derive(Debug, Serialize)]
pub struct OpenSpecPlanData {
    pub ticket_number: u32,
    pub ticket_title: String,
    pub ticket_description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: String,
    pub complexity: String,
    pub dependencies: Vec<String>,
    pub estimated_effort: Option<String>,
    pub attachments_count: usize,
    pub comments_count: usize,
    pub has_images: bool,
}

impl OpenSpecPlanData {
    pub fn generate_prompt(&self) -> String {
        // Debug logging to see what we're working with
        tracing::debug!("generate_prompt: ticket_description length: {}", self.ticket_description.len());
        tracing::debug!("generate_prompt: ticket_description preview: {}", &self.ticket_description[..self.ticket_description.len().min(100)]);

        // The description should already be cleaned from generate_openspec_plan_data()
        let cleaned_acceptance_criteria = clean_text_content_list(&self.acceptance_criteria);

        tracing::debug!("generate_prompt: Using pre-cleaned description of length: {}", self.ticket_description.len());

        format!(
            "You are creating a comprehensive OpenSpec implementation plan for the following Azure DevOps work item.
Follow the complete OpenSpec methodology with proper three-stage workflow, directory structures, and spec formatting.

**Ticket #{}: {}**

**Description:**
{}

**Acceptance Criteria:**
{}

## OpenSpec Implementation Plan Requirements

Create a complete OpenSpec change proposal with these components:

### 1. Change Analysis and Setup
- **Change ID**: Propose a unique kebab-case, verb-led identifier (e.g., \"add-\", \"update-\", \"remove-\", \"refactor-\")
- **Scope Decision**: Is this a new capability or modifying existing capability?
- **Directory Structure**: Plan the openspec/changes/[change-id]/ layout

### 2. Proposal Structure (proposal.md)
Create a comprehensive proposal with:
```markdown
# Change: [Brief description]

## Why
[1-2 sentences on problem/opportunity]

## What Changes
- [Bullet list of changes]
- [Mark breaking changes with **BREAKING**]

## Impact
- Affected specs: [list capabilities]
- Affected code: [key files/systems]
```

### 3. Delta Specifications (specs/[capability]/spec.md)
Create proper delta changes using OpenSpec format:

## ADDED Requirements
### Requirement: [New Feature Name]
The system SHALL provide [detailed requirement description]

#### Scenario: [Success Case Name]
- **WHEN** [user performs action]
- **THEN** [expected result]

#### Scenario: [Edge Case Name]
- **WHEN** [condition occurs]
- **THEN** [expected behavior]

## MODIFIED Requirements
### Requirement: [Existing Feature Name]
[Complete modified requirement with full scenarios]

## REMOVED Requirements (if applicable)
### Requirement: [Old Feature Name]
**Reason**: [Why removing]
**Migration**: [How to handle]

### 4. Implementation Tasks (tasks.md)
Create a detailed implementation checklist:
```markdown
## 1. Analysis and Planning
- [ ] 1.1 Review existing specs in specs/[capability]/spec.md
- [ ] 1.2 Check for conflicting changes in changes/ directory
- [ ] 1.3 Read openspec/project.md for conventions

## 2. Implementation
- [ ] 2.1 [Specific implementation task]
- [ ] 2.2 [Another implementation task]
- [ ] 2.3 Write tests for new functionality
- [ ] 2.4 Update documentation

## 3. Verification
- [ ] 3.1 Run openspec validate [change-id] --strict
- [ ] 3.2 Test all scenarios manually
- [ ] 3.3 Get peer review and approval
```

### 5. Design Documentation (design.md) - ONLY if needed
Include design.md only if ANY of these apply:
- Cross-cutting change (multiple services/modules)
- New external dependency or significant data model changes
- Security, performance, or migration complexity
- Ambiguity that benefits from technical decisions

### 6. Three-Stage Workflow Plan
**Stage 1: Creating Changes**
- Scaffold all files under openspec/changes/[change-id]/
- Write proposal.md, tasks.md, and delta specs
- Run openspec validate [change-id] --strict
- Wait for approval before implementation

**Stage 2: Implementing Changes**
- Read proposal.md, design.md (if exists), and tasks.md
- Implement tasks sequentially
- Update checklist to reflect completion
- Ensure all tasks.md items are marked as completed

**Stage 3: Archiving Changes**
- After deployment, move changes/[change-id]/ → changes/archive/YYYY-MM-DD-[change-id]/
- Update specs/ if capabilities changed
- Run openspec validate --strict

## Critical Formatting Requirements

**Scenario Format (MUST use #### headers):**
```markdown
#### Scenario: User login success
- **WHEN** valid credentials provided
- **THEN** return JWT token
```

**Requirement Wording:**
- Use SHALL/MUST for normative requirements
- Every requirement MUST have at least one scenario
- Use proper delta operation headers: ## ADDED|MODIFIED|REMOVED|RENAMED Requirements

**Decision Tree for Scope:**
- Bug fix restoring intended behavior? → Fix directly (no proposal needed)
- New feature/capability? → Create proposal
- Breaking change? → Create proposal
- Architecture change? → Create proposal
- Unclear? → Create proposal (safer)

Generate a complete, practical OpenSpec plan following this methodology. Focus on what needs to be built, how it will be tested, and how the change will be managed through the full OpenSpec workflow.",
            self.ticket_number,
            self.ticket_title,
            self.ticket_description,
            if cleaned_acceptance_criteria.is_empty() {
                "No explicit acceptance criteria specified".to_string()
            } else {
                cleaned_acceptance_criteria
                    .iter()
                    .enumerate()
                    .map(|(i, ac)| format!("{}. {}", i + 1, ac))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        )
    }

    pub fn generate_filename(&self) -> String {
        // Create a concise title-based filename
        let concise_title = self.ticket_title
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
            .collect::<String>()
            .split_whitespace()
            .take(8) // Limit to 8 words
            .map(|word| word.to_lowercase())
            .collect::<Vec<_>>()
            .join("-");

        format!("{}-{}.md", self.ticket_number, concise_title)
    }
}

impl From<AzureWorkItemResponse> for WorkItem {
    fn from(azure_item: AzureWorkItemResponse) -> Self {
        let fields = azure_item.fields;

        // Extract basic fields
        let title = fields
            .get("System.Title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let description = fields
            .get("System.Description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let state = fields
            .get("System.State")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let work_item_type = fields
            .get("System.WorkItemType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let area_path = fields
            .get("System.AreaPath")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let iteration_path = fields
            .get("System.IterationPath")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let created_date = fields
            .get("System.CreatedDate")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let updated_date = fields
            .get("System.ChangedDate")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let created_by = fields
            .get("System.CreatedBy")
            .and_then(|v| v.as_str())
            .map(|email| User {
                display_name: email.split('@').next().unwrap_or(email).to_string(),
                email: email.to_string(),
                url: format!("mailto:{}", email),
            })
            .unwrap_or_else(|| User {
                display_name: "Unknown".to_string(),
                email: "unknown@example.com".to_string(),
                url: "".to_string(),
            });

        let assigned_to = fields
            .get("System.AssignedTo")
            .and_then(|v| v.as_str())
            .map(|email| User {
                display_name: email.split('@').next().unwrap_or(email).to_string(),
                email: email.to_string(),
                url: format!("mailto:{}", email),
            });

        // Extract acceptance criteria from description or custom field
        let acceptance_criteria = extract_acceptance_criteria(&description);

        Self {
            id: azure_item.id,
            title,
            description,
            acceptance_criteria,
            comments: Vec::new(), // Will be populated separately
            attachments: Vec::new(), // Will be populated from relations
            images: Vec::new(), // Will be extracted from description
            created_date,
            updated_date,
            created_by,
            assigned_to,
            state,
            work_item_type,
            area_path,
            iteration_path,
        }
    }
}

fn extract_acceptance_criteria(description: &str) -> Vec<String> {
    // Look for acceptance criteria patterns in the description
    let ac_patterns = [
        r"(?is)Acceptance Criteria:(.*?)(?=\n\n|\n#|\Z)",
        r"(?is)AC:(.*?)(?=\n\n|\n#|\Z)",
        r"(?is)Requirements:(.*?)(?=\n\n|\n#|\Z)",
        r"(?is)User Story:(.*?)(?=\n\n|\n#|\Z)",
    ];

    for pattern in &ac_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(description) {
                if let Some(ac_text) = caps.get(1) {
                    let criteria: Vec<String> = ac_text
                        .as_str()
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(|line| {
                            line.trim()
                                .trim_start_matches('-')
                                .trim_start_matches('*')
                                .trim_start_matches('#')
                                .trim()
                                .to_string()
                        })
                        .filter(|criterion| !criterion.is_empty())
                        .collect();

                    if !criteria.is_empty() {
                        return criteria;
                    }
                }
            }
        }
    }

    Vec::new()
}