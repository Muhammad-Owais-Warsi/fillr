use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use rayon::prelude::*;

use crate::config::SiteConfig;
use crate::content;
use crate::parser::parse_page;
use crate::renderer::Renderer;
use crate::rss;
use crate::sitemap;
use crate::tags;
use crate::types::Page;

pub fn build_site(config: &crate::config::SiteConfig) -> anyhow::Result<()> {
    let renderer = Renderer::new("templates/**/*.html");

    let content_dir = Path::new(&config.content_dir);
    let output_dir = Path::new(&config.output_dir);
    let assets_dir = Path::new(&config.assets_dir);

    let files = content::walk_dir(content_dir)?;

    let templates_mtime = fs::read_dir(&config.templates_dir)?
        .filter_map(|e| e.ok()?.metadata().ok()?.modified().ok())
        .max()
        .unwrap_or(SystemTime::UNIX_EPOCH);

    content::copy_assets(assets_dir, output_dir)?;

    let mut pages: Vec<Page> = files
        .iter()
        .filter_map(|path| match parse_page(path) {
            Ok(page) => {
                if page.is_draft() && !config.build.drafts {
                    println!("Skipping draft: {}", page.slug);
                    None
                } else {
                    Some(page)
                }
            }
            Err(e) => {
                eprintln!("Error parsing {:?}: {}", path, e);
                None
            }
        })
        .collect();

    // Sort pages by date (newest first) for posts
    pages.sort_by(|a, b| match (&a.frontmatter.date, &b.frontmatter.date) {
        (Some(a_date), Some(b_date)) => b_date.cmp(a_date),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    });

    // Group pages by collection
    let mut collections: HashMap<String, Vec<&Page>> = HashMap::new();
    for page in &pages {
        if let Some(collection) = &page.collection {
            collections
                .entry(collection.clone())
                .or_default()
                .push(page);
        }
    }

    let collections_mtime = collections
        .keys()
        .filter_map(|c| {
            let p = Path::new(output_dir).join(c).join("index.html");
            p.metadata().ok()?.modified().ok()
        })
        .max()
        .unwrap_or(SystemTime::UNIX_EPOCH);

    // Build individual pages
    let mut rebuild_count = 0;
    let mut skip_count = 0;

    for page in &pages {
        let needs_rebuild = if page.output_path.exists() {
            let source_mtime = fs::metadata(&page.source_path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let output_mtime = fs::metadata(&page.output_path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            !(output_mtime > source_mtime && output_mtime > templates_mtime)
        } else {
            true
        };

        if needs_rebuild {
            if let Some(parent) = page.output_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            println!("Building: {}", page.slug);
            renderer.render_page(&page, config, &collections)?;
            rebuild_count += 1;
        } else {
            skip_count += 1;
        }
    }

    // Build collection index pages - only if content changed
    for (collection_name, collection_pages) in &collections {
        let index_path = Path::new(output_dir)
            .join(collection_name)
            .join("index.html");

        let needs_rebuild = if index_path.exists() {
            let coll_mtime = collection_pages
                .iter()
                .filter_map(|p| fs::metadata(&p.source_path).ok()?.modified().ok())
                .max()
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let output_mtime = fs::metadata(&index_path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            !(output_mtime > coll_mtime
                && output_mtime > templates_mtime
                && output_mtime > collections_mtime)
        } else {
            true
        };

        if needs_rebuild {
            if let Some(parent) = index_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            println!("Building collection: {}", collection_name);
            renderer.render_collection(collection_name, collection_pages, config)?;
        }
    }

    // Generate RSS feed - only if posts changed
    let feed_path = Path::new(output_dir).join("feed.xml");
    let needs_rss = !feed_path.exists() || {
        let posts_mtime = pages
            .iter()
            .filter(|p| p.collection.as_deref() == Some("posts"))
            .filter_map(|p| fs::metadata(&p.source_path).ok()?.modified().ok())
            .max()
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let feed_mtime = fs::metadata(&feed_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        posts_mtime > feed_mtime
    };

    if needs_rss {
        if let Err(e) = rss::write_rss(&pages, config) {
            eprintln!("Error generating RSS: {}", e);
        }
    }

    // Generate sitemap
    let sitemap_path = Path::new(output_dir).join("sitemap.xml");
    if !sitemap_path.exists() {
        if let Err(e) = sitemap::write_sitemap(&pages, config) {
            eprintln!("Error generating sitemap: {}", e);
        }
    }

    // Generate tag pages
    let tag_map = tags::collect_tags(&pages);
    for (tag, tag_pages) in &tag_map {
        let tag_path = Path::new(output_dir)
            .join("tags")
            .join(&tag.to_lowercase())
            .join("index.html");
        let needs_tag = !tag_path.exists() || {
            let tag_mtime = tag_pages
                .iter()
                .filter_map(|p| fs::metadata(&p.source_path).ok()?.modified().ok())
                .max()
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let out_mtime = fs::metadata(&tag_path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            tag_mtime > out_mtime
        };

        if needs_tag {
            if let Err(e) = tags::generate_tag_pages(&tag_map, config, &renderer) {
                eprintln!("Error generating tag pages: {}", e);
            }
            break;
        }
    }

    println!(
        "\nBuild complete! {} built, {} skipped",
        rebuild_count, skip_count
    );
    Ok(())
}
