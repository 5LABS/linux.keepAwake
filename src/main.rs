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

use config::SavedMode;

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
    let initial_mode = Mode::from(cfg.mode);

    let mut inhibitor = {
        let mut attempts = 0u32;
        loop {
            match InhibitManager::new().await {
                Ok(m) => break m,
                Err(e) if attempts < 10 => {
                    eprintln!("keep-awake: D-Bus not ready ({e}), retrying…");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    attempts += 1;
                }
                Err(e) => return Err(e.into()),
            }
        }
    };

    let awake = initial_mode.is_awake();
    inhibitor.set(awake, awake && cfg.keep_screen_on).await;

    let mut deadline: Option<Instant> = match initial_mode {
        Mode::Timed { secs } => Some(Instant::now() + Duration::from_secs(secs)),
        _ => None,
    };

    let initial_status = status_text(initial_mode, cfg.keep_screen_on);
    let (tx, mut rx) = mpsc::unbounded_channel::<Cmd>();
    let handle = {
        let mut attempts = 0u32;
        loop {
            match AwakeTray::new(
                cfg.keep_screen_on,
                initial_mode,
                autostart,
                initial_status.clone(),
                tx.clone(),
            )
            .spawn()
            .await
            {
                Ok(h) => break h,
                Err(e) if attempts < 10 => {
                    eprintln!("keep-awake: tray not ready ({e}), retrying…");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    attempts += 1;
                }
                Err(e) => return Err(e.into()),
            }
        }
    };

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

                    config::save(&config::Config { keep_screen_on, mode: SavedMode::from(mode) });

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
