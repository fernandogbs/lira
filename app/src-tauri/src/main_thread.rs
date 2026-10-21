//! Run UI work on the GTK main thread.
//!
//! GTK (and gtk-layer-shell) panic when called from any other thread. IPC
//! commands arrive on tokio workers and voice events on std threads, so every
//! UI entry point goes through [`run_on_main`], which runs the closure inline
//! when already on the main thread (tray/menu events) and otherwise dispatches
//! it and waits for the result, so errors are logged instead of lost.

use std::sync::mpsc;
use std::time::Duration;

/// How long a worker waits for the main thread before giving up (logged).
pub const MAIN_THREAD_TIMEOUT: Duration = Duration::from_secs(5);

pub type Job = Box<dyn FnOnce() + Send + 'static>;

/// Abstraction over "am I on the UI thread" and "post a job to it", so the
/// dispatch logic can be unit-tested without GTK.
pub trait MainThreadExecutor {
    fn is_main_thread(&self) -> bool;
    fn dispatch(&self, job: Job) -> Result<(), String>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum DispatchError {
    Dispatch(String),
    Timeout,
    Dropped,
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dispatch(e) => write!(f, "main-thread dispatch failed: {e}"),
            Self::Timeout => write!(f, "main thread did not run the job in time"),
            Self::Dropped => write!(f, "main-thread job was dropped before completing"),
        }
    }
}

/// Run `f` on the main thread and return its result.
pub fn run_on_main_with<E, T, F>(exec: &E, timeout: Duration, f: F) -> Result<T, DispatchError>
where
    E: MainThreadExecutor + ?Sized,
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    if exec.is_main_thread() {
        return Ok(f());
    }
    let (tx, rx) = mpsc::sync_channel(1);
    exec.dispatch(Box::new(move || {
        let _ = tx.send(f());
    }))
    .map_err(DispatchError::Dispatch)?;
    match rx.recv_timeout(timeout) {
        Ok(v) => Ok(v),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(DispatchError::Timeout),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(DispatchError::Dropped),
    }
}

/// Tauri implementation: the GTK main thread is Tauri's event-loop thread.
pub struct TauriExecutor<'a>(pub &'a tauri::AppHandle);

impl MainThreadExecutor for TauriExecutor<'_> {
    fn is_main_thread(&self) -> bool {
        is_gtk_main_thread()
    }

    fn dispatch(&self, job: Job) -> Result<(), String> {
        self.0.run_on_main_thread(job).map_err(|e| e.to_string())
    }
}

/// Run `f` on the GTK main thread, waiting up to [`MAIN_THREAD_TIMEOUT`].
pub fn run_on_main<T, F>(app: &tauri::AppHandle, f: F) -> Result<T, DispatchError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    run_on_main_with(&TauriExecutor(app), MAIN_THREAD_TIMEOUT, f)
}

#[cfg(target_os = "linux")]
pub fn is_gtk_main_thread() -> bool {
    gtk::is_initialized_main_thread()
}

#[cfg(not(target_os = "linux"))]
pub fn is_gtk_main_thread() -> bool {
    false
}

/// Guard for code that touches GTK directly.
#[inline]
pub fn assert_gtk_main_thread() {
    debug_assert!(
        is_gtk_main_thread(),
        "GTK used outside the main thread; wrap the call in main_thread::run_on_main"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::{self, ThreadId};

    /// Fake UI thread: a dedicated thread draining a job queue.
    struct FakeMain {
        tx: mpsc::Sender<Job>,
        id: ThreadId,
    }

    impl FakeMain {
        fn spawn() -> Self {
            let (tx, rx) = mpsc::channel::<Job>();
            let (id_tx, id_rx) = mpsc::channel();
            thread::spawn(move || {
                id_tx.send(thread::current().id()).unwrap();
                while let Ok(job) = rx.recv() {
                    job();
                }
            });
            Self {
                tx,
                id: id_rx.recv().unwrap(),
            }
        }
    }

    impl MainThreadExecutor for FakeMain {
        fn is_main_thread(&self) -> bool {
            thread::current().id() == self.id
        }
        fn dispatch(&self, job: Job) -> Result<(), String> {
            self.tx.send(job).map_err(|e| e.to_string())
        }
    }

    struct Broken;
    impl MainThreadExecutor for Broken {
        fn is_main_thread(&self) -> bool {
            false
        }
        fn dispatch(&self, _job: Job) -> Result<(), String> {
            Err("event loop gone".into())
        }
    }

    /// Accepts the job and drops it without running it.
    struct Dropping;
    impl MainThreadExecutor for Dropping {
        fn is_main_thread(&self) -> bool {
            false
        }
        fn dispatch(&self, job: Job) -> Result<(), String> {
            drop(job);
            Ok(())
        }
    }

    /// Accepts the job and never runs or drops it (blocked main loop).
    struct Stuck(std::sync::Mutex<Vec<Job>>);
    impl MainThreadExecutor for Stuck {
        fn is_main_thread(&self) -> bool {
            false
        }
        fn dispatch(&self, job: Job) -> Result<(), String> {
            self.0.lock().unwrap().push(job);
            Ok(())
        }
    }

    #[test]
    fn worker_call_runs_on_main_thread_and_returns_value() {
        let main = FakeMain::spawn();
        let main_id = main.id;
        let ran_on = run_on_main_with(&main, Duration::from_secs(2), || thread::current().id())
            .expect("dispatch");
        assert_eq!(ran_on, main_id);
        assert_ne!(ran_on, thread::current().id());
    }

    #[test]
    fn many_worker_threads_all_land_on_main() {
        let main = std::sync::Arc::new(FakeMain::spawn());
        let main_id = main.id;
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let main = std::sync::Arc::clone(&main);
                thread::spawn(move || {
                    run_on_main_with(&*main, Duration::from_secs(2), || thread::current().id())
                        .unwrap()
                })
            })
            .collect();
        for h in handles {
            assert_eq!(h.join().unwrap(), main_id);
        }
    }

    #[test]
    fn call_from_main_runs_inline_without_deadlock() {
        let main = std::sync::Arc::new(FakeMain::spawn());
        let inner = std::sync::Arc::clone(&main);
        // A job already on the main thread calls run_on_main again (tray path).
        let v = run_on_main_with(&*main, Duration::from_secs(2), move || {
            run_on_main_with(&*inner, Duration::from_millis(200), || 7).unwrap()
        })
        .unwrap();
        assert_eq!(v, 7);
    }

    #[test]
    fn dispatch_failure_is_reported() {
        let err = run_on_main_with(&Broken, Duration::from_millis(50), || ()).unwrap_err();
        assert!(matches!(err, DispatchError::Dispatch(_)));
    }

    #[test]
    fn dropped_job_is_reported() {
        let err = run_on_main_with(&Dropping, Duration::from_secs(1), || ()).unwrap_err();
        assert_eq!(err, DispatchError::Dropped);
    }

    #[test]
    fn blocked_main_thread_times_out() {
        let stuck = Stuck(std::sync::Mutex::new(Vec::new()));
        let err = run_on_main_with(&stuck, Duration::from_millis(50), || ()).unwrap_err();
        assert_eq!(err, DispatchError::Timeout);
    }
}
