//! Diffyn's local MCP process. Standard output belongs exclusively to JSON-RPC.

mod prompts;
mod resources;
mod server;

use diffyn::tools;

use std::{process::ExitCode, time::Duration};

fn main() -> ExitCode {
    // Construct the runtime explicitly: Tokio's standard-input fallback uses an
    // uncancellable blocking read on macOS and Windows. Bounded runtime shutdown
    // lets a client terminate the server without first sending another newline.
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => {
            eprintln!("Diffyn could not start its asynchronous runtime.");
            return ExitCode::FAILURE;
        }
    };

    let result = runtime.block_on(server::transport::run());
    runtime.shutdown_timeout(Duration::from_millis(250));

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            // Only fixed messages leave the process. SDK errors may contain
            // received messages, so formatting those errors would leak inputs.
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
