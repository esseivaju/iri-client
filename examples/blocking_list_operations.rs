//! List generated OpenAPI operations from the Rust client.
//!
//! Run:
//! `cargo run --example blocking_list_operations`

use iri_client::{BlockingIriClient, openapi_default_server_url};

fn main() {
    println!("Default OpenAPI server: {}", openapi_default_server_url());

    let operations = BlockingIriClient::operations();
    println!("Loaded {} operations", operations.len());
    println!("First 20 operations:");

    for operation in operations.iter().take(20) {
        println!(
            "- {:<6} {:<60} ({})",
            operation.method, operation.path_template, operation.operation_id
        );
    }
}
