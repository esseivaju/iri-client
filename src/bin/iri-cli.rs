use std::fs;
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};
use iri_client::IriClient;
use reqwest::Method;
use serde_json::Value;

#[derive(Debug, Parser)]
#[command(
    name = "iri-cli",
    version,
    about = "Small async CLI for querying the IRI API"
)]
struct Cli {
    /// Base URL for the API. Defaults to OpenAPI server URL.
    #[arg(long, env = "IRI_BASE_URL")]
    base_url: Option<String>,

    /// Raw access token value sent in Authorization header.
    #[arg(long, env = "IRI_ACCESS_TOKEN")]
    access_token: Option<String>,

    /// Emit compact JSON instead of pretty-printed output.
    #[arg(long)]
    compact: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List generated OpenAPI operation ids.
    Operations {
        /// Filter operations by substring match on operation id (case-sensitive).
        #[arg(long)]
        filter: Option<String>,
    },
    /// Call an endpoint by OpenAPI operation id.
    Call(CallArgs),
    /// Send a raw HTTP request using method + path.
    Request(RequestArgs),
}

#[derive(Debug, Args)]
struct CallArgs {
    /// OpenAPI operation id (for example: getResources).
    operation_id: String,

    /// Path parameter in form key=value. Repeat as needed.
    #[arg(long = "path-param", value_name = "KEY=VALUE")]
    path_param: Vec<String>,

    /// Query parameter in form key=value. Repeat as needed.
    #[arg(long = "query", value_name = "KEY=VALUE")]
    query: Vec<String>,

    #[command(flatten)]
    body: BodyInput,
}

#[derive(Debug, Args)]
struct RequestArgs {
    /// HTTP method (GET, POST, PUT, DELETE, ...).
    method: String,

    /// Request path (for example: /api/v1/facility/sites).
    path: String,

    /// Query parameter in form key=value. Repeat as needed.
    #[arg(long = "query", value_name = "KEY=VALUE")]
    query: Vec<String>,

    #[command(flatten)]
    body: BodyInput,
}

#[derive(Debug, Args)]
struct BodyInput {
    /// JSON request body literal.
    #[arg(long, conflicts_with = "body_file")]
    body_json: Option<String>,

    /// Path to a file containing a JSON request body.
    #[arg(long, value_name = "PATH", conflicts_with = "body_json")]
    body_file: Option<PathBuf>,
}

/// Entry point for the async CLI.
///
/// Parses command-line arguments, builds an authenticated/unauthenticated client,
/// dispatches subcommands, and prints JSON output.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // `operations` is metadata-only; it does not require constructing an HTTP client.
    if let Command::Operations { filter } = &cli.command {
        print_operations(filter.as_deref());
        return Ok(());
    }

    let mut client = match &cli.base_url {
        Some(url) => IriClient::new(url)?,
        None => IriClient::from_openapi_default_server()?,
    };

    if let Some(token) = &cli.access_token {
        client = client.with_authorization_token(token.clone());
    }

    let output = match &cli.command {
        Command::Operations{ .. } => unreachable!("handled above"),
        Command::Call(args) => call_operation(&client, args).await?,
        Command::Request(args) => send_request(&client, args).await?,
    };

    print_json(&output, cli.compact)?;
    Ok(())
}

/// Prints the generated OpenAPI operation catalog.
///
/// When `filter` is provided, only operation ids containing that substring are shown.
fn print_operations(filter: Option<&str>) {
    for operation in IriClient::operations() {
        if let Some(needle) = filter
            && !operation.operation_id.contains(needle)
        {
            continue;
        }
        println!(
            "{}\t{}\t{}",
            operation.operation_id, operation.method, operation.path_template
        );
    }
}

/// Calls a generated OpenAPI operation by `operation_id`.
///
/// Parses path/query pairs and optional JSON body from CLI args, then forwards
/// the request to `IriClient::call_operation`.
async fn call_operation(
    client: &IriClient,
    args: &CallArgs,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Parse repeatable `key=value` args into owned pairs first, then borrow as `&str`
    // for the client call to avoid temporary lifetime issues.
    let path_params = parse_pairs(&args.path_param, "--path-param")?;
    let query = parse_pairs(&args.query, "--query")?;
    let body = parse_body(&args.body)?;

    let borrowed_path: Vec<(&str, &str)> = path_params
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let borrowed_query: Vec<(&str, &str)> = query
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();

    let value = client
        .call_operation(&args.operation_id, &borrowed_path, &borrowed_query, body)
        .await?;
    Ok(value)
}

/// Sends a raw HTTP request using method + path.
///
/// This bypasses operation-id lookup and calls
/// `IriClient::request_json_with_query` directly.
async fn send_request(
    client: &IriClient,
    args: &RequestArgs,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Validate method eagerly so CLI errors are explicit before any network call.
    let method = Method::from_str(&args.method).map_err(|e| {
        IoError::new(
            ErrorKind::InvalidInput,
            format!("invalid HTTP method '{}': {e}", args.method),
        )
    })?;
    let query = parse_pairs(&args.query, "--query")?;
    let body = parse_body(&args.body)?;
    let borrowed_query: Vec<(&str, &str)> = query
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();

    let value = client
        .request_json_with_query(method, &args.path, &borrowed_query, body)
        .await?;
    Ok(value)
}

/// Parses repeated `key=value` arguments into owned key/value pairs.
///
/// Returns an error when a value does not include `=` or has an empty key.
fn parse_pairs(
    values: &[String],
    flag_name: &str,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    // Shared parser for `--query` and `--path-param` arguments.
    let mut pairs = Vec::with_capacity(values.len());
    for item in values {
        let (key, value) = item.split_once('=').ok_or_else(|| {
            IoError::new(
                ErrorKind::InvalidInput,
                format!("invalid {flag_name} value '{item}': expected key=value"),
            )
        })?;
        if key.is_empty() {
            return Err(IoError::new(
                ErrorKind::InvalidInput,
                format!("invalid {flag_name} value '{item}': empty key"),
            )
            .into());
        }
        pairs.push((key.to_owned(), value.to_owned()));
    }
    Ok(pairs)
}

/// Parses an optional JSON body from inline text or a file path.
///
/// Exactly one of `--body-json` or `--body-file` may be set.
fn parse_body(body: &BodyInput) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    match (&body.body_json, &body.body_file) {
        // Inline JSON body for quick ad-hoc calls.
        (Some(raw), None) => Ok(Some(serde_json::from_str(raw)?)),
        (None, Some(path)) => {
            // File-based body for larger payloads and reusable fixtures.
            let raw = fs::read_to_string(path)?;
            Ok(Some(serde_json::from_str(&raw)?))
        }
        (None, None) => Ok(None),
        (Some(_), Some(_)) => Err(IoError::new(
            ErrorKind::InvalidInput,
            "use only one of --body-json or --body-file",
        )
        .into()),
    }
}

/// Prints a JSON value either compact or pretty-formatted.
fn print_json(value: &Value, compact: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Keep output machine-friendly by defaulting to valid JSON in both modes.
    if compact {
        println!("{}", serde_json::to_string(value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}
