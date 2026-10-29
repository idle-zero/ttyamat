use std::{
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};
use ttyamat_terminal::{TerminalEvent, TerminalSession, TerminalSize};

#[cfg(windows)]
fn marker_command(prefix: &str) -> Vec<u8> {
    format!("Write-Output ('{prefix}_' + 'DONE')\r").into_bytes()
}

#[cfg(unix)]
fn marker_command(prefix: &str) -> Vec<u8> {
    format!("printf '%s%s\\n' '{prefix}_' 'DONE'\n").into_bytes()
}

fn wait_for_output(
    session: &TerminalSession,
    events: &Receiver<TerminalEvent>,
    marker: &str,
) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut frame = session.initial_frame();

    loop {
        // Acknowledge before capture so later output can notify us again.
        session.acknowledge_wakeup();
        if let Some(update) = session.take_render_update() {
            frame.apply(update);
        }

        if frame.rows().iter().any(|row| row.contains(marker)) {
            return true;
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() || events.recv_timeout(remaining).is_err() {
            return false;
        }
    }
}

#[test]
fn empty_write_does_not_block_subsequent_input() {
    let size = TerminalSize::new(80, 24, 8, 16).unwrap();
    let (sender, events) = mpsc::channel();
    let mut session = TerminalSession::spawn_default(size, move |event| {
        let _ = sender.send(event);
    })
    .expect("failed to start the default shell");

    session.write(marker_command("BEFORE")).unwrap();
    assert!(
        wait_for_output(&session, &events, "BEFORE_DONE"),
        "shell was not processing commands before the empty write"
    );

    session.write(Vec::new()).unwrap();
    session.write(marker_command("AFTER")).unwrap();

    let output_progressed = wait_for_output(&session, &events, "AFTER_DONE");
    let shutdown = session.shutdown();

    assert!(shutdown.is_ok(), "worker failed: {shutdown:?}");
    assert!(
        output_progressed,
        "input stopped progressing after the empty write"
    );
}
