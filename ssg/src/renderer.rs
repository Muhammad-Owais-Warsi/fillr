use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tera::Tera;

use crate::config::SiteConfig;
use crate::types::Page;

pub struct Renderer {
    pub tera: Arc<Tera>,
}

impl Renderer {
    pub fn new(template_path: &str) -> Self {
        let tera = match Tera::new(template_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Template parsing error: {}", e);
                std::process::exit(1);
            }
        };
        Renderer {
            tera: Arc::new(tera),
        }
    }

    pub fn render_page(
        &self,
        page: &Page,
        config: &SiteConfig,
        collections: &HashMap<String, Vec<&Page>>,
    ) -> anyhow::Result<()> {
        let layout = page.layout();

        let mut context = tera::Context::new();

        context.insert("title", page.title());
        context.insert("slug", &page.slug);
        context.insert("content", &page.content_html);
        context.insert(
            "description",
            page.frontmatter.description.as_deref().unwrap_or(""),
        );
        context.insert("date", page.frontmatter.date.as_deref().unwrap_or(""));
        context.insert("tags", &page.frontmatter.tags);
        context.insert(
            "og_image",
            page.frontmatter.og_image.as_deref().unwrap_or(""),
        );

        context.insert(
            "site",
            &serde_json::json!({
                "title": config.title,
                "description": config.description,
                "base_url": config.base_url,
            }),
        );

        // Add collections to context
        let collections_json: HashMap<String, Vec<serde_json::Value>> = collections
            .iter()
            .map(|(k, v)| {
                let pages_json: Vec<serde_json::Value> = v
                    .iter()
                    .map(|p| {
                        serde_json::json!({
                            "title": p.title(),
                            "slug": p.slug,
                            "description": p.frontmatter.description,
                            "date": p.frontmatter.date,
                        })
                    })
                    .collect();
                (k.clone(), pages_json)
            })
            .collect();
        context.insert("collections", &collections_json);

        let template_name = format!("{}.html", layout);

        let rendered = self.tera.render(&template_name, &context)?;
        fs::write(&page.output_path, rendered)?;
        Ok(())
    }

    pub fn render_collection(
        &self,
        name: &str,
        pages: &[&Page],
        config: &SiteConfig,
    ) -> anyhow::Result<()> {
        let mut context = tera::Context::new();

        let title = match name {
            "posts" => "Blog Posts",
            "projects" => "Projects",
            _ => name,
        };

        context.insert("title", title);
        context.insert("collection", name);
        context.insert(
            "site",
            &serde_json::json!({
                "title": config.title,
                "description": config.description,
                "base_url": config.base_url,
            }),
        );

        let pages_json: Vec<serde_json::Value> = pages
            .iter()
            .map(|p| {
                serde_json::json!({
                    "title": p.title(),
                    "slug": p.slug,
                    "description": p.frontmatter.description,
                    "date": p.frontmatter.date,
                })
            })
            .collect();
        context.insert("pages", &pages_json);

        let rendered = self.tera.render("list.html", &context)?;
        let output_path = Path::new("public").join(name).join("index.html");
        fs::write(output_path, rendered)?;
        Ok(())
    }
}
