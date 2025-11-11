use crate::models::*;
use anyhow::Result;
use std::fs;
use tracing::{debug, info};

pub struct FileSystemOrganizer {
    base_path: String,
    tickets_path: String,
    openspec_path: String,
}

impl FileSystemOrganizer {
    pub fn new(base_path: &str) -> Self {
        let base_path = base_path.to_string();
        Self {
            tickets_path: format!("{}/Tickets", base_path),
            openspec_path: format!("{}/openspec", base_path),
            base_path,
        }
    }

    pub fn ensure_base_structure(&self) -> Result<()> {
        // Create base directories
        fs::create_dir_all(&self.base_path)?;
        fs::create_dir_all(&self.tickets_path)?;
        fs::create_dir_all(&self.openspec_path)?;

        info!("Created base directory structure at {}", self.base_path);
        Ok(())
    }

    pub async fn save_work_item(&self, work_item: &WorkItem) -> Result<String> {
        let ticket_path = format!("{}/{}", self.tickets_path, work_item.id);

        // Create ticket-specific directories
        fs::create_dir_all(&ticket_path)?;
        fs::create_dir_all(format!("{}/attachments", ticket_path))?;
        fs::create_dir_all(format!("{}/images", ticket_path))?;
        fs::create_dir_all(format!("{}/comments", ticket_path))?;

        info!("Saving work item {} to {}", work_item.id, ticket_path);

        // Save metadata
        self.save_metadata(work_item, &ticket_path)?;

        // Save description with image placeholders
        self.save_description(work_item, &ticket_path)?;

        // Save acceptance criteria
        self.save_acceptance_criteria(work_item, &ticket_path)?;

        // Save comments
        self.save_comments(work_item, &ticket_path)?;

        // Save attachment manifest
        self.save_attachment_manifest(work_item, &ticket_path)?;

        // Save image manifest
        self.save_image_manifest(work_item, &ticket_path)?;

        info!("Successfully saved work item {} to {}", work_item.id, ticket_path);
        Ok(ticket_path)
    }

    fn save_metadata(&self, work_item: &WorkItem, ticket_path: &str) -> Result<()> {
        let metadata_path = format!("{}/metadata.json", ticket_path);

        let metadata = serde_json::json!({
            "id": work_item.id,
            "title": work_item.title,
            "state": work_item.state,
            "work_item_type": work_item.work_item_type,
            "area_path": work_item.area_path,
            "iteration_path": work_item.iteration_path,
            "created_date": work_item.created_date,
            "updated_date": work_item.updated_date,
            "created_by": {
                "display_name": work_item.created_by.display_name,
                "email": work_item.created_by.email
            },
            "assigned_to": work_item.assigned_to.as_ref().map(|user| serde_json::json!({
                "display_name": user.display_name,
                "email": user.email
            })),
            "stats": {
                "attachments_count": work_item.attachments.len(),
                "comments_count": work_item.comments.len(),
                "images_count": work_item.images.len(),
                "acceptance_criteria_count": work_item.acceptance_criteria.len()
            }
        });

        fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;
        debug!("Saved metadata to {}", metadata_path);
        Ok(())
    }

    fn save_description(&self, work_item: &WorkItem, ticket_path: &str) -> Result<()> {
        let description_path = format!("{}/description.md", ticket_path);

        // Clean HTML content and replace image URLs with placeholders
        let cleaned_description = clean_html_content(&work_item.description);
        let processed_description = self.replace_image_placeholders(&cleaned_description, &work_item.images);

        let content = format!("# {}\n\n**Work Item ID**: {}\n\n**State**: {}\n\n**Type**: {}\n\n**Created**: {}\n\n**Created By**: {}\n\n---\n\n## Description\n\n{}",
            work_item.title,
            work_item.id,
            work_item.state,
            work_item.work_item_type,
            work_item.created_date.format("%Y-%m-%d %H:%M:%S UTC"),
            work_item.created_by.display_name,
            processed_description
        );

        fs::write(&description_path, content)?;
        debug!("Saved description to {}", description_path);
        Ok(())
    }

