mod build;
mod config;
mod content;
mod graph;
mod parser;
mod renderer;
mod rss;
mod serve;
mod sitemap;
mod tags;
mod types;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ssg")]
#[command(version = "0.1.0")]
#[command(about = "A blazing fast static site generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new site
    Init {
        /// Name of the project
        name: Option<String>,
    },
    /// Build the site
    Build {
        /// Watch for changes and rebuild
        #[arg(short, long)]
        watch: bool,
    },
    /// Serve the site with live reload
    Serve {
        /// Port to serve on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Open browser automatically
        #[arg(short, long)]
        open: bool,
    },
    /// Clean build artifacts
    Clean,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => init_site(name),
        Commands::Build { watch } => build_site(watch),
        Commands::Serve { port, open } => serve_site(port, open),
        Commands::Clean => clean(),
    }
}

fn init_site(name: Option<String>) -> anyhow::Result<()> {
    let dir = match name {
        Some(n) => PathBuf::from(n),
        None => PathBuf::from("."),
    };

    if dir.join("config.toml").exists() {
        anyhow::bail!("Site already exists in this directory");
    }

    println!("Creating new SSG site...");

    // Create directories
    let dirs = ["content/posts", "content/projects", "templates", "static"];
    for d in dirs {
        std::fs::create_dir_all(dir.join(d))?;
    }

    // Create config.toml
    let config = r#"title = "My Site"
description = "A blazing fast static site"
base_url = "http://localhost:8080"
output_dir = "public"
content_dir = "content"
assets_dir = "static"
templates_dir = "templates"

[build]
drafts = true
future = true
"#;
    std::fs::write(dir.join("config.toml"), config)?;

    // Create sample content
    let index_content = r#"---
title: "Welcome"
slug: "index"
date: "2026-04-18"
description: "Welcome to my site"
layout: "page"
---

# Welcome

This is my new site built with SSG.

## Getting Started

1. Edit content in the `content/` folder
2. Customize templates in `templates/`
3. Add static files in `static/`

Run `ssg serve` to preview your site!
"#;
    std::fs::write(dir.join("content").join("index.md"), index_content)?;

    let sample_post = r#"---
title: "Hello World"
slug: "hello-world"
date: "2026-04-18"
description: "My first blog post"
tags: ["intro", "hello"]
layout: "post"
---

# Hello World

This is my first blog post!

## What's Next

More posts coming soon...
"#;
    std::fs::write(dir.join("content/posts/hello-world.md"), sample_post)?;

    println!("\nSite created successfully!");
    println!("\nNext steps:");
    println!("  cd {}", dir.display());
    println!("  ssg serve");
    println!("\nEdit config.toml to customize your site.");

    Ok(())
}

fn build_site(watch: bool) -> anyhow::Result<()> {
    let config = config::load_config()?;
    build::build_site(&config)?;

    if watch {
        println!("\nWatching for changes... (press Ctrl+C to stop)");
        serve::watch_and_serve(&config)?;
    }

    Ok(())
}

fn serve_site(port: u16, _open: bool) -> anyhow::Result<()> {
    // First build
    let config = config::load_config()?;
    build::build_site(&config)?;

    println!("\nServing at http://localhost:{}", port);
    println!("Watching for changes...");

    serve::serve_with_live_reload(&config, port)?;

    Ok(())
}

fn clean() -> anyhow::Result<()> {
    let config = config::load_config().unwrap_or_default();
    let output = PathBuf::from(&config.output_dir);

    if output.exists() {
        std::fs::remove_dir_all(&output)?;
        println!("Cleaned {}", output.display());
    }

    // Clean .ssg cache
    let cache = PathBuf::from(".ssg");
    if cache.exists() {
        std::fs::remove_dir_all(&cache)?;
        println!("Cleaned .ssg cache");
    }

    Ok(())
}
