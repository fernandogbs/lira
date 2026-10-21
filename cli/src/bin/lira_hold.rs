//! Opt-in evdev hold helper: watches a single key and calls `lira start`/`stop`.
//!
//! Requires the user in the `input` group and `hotkey.evdev_acknowledged = true`.

use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process::Command as ProcCommand;
use std::time::Duration;

use evdev::{Device, EventSummary, KeyCode};
use lira_core::settings::{HoldBackend, Settings};
use nix::fcntl::{fcntl, FcntlArg, OFlag};

fn main() {
    let settings = Settings::load().unwrap_or_default();
    let locale = settings.ui.locale.resolve();

    if settings.hotkey.hold_backend != HoldBackend::Evdev {
        eprintln!(
            "lira-hold requires [hotkey] hold_backend = \"evdev\" in config.toml (current: {:?})",
            settings.hotkey.hold_backend
        );
        std::process::exit(2);
    }
    if !settings.hotkey.evdev_acknowledged {
        eprintln!("{}", locale.catalog().evdev_not_acknowledged);
        eprintln!("{}", locale.catalog().evdev_warning);
        std::process::exit(2);
    }

    eprintln!("{}", locale.catalog().evdev_warning);

    let key = parse_key(&settings.hotkey.hold_key).unwrap_or_else(|| {
        eprintln!(
            "unknown hold_key {:?}; use a Linux key name like KEY_SPACE",
            settings.hotkey.hold_key
        );
        std::process::exit(2);
    });

    let lira_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("lira")))
        .filter(|p| p.exists())
        .unwrap_or_else(|| Path::new("lira").to_path_buf());

    let mut devices = open_keyboards();
    if devices.is_empty() {
        eprintln!("no readable keyboard devices under /dev/input (are you in the input group?)");
        std::process::exit(1);
    }

    eprintln!(
        "lira-hold watching {:?} on {} device(s); Ctrl+C to quit",
        settings.hotkey.hold_key,
        devices.len()
    );

    let mut pressed = false;
    loop {
        let mut saw_event = false;
        for device in &mut devices {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        saw_event = true;
                        if let EventSummary::Key(_, code, value) = event.destructure() {
                            if code != key {
                                continue;
                            }
                            match value {
                                1 if !pressed => {
                                    pressed = true;
                                    let _ = ProcCommand::new(&lira_bin).arg("start").status();
                                }
                                0 if pressed => {
                                    pressed = false;
                                    let _ = ProcCommand::new(&lira_bin).arg("stop").status();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(err) => {
                    eprintln!("evdev read error: {err}");
                }
            }
        }
        if !saw_event {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn parse_key(name: &str) -> Option<KeyCode> {
    let upper = name.trim().to_uppercase();
    let name = if upper.starts_with("KEY_") {
        upper
    } else {
        format!("KEY_{upper}")
    };
    match name.as_str() {
        "KEY_SPACE" => Some(KeyCode::KEY_SPACE),
        "KEY_LEFTCTRL" | "KEY_LEFTCONTROL" => Some(KeyCode::KEY_LEFTCTRL),
        "KEY_RIGHTCTRL" | "KEY_RIGHTCONTROL" => Some(KeyCode::KEY_RIGHTCTRL),
        "KEY_LEFTALT" => Some(KeyCode::KEY_LEFTALT),
        "KEY_RIGHTALT" => Some(KeyCode::KEY_RIGHTALT),
        "KEY_LEFTMETA" | "KEY_LEFTSUPER" => Some(KeyCode::KEY_LEFTMETA),
        "KEY_RIGHTMETA" | "KEY_RIGHTSUPER" => Some(KeyCode::KEY_RIGHTMETA),
        "KEY_CAPSLOCK" => Some(KeyCode::KEY_CAPSLOCK),
        "KEY_F13" => Some(KeyCode::KEY_F13),
        "KEY_F14" => Some(KeyCode::KEY_F14),
        "KEY_F15" => Some(KeyCode::KEY_F15),
        _ => None,
    }
}

fn open_keyboards() -> Vec<Device> {
    let Ok(entries) = std::fs::read_dir("/dev/input") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with("event") {
            continue;
        }
        let Ok(device) = Device::open(&path) else {
            continue;
        };
        if device.supported_keys().is_some() {
            let fd = device.as_raw_fd();
            let flags = fcntl(fd, FcntlArg::F_GETFL).unwrap_or(0);
            let _ = fcntl(
                fd,
                FcntlArg::F_SETFL(OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK),
            );
            out.push(device);
        }
    }
    out
}
