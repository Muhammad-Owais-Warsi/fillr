use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub layout: Option<String>,
    pub draft: Option<bool>,
    pub order: Option<i32>,
    pub og_image: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Page {
    pub slug: String,
    pub frontmatter: Frontmatter,
    pub content_html: String,
    pub content_markdown: String,
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub links: Vec<String>,
    pub collection: Option<String>,
}

impl Page {
    pub fn title(&self) -> &str {
        self.frontmatter.title.as_deref().unwrap_or(&self.slug)
    }

    pub fn layout(&self) -> &str {
        self.frontmatter.layout.as_deref().unwrap_or("page")
    }

    pub fn is_draft(&self) -> bool {
        self.frontmatter.draft.unwrap_or(false)
    }
}
