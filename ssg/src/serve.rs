use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

pub fn watch_and_serve(config: &crate::config::SiteConfig) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )?;

    watcher.watch(Path::new(&config.content_dir), RecursiveMode::Recursive)?;
    watcher.watch(Path::new(&config.templates_dir), RecursiveMode::Recursive)?;
    watcher.watch(Path::new(&config.assets_dir), RecursiveMode::Recursive)?;

    println!("Watching for changes...");

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                println!("\nFile changed: {:?}\nRebuilding...", event.paths);
                if let Ok(config) = crate::config::load_config() {
                    if let Err(e) = crate::build::build_site(&config) {
                        eprintln!("Build error: {}", e);
                    } else {
                        println!("Build complete!");
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => break,
        }
    }

    Ok(())
}

pub fn serve_with_live_reload(config: &crate::config::SiteConfig, port: u16) -> anyhow::Result<()> {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)?;

    println!("Server running at http://{}", addr);

    // Start watcher in background
    let config_clone = config.clone();
    std::thread::spawn(move || {
        watch_and_serve(&config_clone).ok();
    });

    loop {
        let (mut stream, _) = listener.accept().map_err(anyhow::Error::msg)?;
        let mut buffer = [0; 1024];

        if let Ok(bytes_read) = stream.read(&mut buffer) {
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);

            // Simple HTTP routing
            let (status, content, content_type) =
                if request.starts_with("GET / ") || request.starts_with("GET /index.html") {
                    serve_file("public/index.html")
                } else if request.starts_with("GET /ws") {
                    (
                        "HTTP/1.1 101 Switching Protocols\r\n\r\n".to_string(),
                        String::new(),
                        "text/plain",
                    )
                } else if request.starts_with("GET /") {
                    let path = request
                        .lines()
                        .next()
                        .unwrap_or("")
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("/");

                    let file_path = format!("public{}", path);
                    let (status, content, _) = if path.ends_with("/") || !path.contains(".") {
                        serve_file(&format!("{}/index.html", file_path))
                    } else {
                        serve_file(&file_path)
                    };

                    // Determine content type by file extension
                    let ext = path.rsplit('.').next().unwrap_or("");
                    let mime = match ext {
                        "html" | "htm" => "text/html",
                        "css" => "text/css",
                        "js" => "application/javascript",
                        "json" => "application/json",
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "svg" => "image/svg+xml",
                        "ico" => "image/x-icon",
                        "xml" => "application/xml",
                        "txt" => "text/plain",
                        _ => "text/html",
                    };

                    (status, content, mime)
                } else {
                    (
                        "HTTP/1.1 404 Not Found\r\n".to_string(),
                        "Not Found".to_string(),
                        "text/plain",
                    )
                };

            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                status,
                content_type,
                content.len(),
                content
            );

            let _ = stream.write_all(response.as_bytes());
        }
    }
}

fn serve_file(path: &str) -> (String, String, &'static str) {
    match std::fs::read_to_string(path) {
        Ok(content) => (format!("HTTP/1.1 200 OK\r\n"), content, "text/html"),
        Err(_) => (
            "HTTP/1.1 404 Not Found\r\n".to_string(),
            "Not Found".to_string(),
            "text/plain",
        ),
    }
}
