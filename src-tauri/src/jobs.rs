//! Job lifetime includes queueing, every engine stage and final publication.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{watch, Semaphore};

#[derive(Default)]
struct Jobs {
    shutting_down: bool,
    active: HashMap<String, watch::Sender<bool>>,
}

pub struct AppState {
    jobs: Mutex<Jobs>,
    pub conversion_semaphore: Arc<Semaphore>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            jobs: Mutex::new(Jobs::default()),
            conversion_semaphore: Arc::new(Semaphore::new(num_cpus::get().clamp(1, 4))),
        }
    }
}

pub struct JobGuard<'a> {
    state: &'a AppState,
    id: String,
}

impl Drop for JobGuard<'_> {
    fn drop(&mut self) {
        self.state.jobs.lock().unwrap().active.remove(&self.id);
    }
}

impl AppState {
    pub fn register(&self, id: &str) -> Result<JobGuard<'_>, String> {
        if id.is_empty() || id.len() > 128 {
            return Err("Invalid job ID".into());
        }
        let mut jobs = self.jobs.lock().unwrap();
        if jobs.shutting_down || jobs.active.len() >= 256 {
            return Err("Conversion queue unavailable or full (maximum 256 jobs)".into());
        }
        if jobs.active.contains_key(id) {
            return Err("Job ID is already active".into());
        }
        jobs.active.insert(id.into(), watch::channel(false).0);
        Ok(JobGuard {
            state: self,
            id: id.into(),
        })
    }

    pub fn subscribe(&self, id: &str) -> Result<watch::Receiver<bool>, String> {
        self.jobs
            .lock()
            .unwrap()
            .active
            .get(id)
            .map(|s| s.subscribe())
            .ok_or_else(|| "Job is no longer active".into())
    }

    pub fn check(&self, id: &str) -> Result<(), String> {
        if *self.subscribe(id)?.borrow() {
            Err("Conversion cancelled".into())
        } else {
            Ok(())
        }
    }

    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let jobs = self.jobs.lock().unwrap();
        let sender = jobs
            .active
            .get(id)
            .ok_or("Job not found or already completed")?;
        sender.send_replace(true);
        Ok(())
    }

    // Serialize the cancellation decision with publication: a successful cancel
    // cannot be followed by publishing an output or starting a new stage.
    pub fn publish<T>(
        &self,
        id: &str,
        action: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let mut jobs = self.jobs.lock().unwrap();
        let sender = jobs.active.get(id).ok_or("Job is no longer active")?;
        if *sender.borrow() {
            return Err("Conversion cancelled".into());
        }
        let result = action();
        jobs.active.remove(id);
        result
    }

    pub fn shutdown(&self) -> bool {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.shutting_down = true;
        for sender in jobs.active.values() {
            sender.send_replace(true);
        }
        !jobs.active.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.lock().unwrap().active.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_covers_stages_and_guards_remove_jobs() {
        let state = AppState::default();
        let guard = state.register("job").unwrap();
        assert!(state.register("job").is_err());
        state.cancel("job").unwrap();
        state.cancel("job").unwrap();
        assert!(state.check("job").is_err());
        assert!(state.publish("job", || Ok(())).is_err());
        drop(guard);
        assert!(state.is_empty());
        assert!(state.cancel("job").is_err());
    }

    #[test]
    fn publication_finishes_job_and_shutdown_rejects_new_work() {
        let state = AppState::default();
        let _guard = state.register("job").unwrap();
        state.publish("job", || Ok(())).unwrap();
        assert!(state.cancel("job").is_err());
        assert!(!state.shutdown());
        assert!(state.register("later").is_err());
    }
}
