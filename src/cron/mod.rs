
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub schedule: String,
    pub command: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CronScheduler {
    pub jobs: Vec<CronJob>,
}

impl CronScheduler {
    pub fn add(&mut self, job: CronJob) {
        self.jobs.push(job);
    }

    pub fn remove(&mut self, id: &str) {
        self.jobs.retain(|j| j.id != id);
    }

    pub fn list(&self) -> &[CronJob] {
        &self.jobs
    }
}
