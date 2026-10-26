//! Bounded, local standard-input/output transport and process lifecycle.

use std::{io, time::Duration};

use rmcp::{
    RoleServer, ServiceExt,
    service::{QuitReason, ServerInitializeError},
    transport::{async_rw::AsyncRwTransport, stdio_pipes},
};
use tokio_util::sync::CancellationToken;

use super::DiffynServer;

/// Maximum bytes in one incoming JSON-RPC line, including its newline.
///
/// The SDK enforces this while reading, before an arbitrarily long line can
/// accumulate in memory. It discards an oversized line and accepts later lines.
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

// The SDK allows two seconds to drain responses when cancelled. This outer
// limit gives cleanup a little more time while keeping process exit bounded.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(3);

/// Serve until the client closes input or the process receives a stop signal.
///
/// Errors are deliberately fixed strings. In particular, do not install a
/// tracing subscriber here: SDK debug events can include full malformed inputs.
/// No environment-controlled logging, file storage, or network transport exists.
pub async fn run() -> Result<(), &'static str> {
    let cancellation = CancellationToken::new();
    let stop_signal = shutdown_signal();
    tokio::pin!(stop_signal);

    // Linux uses reactor-backed pipes where available. Other platforms retain
    // the SDK's portable fallback; main bounds the blocking pool's shutdown.
    let (reader, writer) = stdio_pipes();
    let transport = AsyncRwTransport::<RoleServer, _, _>::new(reader, writer)
        .with_max_line_length(MAX_MESSAGE_BYTES);
    let starting = DiffynServer::default().serve_with_ct(transport, cancellation.clone());
    tokio::pin!(starting);

    let service = tokio::select! {
        // Register signal handlers before polling a potentially idle client.
        biased;
        signal = &mut stop_signal => {
            cancellation.cancel();
            return signal.map_err(|_| "Diffyn could not monitor process stop signals.");
        }
        result = &mut starting => match result {
            Ok(service) => service,
            // Closing stdin before the first request is a normal client exit.
            Err(ServerInitializeError::ConnectionClosed(_)) => return Ok(()),
            Err(ServerInitializeError::Cancelled) => return Ok(()),
            Err(_) => return Err("Diffyn could not establish the MCP connection."),
        },
    };

    let finishing = service.waiting();
    tokio::pin!(finishing);

    tokio::select! {
        result = &mut finishing => finish_result(result),
        signal = &mut stop_signal => {
            // Cancel even if signal registration failed, then await cleanup
            // instead of abandoning an active transport in the background.
            cancellation.cancel();
            let result = tokio::time::timeout(SHUTDOWN_GRACE, &mut finishing)
                .await
                .map_err(|_| "Diffyn exceeded its graceful shutdown deadline.")?;
            finish_result(result)?;
            signal.map_err(|_| "Diffyn could not monitor process stop signals.")
        }
    }
}

fn finish_result(result: Result<QuitReason, tokio::task::JoinError>) -> Result<(), &'static str> {
    match result {
        Ok(QuitReason::Closed | QuitReason::Cancelled) => Ok(()),
        // QuitReason is non-exhaustive. Unexpected lifecycle outcomes must be
        // surfaced without formatting potentially input-bearing error values.
        _ => Err("Diffyn's MCP service stopped unexpectedly."),
    }
}

#[cfg(unix)]
async fn shutdown_signal() -> io::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;
    let received = tokio::select! {
        received = interrupt.recv() => received,
        received = terminate.recv() => received,
    };

    received.ok_or_else(|| io::Error::other("process signal stream closed"))
}

#[cfg(not(unix))]
async fn shutdown_signal() -> io::Result<()> {
    tokio::signal::ctrl_c().await
}
