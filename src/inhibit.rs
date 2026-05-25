use zbus::zvariant::OwnedFd;
use zbus::{proxy, Connection};

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait LoginManager {
    fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> zbus::Result<OwnedFd>;
}

#[proxy(
    interface = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/org/freedesktop/ScreenSaver"
)]
trait ScreenSaver {
    fn inhibit(&self, application_name: &str, reason_for_inhibit: &str) -> zbus::Result<u32>;
    fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
}

/// Holds the active D-Bus inhibitor locks.
///
/// - `idle_lock`: an `OwnedFd` from logind's `Inhibit("idle", ..., "block")`.
///   Keeping the fd open holds the lock; dropping it releases. We only block
///   `idle` (not `sleep`) so the user can still suspend manually.
/// - `screensaver_cookie`: cookie from `org.freedesktop.ScreenSaver.Inhibit`,
///   released via `UnInhibit`.
pub struct InhibitManager {
    system: Connection,
    session: Connection,
    idle_lock: Option<OwnedFd>,
    screensaver_cookie: Option<u32>,
}

impl InhibitManager {
    pub async fn new() -> zbus::Result<Self> {
        Ok(Self {
            system: Connection::system().await?,
            session: Connection::session().await?,
            idle_lock: None,
            screensaver_cookie: None,
        })
    }

    /// Apply the desired inhibitor state. `awake` keeps the system from
    /// auto-suspending; `screen` additionally keeps the display from blanking.
    pub async fn set(&mut self, awake: bool, screen: bool) {
        if let Err(e) = self.set_system_awake(awake).await {
            eprintln!("keep-awake: login1 inhibit failed: {e}");
        }
        if let Err(e) = self.set_screen_on(screen).await {
            eprintln!("keep-awake: screensaver inhibit failed: {e}");
        }
    }

    async fn set_system_awake(&mut self, on: bool) -> zbus::Result<()> {
        match (on, self.idle_lock.is_some()) {
            (true, false) => {
                let proxy = LoginManagerProxy::new(&self.system).await?;
                let fd = proxy
                    .inhibit("idle", "Keep Awake", "User requested wakefulness", "block")
                    .await?;
                self.idle_lock = Some(fd);
            }
            (false, true) => {
                self.idle_lock = None; // drop closes the fd -> releases the lock
            }
            _ => {}
        }
        Ok(())
    }

    async fn set_screen_on(&mut self, on: bool) -> zbus::Result<()> {
        match (on, self.screensaver_cookie) {
            (true, None) => {
                let proxy = ScreenSaverProxy::new(&self.session).await?;
                let cookie = proxy.inhibit("Keep Awake", "User requested screen on").await?;
                self.screensaver_cookie = Some(cookie);
            }
            (false, Some(cookie)) => {
                let proxy = ScreenSaverProxy::new(&self.session).await?;
                proxy.un_inhibit(cookie).await?;
                self.screensaver_cookie = None;
            }
            _ => {}
        }
        Ok(())
    }
}
