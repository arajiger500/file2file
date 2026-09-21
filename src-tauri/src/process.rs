//! Bounded, cancellable process execution. No shell parsing or inherited stdin.
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::sync::watch;

#[cfg(windows)]
// Store the owned Win32 handle as an integer so this guard can move with the
// async task. Kernel handles are process-wide and CloseHandle is thread-safe.
struct WindowsJob(isize);

#[cfg(windows)]
impl WindowsJob {
    fn assign(child: &tokio::process::Child) -> Result<Self, String> {
        use std::{ffi::c_void, mem::size_of};
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(format!(
                "Could not create process job: {}",
                std::io::Error::last_os_error()
            ));
        }
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast::<c_void>(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        let handle = child.raw_handle().ok_or("Missing child process handle")?;
        let assigned = if configured != 0 {
            unsafe { AssignProcessToJobObject(job, handle.cast()) }
        } else {
            0
        };
        if assigned == 0 {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(job) };
            return Err(format!(
                "Could not contain child process: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(Self(job as isize))
    }
}

#[cfg(windows)]
impl Drop for WindowsJob {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(
                self.0 as windows_sys::Win32::Foundation::HANDLE,
            )
        };
    }
}

#[derive(Debug)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

async fn read_bounded(mut pipe: impl AsyncRead + Unpin, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = pipe.read(&mut chunk).await.map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(bytes);
        }
        if bytes.len() + n > limit {
            return Err("Engine output exceeded capture limit".into());
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
}

pub async fn cancelled(receiver: &mut watch::Receiver<bool>) {
    loop {
        if *receiver.borrow_and_update() {
            return;
        }
        if receiver.changed().await.is_err() {
            return;
        }
    }
}

#[cfg(unix)]
struct ProcessGroup(u32);
#[cfg(unix)]
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        // This group was created for this child, never the application's group.
        unsafe {
            libc::kill(-(self.0 as i32), libc::SIGKILL);
        }
    }
}

pub async fn run(
    mut command: Command,
    timeout: Duration,
    limit: usize,
    mut cancellation: Option<watch::Receiver<bool>>,
) -> Result<CommandOutput, String> {
    if cancellation.as_ref().is_some_and(|r| *r.borrow()) {
        return Err("Conversion cancelled".into());
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    #[cfg(windows)]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let mut child = command
        .spawn()
        .map_err(|e| format!("Engine could not start: {e}"))?;
    #[cfg(unix)]
    let group = ProcessGroup(child.id().ok_or("Missing child PID")?);
    #[cfg(windows)]
    let job = WindowsJob::assign(&child)?;
    let stdout = child.stdout.take().ok_or("Missing stdout pipe")?;
    let stderr = child.stderr.take().ok_or("Missing stderr pipe")?;
    let cancel = async {
        match cancellation.as_mut() {
            Some(receiver) => cancelled(receiver).await,
            None => std::future::pending::<()>().await,
        }
    };
    let result = tokio::select! {
        biased;
        _ = cancel => Err("Conversion cancelled".into()),
        _ = tokio::time::sleep(timeout) => Err("Engine timed out".into()),
        output = async {
            let (stdout, stderr, status) = tokio::try_join!(
                read_bounded(stdout, limit),
                read_bounded(stderr, limit),
                async { child.wait().await.map_err(|e| e.to_string()) },
            )?;
            Ok(CommandOutput { success: status.success(), stdout, stderr })
        } => output,
    };
    #[cfg(unix)]
    drop(group);
    #[cfg(windows)]
    drop(job);
    if result.is_err() {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn bounded_reader_rejects_excess_output() {
        assert!(read_bounded(&b"12345"[..], 4).await.is_err());
        assert_eq!(read_bounded(&b"1234"[..], 4).await.unwrap(), b"1234");
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn timeout_and_cancel_reap_real_children() {
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        assert!(run(command, Duration::from_millis(30), 1024, None)
            .await
            .unwrap_err()
            .contains("timed out"));
        let (tx, rx) = watch::channel(false);
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let task = tokio::spawn(run(command, Duration::from_secs(10), 1024, Some(rx)));
        tokio::task::yield_now().await;
        tx.send_replace(true);
        assert!(task.await.unwrap().unwrap_err().contains("cancelled"));
    }
}
