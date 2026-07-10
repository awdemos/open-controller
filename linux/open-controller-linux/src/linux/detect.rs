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

    #[test]
    fn detects_wayland_when_wayland_display_set() {
        let prev = env::var("WAYLAND_DISPLAY").ok();
        let prev_display = env::var("DISPLAY").ok();
        unsafe {
            env::set_var("WAYLAND_DISPLAY", "wayland-1");
            env::remove_var("DISPLAY");
        }
        assert_eq!(detect(), Compositor::Wayland);
        unsafe {
            match prev {
                Some(v) => env::set_var("WAYLAND_DISPLAY", v),
                None => env::remove_var("WAYLAND_DISPLAY"),
            }
            match prev_display {
                Some(v) => env::set_var("DISPLAY", v),
                None => {}
            }
        }
    }

    #[test]
    fn detects_x11_when_display_set() {
        let prev = env::var("DISPLAY").ok();
        let prev_wayland = env::var("WAYLAND_DISPLAY").ok();
        unsafe {
            env::remove_var("WAYLAND_DISPLAY");
            env::set_var("DISPLAY", ":0");
        }
        assert_eq!(detect(), Compositor::X11);
        unsafe {
            match prev {
                Some(v) => env::set_var("DISPLAY", v),
                None => env::remove_var("DISPLAY"),
            }
            match prev_wayland {
                Some(v) => env::set_var("WAYLAND_DISPLAY", v),
                None => {}
            }
        }
    }

    #[test]
    fn detects_headless_when_nothing_set() {
        let prev = env::var("DISPLAY").ok();
        let prev_wayland = env::var("WAYLAND_DISPLAY").ok();
        unsafe {
            env::remove_var("DISPLAY");
            env::remove_var("WAYLAND_DISPLAY");
        }
        assert_eq!(detect(), Compositor::Headless);
        unsafe {
            match prev {
                Some(v) => env::set_var("DISPLAY", v),
                None => {}
            }
            match prev_wayland {
                Some(v) => env::set_var("WAYLAND_DISPLAY", v),
                None => {}
            }
        }
    }
}
