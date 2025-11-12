use crate::models::*;
use anyhow::{anyhow, Result};
use reqwest::Client;
use tracing::{debug, error, info};
use chrono::{DateTime, Utc};

pub struct AzureDevOpsClient {
    client: Client,
    organization: String,
    project: String,
    pat_token: String,
}

impl AzureDevOpsClient {
    pub fn new(organization: String, project: String, pat_token: String) -> Self {
        let client = Client::builder()
            .user_agent("bakery/0.1.0")
            .user_agent("Bakery Azure DevOps Scraper")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            organization,
            project,
            pat_token,
        }
    }

    pub async fn get_work_item(&self, id: u32) -> Result<WorkItem> {
        info!("Fetching work item {} from Azure DevOps", id);

        // First, try to get the work item without expand
        let work_item = match self.get_work_item_raw(id, "").await {
            Ok(item) => item,
            Err(_) => {
                // If that fails, try with expand
                self.get_work_item_raw(id, "$expand=Relations").await?
            }
        };

        // Convert to our internal model
        let mut result_work_item = WorkItem::from(work_item.clone());

        // Extract attachments from relations
        if let Some(relations) = work_item.relations {
            result_work_item.attachments = self.extract_attachments(relations).await?;
        }

        // Extract and download images from description
        result_work_item.images = self.extract_and_download_images(&result_work_item.description, id).await?;

        // Get comments
        result_work_item.comments = self.get_work_item_comments(id).await?;

        info!("Successfully fetched work item {} with {} attachments and {} comments",
              id, result_work_item.attachments.len(), result_work_item.comments.len());

        Ok(result_work_item)
    }

    async fn get_work_item_raw(&self, id: u32, expand: &str) -> Result<AzureWorkItemResponse> {
        let url = if expand.is_empty() {
            format!(
                "https://dev.azure.com/{}/_apis/wit/workitems/{}?api-version=7.1",
                self.organization, id
            )
        } else {
            format!(
                "https://dev.azure.com/{}/_apis/wit/workitems/{}?api-version=7.1&{}",
                self.organization, id, expand
            )
        };

        debug!("Making request to: {}", url);

        let response = match self
            .client
            .get(&url)
            .header("Authorization", format!("Basic {}", self.encode_pat()))
            .header("Accept", "application/json")
            .send()
            .await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Failed to connect to Azure DevOps API: {}", e);
                    return Err(anyhow!("Failed to connect to Azure DevOps API: {}. Check your network connection and organization URL.", e));
                }
            };

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
            let error_message = if error_text.is_empty() {
                format!("HTTP {} {} (URL: {})", status.as_u16(), status.canonical_reason().unwrap_or("Unknown Error"), url)
            } else {
                format!("HTTP {} - {} (URL: {})", status, error_text, url)
            };
            error!("Azure DevOps API error: {}", error_message);
            return Err(anyhow!("Failed to fetch work item: {}", error_message));
        }

        let work_item: AzureWorkItemResponse = response.json().await?;
        Ok(work_item)
    }

    async fn extract_attachments(&self, relations: Vec<AzureRelation>) -> Result<Vec<Attachment>> {
        let mut attachments = Vec::new();

        for relation in relations {
            if relation.rel == "AttachedFile" {
                if let Some(attributes) = relation.attributes {
                    if let Some(filename) = attributes.name {
                        match self.download_attachment(&relation.url, &filename).await {
                            Ok(attachment) => attachments.push(attachment),
                            Err(e) => {
                                error!("Failed to download attachment {}: {}", filename, e);
                                // Continue with other attachments even if one fails
                            }
                        }
                    }
                }
            }
        }

        Ok(attachments)
    }

    async fn download_attachment(&self, url: &str, filename: &str) -> Result<Attachment> {
        debug!("Downloading attachment: {} from {}", filename, url);

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Basic {}", self.encode_pat()))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to download attachment: {}", response.status()));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let size = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        // Create local file path
        let local_path = format!("X:/.OTCX/Tickets/temp/attachments/{}", filename);

        // Ensure directory exists
        std::fs::create_dir_all("X:/.OTCX/Tickets/temp/attachments")?;

        // Download the file content
        let content = response.bytes().await?;
        std::fs::write(&local_path, content)?;

        Ok(Attachment {
            id: rand::random::<u32>(),
            filename: filename.to_string(),
            url: url.to_string(),
            local_path,
            content_type,
            size,
            created_date: chrono::Utc::now(),
        })
    }

    async fn extract_and_download_images(&self, description: &str, work_item_id: u32) -> Result<Vec<ImageReference>> {
        let mut images = Vec::new();
        let img_regex = regex::Regex::new(r#"<img[^>]+src="([^"]+)"[^>]*(?:alt="([^"]*)")?[^>]*>"#)?;

        // Create images directory
        let images_dir = format!("X:/.OTCX/Tickets/{}/images", work_item_id);
        std::fs::create_dir_all(&images_dir)?;

        let mut image_counter = 1;

        for caps in img_regex.captures_iter(description) {
            if let (Some(img_url_match), alt_text) = (caps.get(1), caps.get(2)) {
                let img_url = img_url_match.as_str();
                let alt_text = alt_text.map(|m| m.as_str().to_string());

                // Only process Azure DevOps URLs
                if img_url.contains("dev.azure.com") || img_url.contains("visualstudio.com") {
                    let placeholder = format!("image{:03}.png", image_counter);
                    let local_path = format!("{}/{}", images_dir, placeholder);

                    match self.download_image(img_url, &local_path).await {
                        Ok(_) => {
                            images.push(ImageReference {
                                placeholder: placeholder.clone(),
                                original_url: img_url.to_string(),
                                local_path,
                                width: None,
                                height: None,
                                alt_text,
                            });
                            image_counter += 1;
                        }
                        Err(e) => {
                            error!("Failed to download image {}: {}", img_url, e);
                        }
                    }
                }
            }
        }

        Ok(images)
    }

    async fn download_image(&self, url: &str, local_path: &str) -> Result<()> {
        debug!("Downloading image: {} to {}", url, local_path);

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Basic {}", self.encode_pat()))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to download image: {}", response.status()));
        }

        let content = response.bytes().await?;
        std::fs::write(local_path, content)?;

        Ok(())
    }

    async fn get_work_item_comments(&self, work_item_id: u32) -> Result<Vec<Comment>> {
        info!("Fetching comments for work item {}", work_item_id);

        let url = format!(
            "https://dev.azure.com/{}/_apis/wit/workItems/{}/comments?api-version=7.1",
            self.organization, work_item_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Basic {}", self.encode_pat()))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            // Comments might not be available for all work items
            debug!("No comments available for work item {} or insufficient permissions", work_item_id);
            return Ok(Vec::new());
        }

        let comments_response: AzureCommentsResponse = response.json().await?;
        let mut comments = Vec::new();

        for azure_comment in comments_response.value {
            let created_date = azure_comment.created_date
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now());

            let updated_date = azure_comment.updated_date
                .and_then(|date| date.parse::<DateTime<Utc>>().ok());

            let author = User {
                display_name: azure_comment.author.displayName.clone(),
                email: azure_comment.author.url.clone(), // This might need extraction
                url: azure_comment.author.url,
            };

            // Extract images from comment text
            let comment_images = self.extract_and_download_images_from_text(
                &azure_comment.text,
                work_item_id,
                &format!("comment_{}", azure_comment.id)
            ).await.unwrap_or_default();

            comments.push(Comment {
                id: azure_comment.id,
                author,
                created_date,
                updated_date,
                text: azure_comment.text,
                images: comment_images,
            });
        }

        Ok(comments)
    }

    async fn extract_and_download_images_from_text(
        &self,
        text: &str,
        work_item_id: u32,
        context: &str
    ) -> Result<Vec<ImageReference>> {
        let mut images = Vec::new();
        let img_regex = regex::Regex::new(r#"<img[^>]+src="([^"]+)"[^>]*(?:alt="([^"]*)")?[^>]*>"#)?;

        let images_dir = format!("X:/.OTCX/Tickets/{}/images/{}", work_item_id, context);
        std::fs::create_dir_all(&images_dir)?;

        let mut image_counter = 1;

        for caps in img_regex.captures_iter(text) {
            if let (Some(img_url_match), alt_text) = (caps.get(1), caps.get(2)) {
                let img_url = img_url_match.as_str();
                let alt_text = alt_text.map(|m| m.as_str().to_string());

                if img_url.contains("dev.azure.com") || img_url.contains("visualstudio.com") {
                    let placeholder = format!("image{:03}.png", image_counter);
                    let local_path = format!("{}/{}", images_dir, placeholder);

                    match self.download_image(img_url, &local_path).await {
                        Ok(_) => {
                            images.push(ImageReference {
                                placeholder: placeholder.clone(),
                                original_url: img_url.to_string(),
                                local_path,
                                width: None,
                                height: None,
                                alt_text,
                            });
                            image_counter += 1;
                        }
                        Err(e) => {
                            error!("Failed to download image {}: {}", img_url, e);
                        }
                    }
                }
            }
        }

        Ok(images)
    }

    fn encode_pat(&self) -> String {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.encode(format!(":{}", self.pat_token))
    }
}