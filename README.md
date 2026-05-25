# Keep Awake

Ein minimalistisches Tray-Tool für Linux, das verhindert, dass dein System in den
Ruhezustand wechselt oder der Bildschirm abschaltet – inspiriert von
**Microsoft PowerToys „Keep Awake"**.

Das Tool lebt ausschließlich in der Tray (kein Fenster) und wird komplett über das
Tray-Menü gesteuert. Das Icon ist eine Kaffeetasse: **grau** wenn inaktiv, **grün**
sobald das System wach gehalten wird.

## Funktionen

- **Aus** – das System verhält sich normal, alle Sperren werden freigegeben
- **Unbegrenzt wach** – hält das System wach, bis du es wieder ausschaltest
- **Timer** – wach für eine feste Dauer (30 Min / 1 Std / 2 Std / 4 Std), danach
  automatisch zurück auf „Aus"
- **Bildschirm anlassen** – verhindert zusätzlich das Abdunkeln/Sperren des Bildschirms
- **Beim Login starten** – Autostart-Eintrag direkt aus dem Menü an-/abschalten
- **Beenden**

Die zuletzt gewählte Einstellung von „Bildschirm anlassen" wird gespeichert und beim
nächsten Start wiederhergestellt.

## Voraussetzungen

- Linux mit einer Desktop-Umgebung, die das **StatusNotifierItem**-Protokoll
  unterstützt (z. B. Ubuntu/GNOME mit aktiver AppIndicator-Erweiterung, KDE Plasma)
- **D-Bus** mit `org.freedesktop.login1` (systemd-logind) und `org.freedesktop.ScreenSaver`
- Zum Bauen: **Rust** (Stable, mit `cargo`)

Entwickelt und getestet unter **Ubuntu 26.04 LTS (GNOME/Wayland)**.

## Installation

Das mitgelieferte Skript baut die Release-Binary, installiert sie und richtet
Autostart sowie einen App-Eintrag ein:

```bash
./install.sh
```

Danach läuft das Tool nach dem nächsten Login automatisch. Sofort starten:

```bash
~/.local/bin/keep-awake &
```

Das Skript installiert nach:

| Pfad | Zweck |
|------|-------|
| `~/.local/bin/keep-awake` | die Binary |
| `~/.local/share/applications/keep-awake.desktop` | App-Eintrag (App-Menü) |
| `~/.config/autostart/keep-awake.desktop` | Autostart beim Login |

Konfiguration wird in `~/.config/keep-awake/config.toml` gespeichert.

## Manuell bauen

```bash
cargo build --release
./target/release/keep-awake
```

## Deinstallation

```bash
pkill -x keep-awake
rm -f ~/.local/bin/keep-awake \
      ~/.local/share/applications/keep-awake.desktop \
      ~/.config/autostart/keep-awake.desktop
rm -rf ~/.config/keep-awake
```

## Wie es funktioniert

Keep Awake nimmt keine dauerhaften Systemeinstellungen vor, sondern setzt
**D-Bus-Inhibitor-Sperren**, die nur so lange gelten, wie das Tool sie hält:

- **System wach halten:** `org.freedesktop.login1` →
  `Inhibit("idle", …, "block")`. Es wird nur `idle` blockiert (nicht `sleep`), damit
  manuelles Suspendieren weiterhin möglich bleibt – unterdrückt wird ausschließlich
  das *automatische* Einschlafen bei Inaktivität. Die Sperre ist ein Datei-Deskriptor;
  wird er geschlossen, ist die Sperre sofort frei.
- **Bildschirm anlassen:** `org.freedesktop.ScreenSaver` → `Inhibit(…)` verhindert
  Abdunkeln, Bildschirmschoner und Sperre. Freigabe über `UnInhibit(cookie)`.

Beim Beenden (oder Prozessende) werden alle Sperren automatisch freigegeben.

## Projektstruktur

```
linux.keepAwake/
├─ Cargo.toml
├─ install.sh          # Build + Installation + Autostart
└─ src/
   ├─ main.rs          # tokio-Runtime, Controller-Loop, Timer
   ├─ tray.rs          # Tray-Icon, Menü, State-Modell
   ├─ inhibit.rs       # D-Bus-Inhibitoren (login1 + ScreenSaver)
   ├─ icon.rs          # prozedural gezeichnetes Kaffeetassen-Icon
   └─ config.rs        # Konfiguration + Autostart-Eintrag
```

Technik: **pures Rust** (kein Webview) mit `ksni` (Tray), `zbus` (D-Bus) und
`tokio` (Async-Runtime/Timer).
