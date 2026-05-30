use ksni::menu::{CheckmarkItem, MenuItem, StandardItem, SubMenu};
use ksni::{Icon, Status, ToolTip, Tray};
use tokio::sync::mpsc::UnboundedSender;

use crate::config::SavedMode;
use crate::icon;

/// The selected operating mode. Timer presets are durations in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Off,
    Indefinite,
    Timed { secs: u64 },
}

impl Mode {
    pub fn is_awake(&self) -> bool {
        !matches!(self, Mode::Off)
    }
}

/// Commands sent from tray menu callbacks to the async controller.
pub enum Cmd {
    Apply { mode: Mode, keep_screen_on: bool },
    SetAutostart(bool),
    Quit,
}

pub const TIMER_PRESETS: &[(&str, u64)] = &[
    ("30 Minuten", 30 * 60),
    ("1 Stunde", 60 * 60),
    ("2 Stunden", 2 * 60 * 60),
    ("4 Stunden", 4 * 60 * 60),
];

pub struct AwakeTray {
    pub mode: Mode,
    pub keep_screen_on: bool,
    pub autostart: bool,
    pub status_text: String,
    pub tx: UnboundedSender<Cmd>,
}

impl From<Mode> for SavedMode {
    fn from(m: Mode) -> Self {
        match m {
            Mode::Off => SavedMode::Off,
            Mode::Indefinite => SavedMode::Indefinite,
            Mode::Timed { secs } => SavedMode::Timed { secs },
        }
    }
}

impl From<SavedMode> for Mode {
    fn from(m: SavedMode) -> Self {
        match m {
            SavedMode::Off => Mode::Off,
            SavedMode::Indefinite => Mode::Indefinite,
            SavedMode::Timed { secs } => Mode::Timed { secs },
        }
    }
}

impl AwakeTray {
    pub fn new(
        keep_screen_on: bool,
        mode: Mode,
        autostart: bool,
        status_text: String,
        tx: UnboundedSender<Cmd>,
    ) -> Self {
        Self {
            mode,
            keep_screen_on,
            autostart,
            status_text,
            tx,
        }
    }

    fn apply(&self) {
        let _ = self.tx.send(Cmd::Apply {
            mode: self.mode,
            keep_screen_on: self.keep_screen_on,
        });
    }
}

impl Tray for AwakeTray {
    fn id(&self) -> String {
        "keep-awake".into()
    }

    fn title(&self) -> String {
        "Keep Awake".into()
    }

    fn status(&self) -> Status {
        Status::Active
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        icon::render(self.mode.is_awake())
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "Keep Awake".into(),
            description: self.status_text.clone(),
            icon_pixmap: icon::render(self.mode.is_awake()),
            icon_name: String::new(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let current = self.mode;

        let timer_items: Vec<MenuItem<Self>> = TIMER_PRESETS
            .iter()
            .map(|&(label, secs)| {
                CheckmarkItem {
                    label: label.into(),
                    checked: matches!(current, Mode::Timed { secs: s } if s == secs),
                    activate: Box::new(move |t: &mut Self| {
                        t.mode = Mode::Timed { secs };
                        t.apply();
                    }),
                    ..Default::default()
                }
                .into()
            })
            .collect();

        vec![
            CheckmarkItem {
                label: "Aus".into(),
                checked: matches!(current, Mode::Off),
                activate: Box::new(|t: &mut Self| {
                    t.mode = Mode::Off;
                    t.apply();
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: "Unbegrenzt wach".into(),
                checked: matches!(current, Mode::Indefinite),
                activate: Box::new(|t: &mut Self| {
                    t.mode = Mode::Indefinite;
                    t.apply();
                }),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Timer".into(),
                submenu: timer_items,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            CheckmarkItem {
                label: "Bildschirm anlassen".into(),
                checked: self.keep_screen_on,
                activate: Box::new(|t: &mut Self| {
                    t.keep_screen_on = !t.keep_screen_on;
                    t.apply();
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: "Beim Login starten".into(),
                checked: self.autostart,
                activate: Box::new(|t: &mut Self| {
                    t.autostart = !t.autostart;
                    let _ = t.tx.send(Cmd::SetAutostart(t.autostart));
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Beenden".into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.tx.send(Cmd::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
