/// Index all commit stubs in `.Orqestra/graph/commits/` into triples.
use std::path::PathBuf;

use graph_store::{TripleStore, index_commits};

fn main() {
    let project_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let triples_dir = project_root.join(".Orqestra/graph/triples");
    let commits_dir = project_root.join(".Orqestra/graph/commits");

    println!("Indexing commits from {:?}", commits_dir);

    let store = TripleStore::load(triples_dir).expect("Failed to load triple store");
    let count = index_commits(&store, &commits_dir).expect("Failed to index commits");

    println!("Indexed {} triples from commit stubs", count);
    println!("Total triples in store: {}", store.len());

    // Show some example queries
    println!("\n--- Example queries ---");

    let all_intents = store.query(None, Some("has_intent"), None);
    println!("All commit intents:");
    for t in &all_intents {
        println!("  {} → {}", t.subject, t.object);
    }

    let concepts = store.query(None, Some("affects_concept"), None);
    println!("\nAffected concepts:");
    for t in &concepts {
        println!("  {} → {}", t.subject, t.object);
    }
}
