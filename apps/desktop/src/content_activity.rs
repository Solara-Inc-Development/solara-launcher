//! In-memory activity for the inherited asynchronous non-modpack installer.
//! Raw engine error strings are intentionally never included in public snapshots.
use anyhow::{bail, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Queued,
    Running,
    Completed,
    Failed,
    Unknown,
}
impl Status {
    fn active(self) -> bool {
        matches!(self, Self::Queued | Self::Running | Self::Unknown)
    }
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub detail: &'static str,
    pub actions: Vec<&'static str>,
    #[serde(skip)]
    instance: String,
    #[serde(skip)]
    project: String,
}
#[derive(Default)]
pub struct Ledger {
    pub jobs: Vec<Job>,
}
impl Ledger {
    pub fn begin(&mut self, instance: &str, project: &str, name: &str) -> Result<String> {
        if self
            .jobs
            .iter()
            .any(|job| job.instance == instance && job.project == project && job.status.active())
        {
            bail!("This content is already being installed in that instance. Check Downloads before trying again.")
        }
        if self.jobs.iter().filter(|job| job.status.active()).count() >= 32 {
            bail!("Wait for an active content installation to finish before adding more")
        }
        while self.jobs.len() >= 100 {
            if let Some(index) = self.jobs.iter().position(|job| !job.status.active()) {
                self.jobs.remove(index);
            } else {
                bail!("Content installation queue is full")
            }
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.jobs.push(Job {
            id: id.clone(),
            name: format!("Content for {name}"),
            status: Status::Queued,
            detail: "Checking compatible versions and dependencies…",
            actions: vec![],
            instance: instance.into(),
            project: project.into(),
        });
        Ok(id)
    }
    pub fn running(&mut self, id: &str) {
        if let Some(job) = self
            .jobs
            .iter_mut()
            .find(|job| job.id == id && job.status == Status::Queued)
        {
            job.status = Status::Running;
            job.detail = "Downloading and installing content…";
        }
    }
    pub fn rejected(&mut self, id: &str) {
        if let Some(job) = self.jobs.iter_mut().find(|job| job.id == id) {
            job.status = Status::Failed;
            job.detail="Could not start installation. Check the instance's game version and loader, then try again.";
        }
    }
    pub fn completed(&mut self, instance: &str, projects: &[String], success: bool) {
        for job in &mut self.jobs {
            if job.instance == instance && projects.contains(&job.project) && job.status.active() {
                job.status = if success {
                    Status::Completed
                } else {
                    Status::Failed
                };
                job.detail = if success {
                    "Content installed."
                } else {
                    "Content download or installation failed. Check your connection and instance settings, then try again."
                };
            }
        }
    }
    pub fn unconfirmed(&mut self) {
        for job in &mut self.jobs {
            if job.status.active() {
                job.status = Status::Unknown;
                job.detail="Installation status could not be confirmed. Check this instance before retrying."
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn events_match_instance_and_project_and_cannot_regress_completion() {
        let mut ledger = Ledger::default();
        let id = ledger.begin("a", "mod", "World A").unwrap();
        assert!(ledger.begin("a", "mod", "World A").is_err());
        ledger.completed("other", &["mod".into()], true);
        assert_eq!(ledger.jobs[0].status, Status::Queued);
        ledger.completed("a", &["other".into()], true);
        assert_eq!(ledger.jobs[0].status, Status::Queued);
        ledger.completed("a", &["mod".into()], true);
        ledger.running(&id);
        assert_eq!(ledger.jobs[0].status, Status::Completed);
        assert!(ledger.begin("a", "mod", "World A").is_ok());
    }
    #[test]
    fn unknown_events_never_invent_failure_and_late_completion_recovers() {
        let mut ledger = Ledger::default();
        let id = ledger.begin("a", "mod", "World A").unwrap();
        ledger.running(&id);
        ledger.unconfirmed();
        assert_eq!(ledger.jobs[0].status, Status::Unknown);
        assert!(ledger.begin("a", "mod", "World A").is_err());
        ledger.completed("a", &["mod".into()], false);
        assert_eq!(ledger.jobs[0].status, Status::Failed);
        assert!(ledger.begin("a", "mod", "World A").is_ok());
    }
    #[test]
    fn history_is_bounded_and_active_jobs_are_not_evicted() {
        let mut ledger = Ledger::default();
        let active = ledger.begin("active", "mod", "Active").unwrap();
        for _ in 0..120 {
            let id = ledger.begin("a", "mod", "World A").unwrap();
            ledger.rejected(&id)
        }
        assert_eq!(ledger.jobs.len(), 100);
        assert!(ledger.jobs.iter().any(|job| job.id == active));
        let mut active = Ledger::default();
        for i in 0..32 {
            active.begin(&i.to_string(), "mod", "World").unwrap();
        }
        assert!(active.begin("overflow", "mod", "World").is_err());
    }
}
