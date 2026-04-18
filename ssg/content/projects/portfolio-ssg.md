---
title: "Portfolio SSG"
slug: "portfolio-ssg"
date: "2026-04-01"
description: "A blazing fast static site generator built with Rust"
tags: ["rust", "ssg", "open-source"]
layout: "page"
featured_image: "/images/project1.jpg"
---

# Portfolio SSG

This is the project I'm currently building - a high-performance static site generator.

## Features

- Parallel processing with Rayon
- Memory-mapped file reading
- Incremental builds
- Template caching

## Tech Stack

- **Rust** - Memory-safe, fast
- **Rushdown** - Fast Markdown parser
- **Tera** - Template engine

## Performance

Build times for 10,000 pages:
- First build: ~2 seconds
- Incremental: ~50ms

[View on GitHub](https://github.com/example/ssg)