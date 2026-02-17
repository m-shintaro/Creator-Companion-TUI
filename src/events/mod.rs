use crate::app::action::Action;
use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

pub fn spawn_event_loop(action_tx: mpsc::UnboundedSender<Action>) {
    tokio::spawn(async move {
        if let Err(err) = event_loop(action_tx.clone()).await {
            let _ = action_tx.send(Action::TaskOutput {
                task_id: 0,
                stream: crate::app::action::OutputStream::Stderr,
                line: format!("event loop error: {err}"),
            });
        }
    });
}

async fn event_loop(action_tx: mpsc::UnboundedSender<Action>) -> Result<()> {
    let mut reader = EventStream::new();
    let mut tick = time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = tick.tick() => {
                if action_tx.send(Action::Tick).is_err() {
                    break;
                }
            }
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(CrosstermEvent::Key(key))) => {
                        if action_tx.send(Action::Key(key)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        let _ = action_tx.send(Action::TaskOutput {
                            task_id: 0,
                            stream: crate::app::action::OutputStream::Stderr,
                            line: format!("input error: {err}")
                        });
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}
