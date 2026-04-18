use crate::types::Page;
use std::collections::HashMap;

pub fn build_dependency_graph(pages: &[Page]) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for page in pages {
        for link in &page.links {
            graph
                .entry(link.clone())
                .or_default()
                .push(page.slug.clone());
        }
    }

    graph
}
