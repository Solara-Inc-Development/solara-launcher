use super::download::Cancellation;
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::sync::Mutex;

#[derive(Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub status: String,
    pub kind: String,
    pub detail: String,
    pub downloaded: u64,
    pub total: u64,
    pub progress: Option<f64>,
    pub actions: Vec<String>,
    #[serde(rename = "instanceId")]
    pub instance: String,
    #[serde(skip)]
    pub cancel: Cancellation,
    #[serde(skip)]
    pub content_files: Option<Vec<super::content::VerifiedFile>>,
}
pub struct Jobs {
    items: Mutex<Vec<Job>>,
}
impl Default for Jobs {
    fn default() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
        }
    }
}
impl Jobs {
    pub fn begin(&self, instance: &str, name: &str) -> Result<Job> {
        self.begin_kind(instance, name, "install")
    }
    pub fn begin_kind(&self, instance: &str, name: &str, kind: &str) -> Result<Job> {
        self.begin_with_content(instance, name, kind, None)
    }
    pub fn begin_content(
        &self,
        instance: &str,
        name: &str,
        files: Vec<super::content::VerifiedFile>,
    ) -> Result<Job> {
        self.begin_with_content(instance, name, "content", Some(files))
    }
    fn begin_with_content(
        &self,
        instance: &str,
        name: &str,
        kind: &str,
        content_files: Option<Vec<super::content::VerifiedFile>>,
    ) -> Result<Job> {
        if !matches!(kind, "install" | "java" | "content" | "backup") {
            bail!("Unknown local operation")
        }

        let mut jobs = self
            .items
            .lock()
            .map_err(|_| anyhow::anyhow!("Installation queue unavailable"))?;
        if jobs
            .iter()
            .any(|j| j.instance == instance && matches!(j.status.as_str(), "queued" | "running"))
        {
            bail!("This instance is already being installed")
        }
        if jobs
            .iter()
            .filter(|j| matches!(j.status.as_str(), "queued" | "running"))
            .count()
            >= 16
        {
            bail!("The install queue is full. Wait for a download to finish.")
        }
        while jobs.len() >= 100 {
            if let Some(at) = jobs
                .iter()
                .position(|j| !matches!(j.status.as_str(), "queued" | "running"))
            {
                jobs.remove(at);
            } else {
                bail!("Installation queue is full")
            }
        }
        let job = Job {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            status: "queued".into(),
            kind: kind.into(),
            detail: "Waiting to start".into(),
            downloaded: 0,
            total: 0,
            progress: None,
            actions: vec!["cancel".into()],
            instance: instance.into(),
            cancel: Cancellation::default(),
            content_files,
        };
        jobs.push(job.clone());
        Ok(job)
    }
    pub fn list(&self) -> Result<Vec<Job>> {
        Ok(self
            .items
            .lock()
            .map_err(|_| anyhow::anyhow!("Installation queue unavailable"))?
            .clone())
    }
    pub fn get(&self, id: &str) -> Result<Job> {
        self.list()?
            .into_iter()
            .find(|j| j.id == id)
            .context("Download record not found")
    }
    pub fn report(&self, id: &str, phase: String, done: u64, total: u64) {
        if let Ok(mut jobs) = self.items.lock() {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                job.status = "running".into();
                job.detail = phase;
                job.downloaded = done;
                job.total = total;
                job.progress = if total > 0 {
                    Some((done as f64 / total as f64).clamp(0., 1.))
                } else {
                    None
                };
            }
        }
    }
    pub fn finish(&self, id: &str, success: bool, cancelled: bool) {
        if let Ok(mut jobs) = self.items.lock() {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                job.status = if success {
                    "completed"
                } else if cancelled {
                    "cancelled"
                } else {
                    "failed"
                }
                .into();
                job.detail=if success && job.kind == "java" {"Java is ready"} else if success && job.kind == "content" {"Content installed"} else if success {"Minecraft files are ready"}else if cancelled {"Installation cancelled. Your worlds were kept."}else{"Could not install these files. Check your connection and retry. Some older versions are not supported yet."}.into();
                job.actions = if success || (job.kind == "content" && job.content_files.is_none()) {
                    vec![]
                } else {
                    vec!["retry".into()]
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_core::{content::VerifiedFile, metadata::Artifact};
    #[test]
    fn queue_prevents_duplicates_and_records_real_terminal_states() {
        let jobs = Jobs::default();
        let first = jobs.begin("instance", "Fixture world").unwrap();
        assert!(jobs.begin("instance", "Duplicate").is_err());
        jobs.report(&first.id, "Preparing verified files".into(), 10, 20);
        assert_eq!(jobs.get(&first.id).unwrap().progress, Some(0.5));
        jobs.finish(&first.id, false, true);
        assert_eq!(jobs.get(&first.id).unwrap().status, "cancelled");
        assert!(jobs.begin("instance", "Retry").is_ok());

        let content_job = jobs
            .begin_content(
                "instance-2",
                "ModMenu for Fixture",
                vec![VerifiedFile {
                    destination: "mods/modmenu.jar".into(),
                    artifact: Artifact {
                        url: "https://cdn.modrinth.com/data/test/modmenu.jar".into(),
                        sha1: "a".repeat(40),
                        size: 128,
                        path: Some("mods/modmenu.jar".into()),
                    },
                    sha512: "b".repeat(128),
                }],
            )
            .unwrap();
        jobs.finish(&content_job.id, false, false);
        let failed_content = jobs.get(&content_job.id).unwrap();
        assert_eq!(failed_content.actions, vec!["retry"]);
        assert_eq!(failed_content.content_files.unwrap().len(), 1);

        let json = serde_json::to_value(jobs.list().unwrap()).unwrap();
        assert!(json[0].get("instance").is_none());
        assert_eq!(json[0]["instanceId"], "instance");
        assert!(json[0].get("cancel").is_none());
        assert!(json[2].get("content_files").is_none());
    }
}
