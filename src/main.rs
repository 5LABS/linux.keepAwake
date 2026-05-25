mod config;
mod icon;
mod inhibit;
mod tray;

use std::future::pending;

use ksni::TrayMethods;
use tokio::sync::mpsc;
use tokio::time::{sleep_until, Duration, Instant};

use inhibit::InhibitManager;
use tray::{AwakeTray, Cmd, Mode, TIMER_PRESETS};

fn status_text(mode: Mode, keep_screen_on: bool) -> String {
    let base = match mode {
        Mode::Off => return "Aus".to_owned(),
        Mode::Indefinite => "Unbegrenzt wach".to_owned(),
        Mode::Timed { secs } => {
            let label = TIMER_PRESETS
                .iter()
                .find(|&&(_, s)| s == secs)
                .map(|&(l, _)| l.to_owned())
                .unwrap_or_else(|| format!("{} Min", secs / 60));
            format!("Wach für {label}")
        }
    };
    if keep_screen_on {
        format!("{base} · Bildschirm an")
    } else {
        base
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load();
    let autostart = config::autostart_enabled();

    let mut inhibitor = InhibitManager::new().await?;

    let (tx, mut rx) = mpsc::unbounded_channel::<Cmd>();
    let handle = AwakeTray::new(cfg.keep_screen_on, autostart, tx)
        .spawn()
        .await?;

    // Deadline for the active timer, if any.
    let mut deadline: Option<Instant> = None;

    loop {
        let timer = async {
            match deadline {
                Some(d) => sleep_until(d).await,
                None => pending::<()>().await,
            }
        };

        tokio::select! {
            cmd = rx.recv() => match cmd {
                Some(Cmd::Apply { mode, keep_screen_on }) => {
                    let awake = mode.is_awake();
                    inhibitor.set(awake, awake && keep_screen_on).await;

                    deadline = match mode {
                        Mode::Timed { secs } => Some(Instant::now() + Duration::from_secs(secs)),
                        _ => None,
                    };

                    config::save(&config::Config { keep_screen_on });

                    let text = status_text(mode, keep_screen_on);
                    handle.update(|t| t.status_text = text).await;
                }
                Some(Cmd::SetAutostart(on)) => {
                    config::set_autostart(on);
                }
                Some(Cmd::Quit) | None => {
                    inhibitor.set(false, false).await;
                    handle.shutdown();
                    break;
                }
            },

            _ = timer => {
                inhibitor.set(false, false).await;
                deadline = None;
                handle.update(|t| {
                    t.mode = Mode::Off;
                    t.status_text = "Aus".to_owned();
                }).await;
            }

            _ = tokio::signal::ctrl_c() => {
                inhibitor.set(false, false).await;
                break;
            }
        }
    }

    Ok(())
}
