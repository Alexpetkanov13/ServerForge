use std::ffi::OsString;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Prevent helper processes (java.exe, installers, git via BuildTools) from
/// flashing a console window on Windows.
pub fn hide_window(cmd: &mut Command) {
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
}

pub fn capture_hidden(cmd: &mut Command) -> &mut Command {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_window(cmd);
    cmd
}

pub fn kill_tree(pid: u32) {
    if pid == 0 {
        return;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("taskkill");
        cmd.args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.status();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

pub fn process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]), true);
    sys.process(sysinfo::Pid::from_u32(pid)).is_some()
}

/// Kill the tracked PID tree and any Java process whose cwd or command line
/// belongs to this server folder. Used so Stop still works if the JVM was
/// re-parented or ServerForge restarted while the server was running.
pub fn kill_server_processes(root_pid: u32, server_dir: Option<&Path>) {
    if root_pid != 0 {
        terminate_pid(root_pid);
    }
    let Some(dir) = server_dir else {
        return;
    };
    for pid in java_pids_for_server(dir) {
        if pid != 0 && pid != std::process::id() {
            terminate_pid(pid);
        }
    }
}

fn terminate_pid(pid: u32) {
    kill_tree(pid);
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]), true);
    if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid)) {
        let _ = proc.kill();
    }
}

pub fn java_running_in_dir(server_dir: &Path) -> bool {
    !java_pids_for_server(server_dir).is_empty()
}

fn java_pids_for_server(server_dir: &Path) -> Vec<u32> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        sysinfo::ProcessRefreshKind::everything(),
    );
    sys.processes()
        .iter()
        .filter_map(|(pid, proc)| {
            if belongs_to_server(server_dir, proc) {
                Some(pid.as_u32())
            } else {
                None
            }
        })
        .collect()
}

fn belongs_to_server(server_dir: &Path, proc: &sysinfo::Process) -> bool {
    let name = proc.name().to_string_lossy().to_lowercase();
    if !name.contains("java") {
        return false;
    }
    if proc.cwd().is_some_and(|cwd| paths_related(server_dir, cwd)) {
        return true;
    }
    if proc.exe().is_some_and(|exe| paths_related(server_dir, exe)) {
        return true;
    }
    cmd_belongs(server_dir, proc.cmd())
}

fn cmd_belongs(server_dir: &Path, cmd: &[OsString]) -> bool {
    if cmd.iter().any(|arg| paths_related(server_dir, Path::new(arg))) {
        return true;
    }
    let hay = cmd
        .iter()
        .map(|s| s.to_string_lossy().replace('/', "\\").to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let needle = path_key(server_dir).join("\\");
    if needle.len() < 4 {
        return false;
    }
    hay.contains(&(needle.clone() + "\\"))
        || hay.contains(&(needle.clone() + " "))
        || hay.contains(&(needle.clone() + "\""))
        || hay.ends_with(&needle)
}

fn path_key(path: &Path) -> Vec<String> {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .filter(|s| s != "." && s != "\\" && s != "/")
        .collect()
}

fn paths_related(server_dir: &Path, other: &Path) -> bool {
    let server = path_key(server_dir);
    let other = path_key(other);
    !server.is_empty() && other.starts_with(&server)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn server_folder_does_not_match_sibling_prefix() {
        assert!(!paths_related(Path::new(r"C:\servers\foo"), Path::new(r"C:\servers\foo-2")));
        assert!(paths_related(Path::new(r"C:\servers\foo"), Path::new(r"C:\servers\foo")));
        assert!(paths_related(
            Path::new(r"C:\servers\foo"),
            Path::new(r"C:\servers\foo\plugins")
        ));
    }
}
