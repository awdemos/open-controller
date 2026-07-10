use rmcp::schemars;
use serde::Deserialize;
use sysinfo::{get_current_pid, Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProcessMode {
    List,
    Kill,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProcessArgs {
    pub mode: ProcessMode,
    #[serde(default)]
    pub process_id: Option<u32>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub signal: Option<String>,
    #[serde(default = "default_false")]
    pub force: bool,
    #[serde(default = "default_false")]
    pub confirm: bool,
}

fn default_false() -> bool {
    false
}

fn current_uid(sys: &System) -> Option<u32> {
    let pid = get_current_pid().ok()?;
    sys.process(pid)
        .and_then(|p| p.user_id())
        .map(|u| **u as u32)
}

fn parse_signal(s: &str) -> Option<Signal> {
    match s.to_lowercase().as_str() {
        "kill" | "sigkill" => Some(Signal::Kill),
        "term" | "terminate" | "sigterm" => Some(Signal::Term),
        "int" | "interrupt" | "sigint" => Some(Signal::Interrupt),
        "hup" | "sighup" => Some(Signal::Hangup),
        "quit" | "sigquit" => Some(Signal::Quit),
        "usr1" | "sigusr1" => Some(Signal::User1),
        "usr2" | "sigusr2" => Some(Signal::User2),
        _ => None,
    }
}

fn is_protected_pid(pid: Pid) -> bool {
    // Pid 0 and 1 are kernel/init; we also refuse to kill ourselves.
    pid.as_u32() <= 1
}

pub fn run_process(args: &ProcessArgs, confirm_destructive: bool) -> anyhow::Result<String> {
    let mut sys = System::new_all();

    match args.mode {
        ProcessMode::List => list_processes(&sys, args),
        ProcessMode::Kill => {
            if !confirm_destructive && !args.confirm {
                anyhow::bail!(
                    "killing a process requires --confirm-destructive or per-call confirm=true"
                );
            }
            kill_process(&mut sys, args)
        }
    }
}

fn list_processes(sys: &System, args: &ProcessArgs) -> anyhow::Result<String> {
    let name_filter = args.name.as_deref().unwrap_or("").to_lowercase();
    let user_filter = args.user.as_deref();

    let mut lines: Vec<String> = Vec::new();
    for (pid, process) in sys.processes() {
        let proc_name = process.name().to_string_lossy();
        if !name_filter.is_empty() && !proc_name.to_lowercase().contains(&name_filter) {
            continue;
        }
        let uid = process.user_id().map(|u| u.to_string()).unwrap_or_else(|| "?".to_string());
        if let Some(filter) = user_filter {
            if uid != filter {
                continue;
            }
        }
        lines.push(format!(
            "pid={} name={} uid={} mem={}",
            pid.as_u32(),
            proc_name,
            uid,
            process.memory()
        ));
    }

    lines.sort();
    Ok(lines.join("\n"))
}

fn kill_process(sys: &mut System, args: &ProcessArgs) -> anyhow::Result<String> {
    let pid_value = args
        .process_id
        .ok_or_else(|| anyhow::anyhow!("process_id is required for kill mode"))?;
    let pid = Pid::from_u32(pid_value);

    let current_pid = get_current_pid()
        .map_err(|_| anyhow::anyhow!("unable to determine current process"))?;
    if is_protected_pid(pid) || pid == current_pid {
        anyhow::bail!("cannot kill protected or current process {}", pid_value);
    }

    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::everything(),
    );

    let current_uid = current_uid(sys).ok_or_else(|| anyhow::anyhow!("unable to determine current user"))?;
    let process = sys
        .process(pid)
        .ok_or_else(|| anyhow::anyhow!("process {} not found", pid_value))?;

    let proc_uid = process.user_id().map(|u| **u as u32);
    if proc_uid != Some(current_uid) {
        anyhow::bail!(
            "process {} is owned by a different user (refusing cross-user kill)",
            pid_value
        );
    }

    let signal = if args.force {
        Signal::Kill
    } else {
        args.signal
            .as_deref()
            .and_then(parse_signal)
            .unwrap_or(Signal::Term)
    };

    match process.kill_with(signal) {
        Some(true) => Ok(format!("sent {:?} to process {}", signal, pid_value)),
        Some(false) => anyhow::bail!("failed to send {:?} to process {}", signal, pid_value),
        None => anyhow::bail!("sending {:?} to process {} is not supported", signal, pid_value),
    }
}
