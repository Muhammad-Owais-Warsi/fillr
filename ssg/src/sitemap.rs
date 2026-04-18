use crate::config::SiteConfig;
use crate::types::Page;
use std::fs;

pub fn generate_sitemap(pages: &[Page], config: &SiteConfig) -> String {
    let mut sitemap = String::new();
    sitemap.push_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    // Add home page
    sitemap.push_str("  <url>\n    <loc>");
    sitemap.push_str(&config.base_url);
    sitemap.push_str("/</loc>\n    <changefreq>weekly</changefreq>\n  </url>\n");

    // Add all pages
    for page in pages {
        let loc = match &page.collection {
            Some(col) => format!("{}/{}/{}/", config.base_url, col, page.slug),
            None => format!("{}/{}/", config.base_url, page.slug),
        };

        sitemap.push_str("  <url>\n    <loc>");
        sitemap.push_str(&loc);
        sitemap.push_str("</loc>\n    <changefreq>monthly</changefreq>\n  </url>\n");
    }

    sitemap.push_str("</urlset>");
    sitemap
}

pub fn write_sitemap(pages: &[Page], config: &SiteConfig) -> anyhow::Result<()> {
    let sitemap = generate_sitemap(pages, config);
    fs::write("public/sitemap.xml", sitemap)?;
    println!("Generated sitemap.xml");
    Ok(())
}
