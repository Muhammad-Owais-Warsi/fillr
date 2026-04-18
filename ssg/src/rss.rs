use crate::config::SiteConfig;
use crate::types::Page;
use std::fs;
use std::path::Path;

pub fn generate_rss(pages: &[Page], config: &SiteConfig) -> anyhow::Result<String> {
    let posts: Vec<&Page> = pages
        .iter()
        .filter(|p| p.collection.as_deref() == Some("posts"))
        .collect();

    let mut rss = String::new();
    rss.push_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
    <title>"#,
    );
    rss.push_str(&config.title);
    rss.push_str("</title>\n    <description>");
    rss.push_str(&config.description);
    rss.push_str("</description>\n    <link>");
    rss.push_str(&config.base_url);
    rss.push_str("</link>\n    <atom:link href=\"");
    rss.push_str(&config.base_url);
    rss.push_str("/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>\n");

    for post in posts.iter().take(20) {
        let url = format!("{}/posts/{}/", config.base_url, post.slug);
        rss.push_str("    <item>\n        <title>");
        rss.push_str(post.title());
        rss.push_str("</title>\n        <description>");
        rss.push_str(post.frontmatter.description.as_deref().unwrap_or(""));
        rss.push_str("</description>\n        <link>");
        rss.push_str(&url);
        rss.push_str("</link>\n        <guid>");
        rss.push_str(&url);
        rss.push_str("</guid>\n");
        if let Some(date) = &post.frontmatter.date {
            rss.push_str("        <pubDate>");
            rss.push_str(date);
            rss.push_str("</pubDate>\n");
        }
        rss.push_str("    </item>\n");
    }

    rss.push_str("</channel>\n</rss>");

    Ok(rss)
}

pub fn write_rss(pages: &[Page], config: &SiteConfig) -> anyhow::Result<()> {
    let rss = generate_rss(pages, config)?;
    fs::write("public/feed.xml", rss)?;
    println!("Generated feed.xml");
    Ok(())
}
