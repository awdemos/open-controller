use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, Screen, StackMode, Window};
use x11rb::rust_connection::RustConnection;

pub struct WindowInfo {
    pub id: u32,
    pub title: String,
    pub class: String,
}

pub fn list_windows() -> anyhow::Result<Vec<WindowInfo>> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;
    let tree = conn.query_tree(root)?.reply()?;

    let mut windows = Vec::new();
    for &child in &tree.children {
        if let Some(info) = get_window_info(&conn, screen, child)? {
            windows.push(info);
        }
    }
    Ok(windows)
}

pub fn raise_window(name: &str) -> anyhow::Result<()> {
    let win = find_window(name)?;
    let (conn, _screen_num) = RustConnection::connect(None)?;
    let values = x11rb::protocol::xproto::ConfigureWindowAux::new().stack_mode(StackMode::ABOVE);
    conn.configure_window(win.id, &values)?.check()?;
    conn.set_input_focus(
        x11rb::protocol::xproto::InputFocus::POINTER_ROOT,
        win.id,
        CURRENT_TIME,
    )?
    .check()?;
    Ok(())
}

pub fn close_window(name: &str) -> anyhow::Result<()> {
    let win = find_window(name)?;
    let (conn, _screen_num) = RustConnection::connect(None)?;
    conn.kill_client(win.id)?.check()?;
    Ok(())
}

fn find_window(name: &str) -> anyhow::Result<WindowInfo> {
    let lowered = name.to_lowercase();
    let windows = list_windows()?;
    windows
        .into_iter()
        .find(|w| {
            w.title.to_lowercase().contains(&lowered) || w.class.to_lowercase().contains(&lowered)
        })
        .ok_or_else(|| anyhow::anyhow!("no window matching '{}' found", name))
}

fn get_window_info(
    conn: &RustConnection,
    screen: &Screen,
    win: Window,
) -> anyhow::Result<Option<WindowInfo>> {
    let title = net_wm_name(conn, screen, win)
        .ok()
        .flatten()
        .or_else(|| wm_name(conn, screen, win).ok().flatten())
        .unwrap_or_default();
    let class = wm_class(conn, screen, win)
        .ok()
        .flatten()
        .unwrap_or_default();
    if title.is_empty() && class.is_empty() {
        return Ok(None);
    }
    Ok(Some(WindowInfo {
        id: win,
        title,
        class,
    }))
}

fn net_wm_name(
    conn: &RustConnection,
    _screen: &Screen,
    win: Window,
) -> anyhow::Result<Option<String>> {
    let atom = conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom;
    let utf8 = conn.intern_atom(false, b"UTF8_STRING")?.reply()?.atom;
    let reply = conn
        .get_property(false, win, atom, utf8, 0, 1024)?
        .reply()?;
    if reply.value.is_empty() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&reply.value).to_string()))
}

fn wm_name(conn: &RustConnection, _screen: &Screen, win: Window) -> anyhow::Result<Option<String>> {
    let reply = conn
        .get_property(
            false,
            win,
            x11rb::protocol::xproto::Atom::from(x11rb::protocol::xproto::AtomEnum::WM_NAME),
            x11rb::protocol::xproto::Atom::from(x11rb::protocol::xproto::AtomEnum::STRING),
            0,
            1024,
        )?
        .reply()?;
    if reply.value.is_empty() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&reply.value).to_string()))
}

fn wm_class(
    conn: &RustConnection,
    _screen: &Screen,
    win: Window,
) -> anyhow::Result<Option<String>> {
    let reply = conn
        .get_property(
            false,
            win,
            x11rb::protocol::xproto::Atom::from(x11rb::protocol::xproto::AtomEnum::WM_CLASS),
            x11rb::protocol::xproto::Atom::from(x11rb::protocol::xproto::AtomEnum::STRING),
            0,
            1024,
        )?
        .reply()?;
    if reply.value.is_empty() {
        return Ok(None);
    }
    let s = String::from_utf8_lossy(&reply.value);
    // WM_CLASS contains two null-terminated strings: instance and class.
    let class = s.split('\0').nth(1).unwrap_or(&s).to_string();
    Ok(Some(class))
}
