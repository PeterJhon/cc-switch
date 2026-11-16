//! OpenCharm is an isolated Windows-only extension for CC Switch.
//! This module owns a trusted local child process; it does not expose a shell,
//! secrets, device endpoints, or claim that charmd is authenticated.

use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub installed: bool,
    pub process_running: bool,
    pub pid: Option<u32>,
    pub protocol_ready: bool,
    pub detail: String,
}

#[derive(Default)]
pub struct RuntimeManager {
    child: Mutex<Option<Child>>,
}

impl RuntimeManager {
    pub fn status(&self, binary: &Path) -> Result<RuntimeStatus, String> {
        let mut child = self.child.lock().map_err(|_| "OpenCharm lock poisoned")?;
        refresh(&mut child)?;
        Ok(snapshot(&child, binary))
    }

    pub fn start(&self, binary: &Path, data: &Path) -> Result<RuntimeStatus, String> {
        if !cfg!(windows) {
            return Err("OpenCharm extension currently targets Windows".into());
        }
        let mut child = self.child.lock().map_err(|_| "OpenCharm lock poisoned")?;
        refresh(&mut child)?;
        if child.is_some() {
            return Ok(snapshot(&child, binary));
        }
        if !binary.is_file() {
            return Err(format!("OpenCharm launcher has not been bundled: {}", binary.display()));
        }
        fs::create_dir_all(data).map_err(|e| e.to_string())?;
        let out = OpenOptions::new()
            .append(true)
            .create(true)
            .open(data.join("opencharm.log"))
            .map_err(|e| e.to_string())?;
        let err = out.try_clone().map_err(|e| e.to_string())?;
        let mut process = Command::new(binary);
        process.arg("serve")
            .env("OPENCHARM_DATA", data)
            .current_dir(data)
            .stdin(Stdio::null())
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            process.creation_flags(0x0800_0000);
        }
        *child = Some(process.spawn().map_err(|e| e.to_string())?);
        Ok(snapshot(&child, binary))
    }

    /// P1 local stop only. P2 must use graceful, authenticated admin IPC
    /// before long-running development tasks can be managed.
    pub fn stop(&self) -> Result<(), String> {
        let mut child = self.child.lock().map_err(|_| "OpenCharm lock poisoned")?;
        refresh(&mut child)?;
        if let Some(mut process) = child.take() {
            if let Err(e) = process.kill() {
                *child = Some(process);
                return Err(e.to_string());
            }
            process.wait().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

fn refresh(child: &mut Option<Child>) -> Result<(), String> {
    if let Some(process) = child.as_mut() {
        if process.try_wait().map_err(|e| e.to_string())?.is_some() {
            *child = None;
        }
    }
    Ok(())
}

fn snapshot(child: &Option<Child>, binary: &Path) -> RuntimeStatus {
    let installed = binary.is_file();
    let process_running = child.is_some();
    RuntimeStatus {
        installed,
        process_running,
        pid: child.as_ref().map(Child::id),
        protocol_ready: false,
        detail: if process_running {
            "Process alive; authenticated protocol readiness not verified"
        } else if installed {
            "OpenCharm launcher installed but not started"
        } else {
            "Launcher unavailable; Windows packaging is a separate step"
        }.into(),
    }
}

pub fn executable(resources: &Path) -> PathBuf {
    #[cfg(all(windows, debug_assertions))]
    if let Some(value) = std::env::var_os("CCSWITCH_OPENCHARM_DEV_EXECUTABLE") {
        return PathBuf::from(value);
    }
    resources.join("opencharm").join("opencharm-runtime.exe")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_runtime_does_not_report_ready() {
        let runtime = RuntimeManager::default();
        let missing = std::env::temp_dir().join("opencharm_missing_71c4.exe");
        let s = runtime.status(&missing).unwrap();
        assert!(!s.installed && !s.process_running && !s.protocol_ready);
        assert!(runtime.start(&missing, &std::env::temp_dir()).is_err());
        runtime.stop().unwrap();
    }
}
