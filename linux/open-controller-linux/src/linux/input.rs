use crate::linux::detect::{Compositor, detect};
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use std::time::Duration;

pub struct InputBackend {
    enigo: Enigo,
}

impl InputBackend {
    pub fn try_new() -> anyhow::Result<Self> {
        match detect() {
            Compositor::X11 => {
                let enigo = Enigo::new(&Settings::default())
                    .map_err(|e| anyhow::anyhow!("failed to initialize enigo: {}", e))?;
                Ok(Self { enigo })
            }
            Compositor::Wayland => anyhow::bail!(
                "native Wayland input requires a desktop portal; run under XWayland or use an X11 session"
            ),
            Compositor::Headless => anyhow::bail!("no display detected; input requires X11 or Wayland"),
        }
    }

    pub fn move_mouse(&mut self, x: i32, y: i32, relative: bool) -> anyhow::Result<()> {
        let coord = if relative { Coordinate::Rel } else { Coordinate::Abs };
        self.enigo.move_mouse(x, y, coord)?;
        Ok(())
    }

    pub fn click(&mut self, button: i32, x: i32, y: i32) -> anyhow::Result<()> {
        let btn = match button {
            0 => Button::Left,
            1 => Button::Middle,
            2 => Button::Right,
            _ => anyhow::bail!("unsupported button {}", button),
        };
        self.enigo.move_mouse(x, y, Coordinate::Abs)?;
        std::thread::sleep(Duration::from_millis(10));
        self.enigo.button(btn, Direction::Click)?;
        Ok(())
    }

    pub fn scroll(
        &mut self,
        direction: i32,
        x: i32,
        y: i32,
        amount: i32,
    ) -> anyhow::Result<()> {
        let (axis, sign) = match direction {
            0 => (Axis::Vertical, -1),   // up
            1 => (Axis::Vertical, 1),    // down
            2 => (Axis::Horizontal, -1), // left
            3 => (Axis::Horizontal, 1),   // right
            _ => anyhow::bail!("unsupported scroll direction {}", direction),
        };
        let amount = amount.max(1);
        self.enigo.move_mouse(x, y, Coordinate::Abs)?;
        std::thread::sleep(Duration::from_millis(10));
        self.enigo.scroll(sign * amount, axis)?;
        Ok(())
    }

    pub fn type_text(&mut self, text: &str) -> anyhow::Result<()> {
        self.enigo.text(text)?;
        Ok(())
    }

    pub fn shortcut(&mut self, keys: &str) -> anyhow::Result<()> {
        let tokens: Vec<String> = keys
            .split(['+', '-'])
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        if tokens.is_empty() {
            anyhow::bail!("empty shortcut");
        }
        let parsed: Vec<Key> = tokens
            .iter()
            .map(|t| parse_key(t))
            .collect::<Result<Vec<_>, _>>()?;
        for key in &parsed {
            self.enigo.key(*key, Direction::Press)?;
        }
        let last = *parsed.last().unwrap();
        if !is_modifier(&last) {
            self.enigo.key(last, Direction::Click)?;
        }
        for key in parsed.iter().rev() {
            self.enigo.key(*key, Direction::Release)?;
        }
        Ok(())
    }
}

fn parse_key(token: &str) -> anyhow::Result<Key> {
    Ok(match token {
        "ctrl" | "control" => Key::Control,
        "alt" => Key::Alt,
        "shift" => Key::Shift,
        "meta" | "win" | "command" | "super" => Key::Meta,
        "enter" | "return" => Key::Return,
        "space" => Key::Space,
        "tab" => Key::Tab,
        "esc" | "escape" => Key::Escape,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        s if s.len() == 1 => Key::Unicode(s.chars().next().unwrap()),
        _ => anyhow::bail!("unknown key '{}'", token),
    })
}

fn is_modifier(key: &Key) -> bool {
    matches!(key, Key::Control | Key::Alt | Key::Shift | Key::Meta)
}
