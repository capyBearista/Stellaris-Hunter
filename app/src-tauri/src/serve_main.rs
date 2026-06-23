//! Entry point for the `stellaris-hunter-serve` HTTP sidecar binary.
//!
//! Delegates to `stellaris_hunter_scan::serve::run()` with CLI args parsed
//! by clap.

use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = stellaris_hunter_scan::serve::ServeArgs::parse();
    stellaris_hunter_scan::serve::run(args).await
}
