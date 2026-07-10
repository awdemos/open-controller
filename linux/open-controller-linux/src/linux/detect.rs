use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compositor {
    X11,
    Wayland,
    Headless,
}

pub fn detect() -> Compositor {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return Compositor::Wayland;
    }
    if env::var("DISPLAY").is_ok() {
        return Compositor::X11;
    }
    Compositor::Headless
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn restore_env(key: &str, prev: Option<String>) {
        unsafe {
            match prev {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
    }

    #[test]
    fn detects_wayland_when_wayland_display_set() {
        let _g = ENV_LOCK.lock().unwrap();
        let prev = env::var("WAYLAND_DISPLAY").ok();
        let prev_display = env::var("DISPLAY").ok();
        unsafe {
            env::set_var("WAYLAND_DISPLAY", "wayland-1");
            env::remove_var("DISPLAY");
        }
        assert_eq!(detect(), Compositor::Wayland);
        restore_env("WAYLAND_DISPLAY", prev);
        restore_env("DISPLAY", prev_display);
    }

    #[test]
    fn detects_x11_when_display_set() {
        let _g = ENV_LOCK.lock().unwrap();
        let prev = env::var("DISPLAY").ok();
        let prev_wayland = env::var("WAYLAND_DISPLAY").ok();
        unsafe {
            env::remove_var("WAYLAND_DISPLAY");
            env::set_var("DISPLAY", ":0");
        }
        assert_eq!(detect(), Compositor::X11);
        restore_env("DISPLAY", prev);
        restore_env("WAYLAND_DISPLAY", prev_wayland);
    }

    #[test]
    fn detects_headless_when_nothing_set() {
        let _g = ENV_LOCK.lock().unwrap();
        let prev = env::var("DISPLAY").ok();
        let prev_wayland = env::var("WAYLAND_DISPLAY").ok();
        unsafe {
            env::remove_var("DISPLAY");
            env::remove_var("WAYLAND_DISPLAY");
        }
        assert_eq!(detect(), Compositor::Headless);
        restore_env("DISPLAY", prev);
        restore_env("WAYLAND_DISPLAY", prev_wayland);
    }
}