    fn save_acceptance_criteria(&self, work_item: &WorkItem, ticket_path: &str) -> Result<()> {
        let ac_path = format!("{}/acceptance-criteria.md", ticket_path);

        if work_item.acceptance_criteria.is_empty() {
            let content = "# Acceptance Criteria\n\nNo explicit acceptance criteria specified in the work item.";
            fs::write(&ac_path, content)?;
        } else {
            let cleaned_criteria = clean_text_content_list(&work_item.acceptance_criteria);
            let content = format!("# Acceptance Criteria\n\n{}",
                cleaned_criteria
                    .iter()
                    .enumerate()
                    .map(|(i, ac)| format!("{}. {}", i + 1, ac))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            );
            fs::write(&ac_path, content)?;
        }

        debug!("Saved acceptance criteria to {}", ac_path);
        Ok(())
    }

    fn save_comments(&self, work_item: &WorkItem, ticket_path: &str) -> Result<()> {
        let comments_dir = format!("{}/comments", ticket_path);

        if work_item.comments.is_empty() {
            // Create a placeholder file indicating no comments
            let placeholder_path = format!("{}/no-comments.md", comments_dir);
            let content = "# Comments\n\nNo comments found for this work item.";
            fs::write(&placeholder_path, content)?;
        } else {
            for (index, comment) in work_item.comments.iter().enumerate() {
                let comment_filename = format!("comment_{:03}.json", index + 1);
                let comment_path = format!("{}/{}", comments_dir, comment_filename);

                let comment_data = serde_json::json!({
                    "id": comment.id,
                    "author": {
                        "display_name": comment.author.display_name,
                        "email": comment.author.email
                    },
                    "created_date": comment.created_date,
                    "updated_date": comment.updated_date,
                    "text": clean_html_content(&comment.text),
                    "images": comment.images.iter().map(|img| serde_json::json!({
                        "placeholder": img.placeholder,
                        "original_url": img.original_url,
                        "local_path": img.local_path,
                        "alt_text": img.alt_text
                    })).collect::<Vec<_>>()
                });

                fs::write(&comment_path, serde_json::to_string_pretty(&comment_data)?)?;

                // Also save as markdown for readability
                let markdown_filename = format!("comment_{:03}.md", index + 1);
                let markdown_path = format!("{}/{}", comments_dir, markdown_filename);

                let processed_text = self.replace_image_placeholders(&comment.text, &comment.images);
                let markdown_content = format!(
                    "# Comment by {}\n\n**Date**: {}\n\n---\n\n{}",
                    comment.author.display_name,
                    comment.created_date.format("%Y-%m-%d %H:%M:%S UTC"),
                    processed_text
                );

                fs::write(&markdown_path, markdown_content)?;
            }
        }

        debug!("Saved {} comments to {}", work_item.comments.len(), comments_dir);
        Ok(())
    }

    fn save_attachment_manifest(&self, work_item: &WorkItem, ticket_path: &str) -> Result<()> {
        let manifest_path = format!("{}/attachments/manifest.json", ticket_path);

        let manifest = serde_json::json!({
            "attachments": work_item.attachments.iter().map(|att| serde_json::json!({
                "id": att.id,
                "filename": att.filename,
                "original_url": att.url,
                "local_path": att.local_path,
                "content_type": att.content_type,
                "size_bytes": att.size,
                "created_date": att.created_date
            })).collect::<Vec<_>>()
        });

        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
        debug!("Saved attachment manifest to {}", manifest_path);
        Ok(())
    }

    fn save_image_manifest(&self, work_item: &WorkItem, ticket_path: &str) -> Result<()> {
        let manifest_path = format!("{}/images/manifest.json", ticket_path);

        let manifest = serde_json::json!({
            "images": work_item.images.iter().map(|img| serde_json::json!({
                "placeholder": img.placeholder,
                "original_url": img.original_url,
                "local_path": img.local_path,
                "width": img.width,
                "height": img.height,
                "alt_text": img.alt_text
            })).collect::<Vec<_>>()
        });

        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
        debug!("Saved image manifest to {}", manifest_path);
        Ok(())
    }

