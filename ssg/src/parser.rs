use anyhow::Result;
use rushdown::markdown_to_html_string;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::types::{Frontmatter, Page};

pub fn parse_page(path: &Path) -> Result<Page> {
    let raw = fs::read_to_string(path)?;

    let (frontmatter, body) = parse_frontmatter(&raw);

    let slug = frontmatter.slug.clone().unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("index")
            .to_string()
    });

    let mut content_html = String::new();
    markdown_to_html_string(&mut content_html, body)
        .map_err(|e| anyhow::anyhow!("Markdown parse error: {:?}", e))?;

    let links = extract_links(body);

    let collection = detect_collection(path);
    let output_path = build_output_path(path, &slug, &collection);

    Ok(Page {
        slug,
        frontmatter,
        content_html,
        content_markdown: body.to_string(),
        source_path: path.to_path_buf(),
        output_path,
        links,
        collection,
    })
}

fn parse_frontmatter(content: &str) -> (Frontmatter, &str) {
    let parts: Vec<&str> = content.split("---").collect();

    if parts.len() >= 2 {
        let fm: Frontmatter = toml::from_str(parts[1]).unwrap_or_default();
        let body = parts.get(2).map(|s| s.trim()).unwrap_or("");
        (fm, body)
    } else {
        (Frontmatter::default(), content.trim())
    }
}

pub fn extract_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();

    for line in content.lines() {
        if let Some(start) = line.find("[") {
            if let Some(link_start) = line[start..].find("](") {
                let url_start = start + link_start + 2;
                if let Some(url_end) = line[url_start..].find(')') {
                    let url = &line[url_start..url_start + url_end];
                    if url.ends_with(".md") {
                        links.push(
                            url.trim_start_matches("./")
                                .trim_end_matches(".md")
                                .to_string(),
                        );
                    }
                }
            }
        }
    }
    links
}

fn detect_collection(path: &Path) -> Option<String> {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

fn build_output_path(_source: &Path, slug: &str, collection: &Option<String>) -> PathBuf {
    let mut parts = Vec::new();

    if let Some(col) = collection {
        parts.push(col.clone());
    }

    parts.push(slug.to_string());

    PathBuf::from("public")
        .join(parts.join("/"))
        .join("index.html")
}
