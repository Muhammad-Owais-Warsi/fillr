use crate::config::SiteConfig;
use crate::renderer::Renderer;
use crate::types::Page;
use std::collections::HashMap;
use std::fs;

pub fn collect_tags(pages: &[Page]) -> HashMap<String, Vec<&Page>> {
    let mut tags: HashMap<String, Vec<&Page>> = HashMap::new();

    for page in pages {
        for tag in &page.frontmatter.tags {
            tags.entry(tag.clone()).or_default().push(page);
        }
    }

    tags
}

pub fn generate_tag_pages(
    tags: &HashMap<String, Vec<&Page>>,
    config: &SiteConfig,
    renderer: &Renderer,
) -> anyhow::Result<()> {
    for (tag, pages) in tags {
        let title = format!("Tagged: {}", tag);

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

        let mut context = tera::Context::new();
        context.insert("title", &title);
        context.insert("tag", tag);
        context.insert("pages", &pages_json);
        context.insert(
            "site",
            &serde_json::json!({
                "title": config.title,
                "description": config.description,
                "base_url": config.base_url,
            }),
        );

        let rendered = renderer.tera.render("list.html", &context)?;

        // Create URL-safe tag name
        let safe_tag = tag.replace(' ', "-").to_lowercase();
        let output_path = format!("public/tags/{}/index.html", safe_tag);

        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&output_path, rendered)?;
        println!("Generated tags/{}", safe_tag);
    }

    Ok(())
}
