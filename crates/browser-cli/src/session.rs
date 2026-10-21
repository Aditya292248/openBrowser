use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, OpenOptions};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInfo {
    pub port: u16,
    pub token: String,
}

impl DaemonInfo {
    pub fn address(&self) -> SocketAddrV4 {
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, self.port)
    }
}

pub struct SessionGuard {
    path: PathBuf,
    token: String,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        if let Ok(bytes) = fs::read(&self.path) {
            if let Ok(current) = serde_json::from_slice::<DaemonInfo>(&bytes) {
                if current.token == self.token {
                    let _ = fs::remove_file(&self.path);
                }
            }
        }
    }
}

pub fn read() -> Result<DaemonInfo, String> {
    let path = session_path()?;
    let bytes = fs::read(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "daemon is not running; run `my_browser daemon start`".to_string()
        } else {
            format!("cannot read daemon state at {}: {error}", path.display())
        }
    })?;
    let info: DaemonInfo = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid daemon state at {}: {error}", path.display()))?;
    if info.port == 0 || info.token.is_empty() {
        return Err(format!("invalid daemon state at {}", path.display()));
    }
    Ok(info)
}

pub fn publish(info: &DaemonInfo) -> Result<SessionGuard, String> {
    let path = session_path()?;
    let directory = path.parent().expect("session path has a parent");
    fs::create_dir_all(directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
            .map_err(|error| format!("cannot protect {}: {error}", directory.display()))?;
    }

    if path.exists() {
        if let Ok(existing) = read() {
            if TcpStream::connect_timeout(&existing.address().into(), Duration::from_millis(300))
                .is_ok()
            {
                return Err("daemon is already running".to_string());
            }
        }
        fs::remove_file(&path)
            .map_err(|error| format!("cannot replace stale daemon state: {error}"))?;
    }

    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("cannot create {}: {error}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("cannot protect {}: {error}", path.display()))?;
    }

    serde_json::to_writer(file, info)
        .map_err(|error| format!("cannot write daemon state: {error}"))?;
    Ok(SessionGuard {
        path,
        token: info.token.clone(),
    })
}

fn session_path() -> Result<PathBuf, String> {
    let base = if let Some(path) = env::var_os("MY_BROWSER_RUNTIME_DIR") {
        PathBuf::from(path)
    } else if cfg!(windows) {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .ok_or("LOCALAPPDATA is not set")?
    } else if let Some(path) = env::var_os("XDG_RUNTIME_DIR") {
        PathBuf::from(path)
    } else {
        env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(".cache"))
            .ok_or("HOME is not set")?
    };
    Ok(base.join("my_browser").join("session.json"))
}