    fn replace_image_placeholders(&self, text: &str, images: &[ImageReference]) -> String {
        let mut processed_text = text.to_string();

        for image in images {
            // Replace the original image URL with the placeholder
            processed_text = processed_text.replace(&image.original_url, &format!("images/{}", image.placeholder));

            // Also replace any remaining HTML img tags with markdown
            processed_text = regex::Regex::new(&format!(r#"<img[^>]*src="{}"[^>]*>"#, regex::escape(&image.original_url)))
                .unwrap()
                .replace_all(&processed_text, format!("![{}]({})",
                    image.alt_text.as_deref().unwrap_or("image"),
                    format!("images/{}", image.placeholder)
                ))
                .to_string();
        }

        processed_text
    }

    pub fn generate_openspec_plan_data(&self, work_item: &WorkItem) -> OpenSpecPlanData {
        // Debug log the raw description from API
        info!("Raw API description length: {} chars", work_item.description.len());
        info!("Raw API description preview: {}", &work_item.description[..work_item.description.len().min(100)]);

        let cleaned_description = clean_html_content(&work_item.description);
        info!("Cleaned description length: {} chars", cleaned_description.len());
        info!("Cleaned description preview: {}", &cleaned_description[..cleaned_description.len().min(100)]);

        OpenSpecPlanData {
            ticket_number: work_item.id,
            ticket_title: work_item.title.clone(),
            ticket_description: cleaned_description,
            acceptance_criteria: work_item.acceptance_criteria.clone(),
            priority: self.extract_priority(&work_item.area_path),
            complexity: self.estimate_complexity(work_item),
            dependencies: self.extract_dependencies(&work_item.description),
            estimated_effort: None, // Would need additional logic or API call
            attachments_count: work_item.attachments.len(),
            comments_count: work_item.comments.len(),
            has_images: !work_item.images.is_empty(),
        }
    }

    fn strip_html(&self, text: &str) -> String {
        // Remove HTML tags while preserving some formatting
        let clean_text = regex::Regex::new(r#"<[^>]*>"#)
            .unwrap()
            .replace_all(text, "");

        // Clean up extra whitespace
        regex::Regex::new(r"\n\s*\n\s*\n")
            .unwrap()
            .replace_all(&clean_text, "\n\n")
            .to_string()
            .trim()
            .to_string()
    }

    fn extract_priority(&self, area_path: &str) -> String {
        // Extract priority from area path if possible
        if area_path.to_lowercase().contains("critical") || area_path.to_lowercase().contains("urgent") {
            "High".to_string()
        } else if area_path.to_lowercase().contains("normal") {
            "Medium".to_string()
        } else if area_path.to_lowercase().contains("low") {
            "Low".to_string()
        } else {
            "Medium".to_string() // Default
        }
    }

    fn estimate_complexity(&self, work_item: &WorkItem) -> String {
        let description_length = work_item.description.len();
        let acceptance_criteria_count = work_item.acceptance_criteria.len();
        let attachments_count = work_item.attachments.len();
        let comments_count = work_item.comments.len();

        // Simple heuristic-based complexity estimation
        let complexity_score = description_length / 100 + acceptance_criteria_count * 2 + attachments_count + comments_count;

        match complexity_score {
            0..=10 => "Low".to_string(),
            11..=50 => "Medium".to_string(),
            51..=100 => "High".to_string(),
            _ => "Very High".to_string(),
        }
    }

    fn extract_dependencies(&self, description: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Look for work item references in the description
        let work_item_regex = regex::Regex::new(r"#(\d+)").unwrap();
        for cap in work_item_regex.captures_iter(description) {
            if let Some(id_match) = cap.get(1) {
                dependencies.push(format!("Work Item #{}", id_match.as_str()));
            }
        }

        dependencies
    }
}