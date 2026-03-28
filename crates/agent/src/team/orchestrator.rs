use std::sync::Arc;

use tokio::sync::RwLock;

use super::types::{TaskStatus, TeamConfig, TeamEvent, TeamTask};

pub trait TeamEventHandler: Send + Sync + 'static {
    fn on_event(&self, event: TeamEvent);
}

pub struct TeamOrchestrator {
    config: TeamConfig,
    tasks: RwLock<Vec<TeamTask>>,
    handler: Arc<dyn TeamEventHandler>,
}

impl TeamOrchestrator {
    pub fn new(config: TeamConfig, handler: Arc<dyn TeamEventHandler>) -> Self {
        Self {
            config,
            tasks: RwLock::new(Vec::new()),
            handler,
        }
    }

    pub async fn add_tasks(&self, tasks: Vec<TeamTask>) {
        let mut current = self.tasks.write().await;
        current.extend(tasks);
    }

    pub async fn assign_task(&self, task_id: &str, member_role: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.assigned_to = Some(member_role.to_string());
            task.status = TaskStatus::InProgress;
            self.handler.on_event(TeamEvent::TaskAssigned {
                task_id: task_id.to_string(),
                member_role: member_role.to_string(),
            });
            true
        } else {
            false
        }
    }

    pub async fn complete_task(&self, task_id: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            let role = task.assigned_to.clone().unwrap_or_default();
            task.status = TaskStatus::Completed;
            self.handler.on_event(TeamEvent::TaskCompleted {
                task_id: task_id.to_string(),
                member_role: role,
            });

            if tasks.iter().all(|t| t.status == TaskStatus::Completed) {
                self.handler.on_event(TeamEvent::AllComplete);
            }
        }
    }

    pub async fn fail_task(&self, task_id: &str, reason: String) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            let role = task.assigned_to.clone().unwrap_or_default();
            task.status = TaskStatus::Failed;
            self.handler.on_event(TeamEvent::TaskFailed {
                task_id: task_id.to_string(),
                member_role: role,
                reason,
            });
        }
    }

    pub async fn pending_tasks(&self) -> Vec<TeamTask> {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .cloned()
            .collect()
    }

    pub async fn check_file_conflict(&self, member_role: &str, path: &str) -> bool {
        let tasks = self.tasks.read().await;
        tasks.iter().any(|t| {
            t.status == TaskStatus::InProgress
                && t.assigned_to.as_deref() != Some(member_role)
                && t.file_ownership.iter().any(|f| path.starts_with(f.as_str()))
        })
    }

    pub fn config(&self) -> &TeamConfig {
        &self.config
    }
}
