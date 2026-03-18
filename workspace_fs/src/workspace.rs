use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use axum::Json;
use axum::{http::StatusCode, response::{IntoResponse, Response}};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;

use crate::{
    config::RepositoryConfig,
    plugin::{PluginRunner, PluginTrigger},
    repository::Repository,
};

#[derive(Debug)]
pub struct WorkspaceError {
    pub status: StatusCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum MethodKind {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PolicyPermissions {
    #[serde(rename = "GET")]
    pub get: bool,
    #[serde(rename = "POST")]
    pub post: bool,
    #[serde(rename = "PUT")]
    pub put: bool,
    #[serde(rename = "DELETE")]
    pub delete: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PolicySpecificity {
    pub depth: usize,
    pub chars: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PolicyMatchInfo {
    pub index: usize,
    pub pattern: String,
    pub specificity: PolicySpecificity,
    pub permissions: PolicyPermissions,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SelectedPolicyInfo {
    pub index: usize,
    pub pattern: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PolicyInspection {
    pub path: String,
    pub matches: Vec<PolicyMatchInfo>,
    pub selected: Option<SelectedPolicyInfo>,
    pub effective: PolicyPermissions,
}

pub struct WorkspaceService {
    repository: Arc<dyn Repository>,
    config: Arc<RepositoryConfig>,
    repository_name: String,
}

impl WorkspaceError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, message: message.into() }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self { status: StatusCode::NOT_FOUND, message: message.into() }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self { status: StatusCode::CONFLICT, message: message.into() }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self { status: StatusCode::FORBIDDEN, message: message.into() }
    }

    pub fn internal(error: impl std::fmt::Display) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: error.to_string() }
    }
}

impl IntoResponse for WorkspaceError {
    fn into_response(self) -> Response {
        (
            self.status,
            [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            self.message,
        ).into_response()
    }
}

impl WorkspaceService {
    pub fn new(
        repository: Arc<dyn Repository>,
        config: Arc<RepositoryConfig>,
        repository_name: String,
    ) -> Self {
        Self { repository, config, repository_name }
    }

    pub fn serve_port(&self) -> u16 {
        self.config.serve.port
    }

    pub fn repository_root(&self) -> &Utf8Path {
        self.repository.repository_root()
    }

    pub async fn run_task(&self, task_name: &str) -> Result<()> {
        self.plugin_runner().run_task(task_name).await
    }

    pub async fn run_manual_plugin(&self, plugin_name: &str, user_identity: &str) -> Result<()> {
        self.plugin_runner().run_manual_plugin(plugin_name, user_identity).await
    }

    pub fn plugin_url_prefix(&self) -> &str {
        &self.config.serve.plugin_url_prefix
    }

    pub fn policy_url_prefix(&self) -> &str {
        &self.config.serve.policy_url_prefix
    }

    pub async fn get_root(&self, user_identity: &str) -> Result<Response, WorkspaceError> {
        self.enforce_policy(MethodKind::Get, "")?;
        self.directory_response("", user_identity).await
    }

    pub async fn get_path(&self, path: &str, user_identity: &str) -> Result<Response, WorkspaceError> {
        if let Some(response) = self.mounted_get_response(path, user_identity).await? {
            return Ok(response);
        }

        self.enforce_policy(MethodKind::Get, path)?;

        if path.ends_with('/') {
            return self.directory_response(path, user_identity).await;
        }

        if self.repository.list_directory(path).await.is_ok() {
            return Err(WorkspaceError::bad_request("directory path must end with /"));
        }

        let content = match self.repository.read_text_file(path).await {
            Ok(content) => content,
            Err(error) => {
                let mapped = self.map_read_error(error);
                tracing::warn!(user = %user_identity, path = %path, status = %mapped.status, error = %mapped.message, "read failed");
                return Err(mapped);
            }
        };

        self.run_trigger(PluginTrigger::Get, path, user_identity).await?;
        Ok(text_response(StatusCode::OK, content))
    }

    pub async fn create_path(&self, path: &str, body: &str, user_identity: &str) -> Result<Response, WorkspaceError> {
        self.enforce_policy(MethodKind::Post, path)?;

        if path.ends_with('/') {
            match self.repository.create_directory(path).await {
                Ok(()) => tracing::info!(user = %user_identity, path = %path, "directory created"),
                Err(error) => {
                    let mapped = self.map_create_directory_error(error);
                    tracing::warn!(user = %user_identity, path = %path, status = %mapped.status, error = %mapped.message, "directory create failed");
                    return Err(mapped);
                }
            }
        } else {
            match self.repository.create_text_file(path, body).await {
                Ok(()) => tracing::info!(user = %user_identity, path = %path, "file created"),
                Err(error) => {
                    let mapped = self.map_create_error(error);
                    tracing::warn!(user = %user_identity, path = %path, status = %mapped.status, error = %mapped.message, "file create failed");
                    return Err(mapped);
                }
            }
        }

        self.run_trigger(PluginTrigger::Post, path, user_identity).await?;
        Ok(StatusCode::CREATED.into_response())
    }

    pub async fn update_file(&self, path: &str, body: &str, user_identity: &str) -> Result<Response, WorkspaceError> {
        self.enforce_policy(MethodKind::Put, path)?;
        reject_directory_path(path, "cannot update a directory path with PUT")?;

        match self.repository.write_text_file(path, body).await {
            Ok(()) => tracing::info!(user = %user_identity, path = %path, "file updated"),
            Err(error) => {
                let mapped = self.map_write_error(error);
                tracing::warn!(user = %user_identity, path = %path, status = %mapped.status, error = %mapped.message, "file update failed");
                return Err(mapped);
            }
        }

        self.run_trigger(PluginTrigger::Put, path, user_identity).await?;
        Ok(StatusCode::NO_CONTENT.into_response())
    }

    pub async fn delete_path(&self, path: &str, user_identity: &str) -> Result<Response, WorkspaceError> {
        self.enforce_policy(MethodKind::Delete, path)?;

        if path.ends_with('/') {
            match self.repository.delete_directory(path).await {
                Ok(()) => tracing::info!(user = %user_identity, path = %path, "directory deleted"),
                Err(error) => {
                    let mapped = self.map_delete_directory_error(error);
                    tracing::warn!(user = %user_identity, path = %path, status = %mapped.status, error = %mapped.message, "directory delete failed");
                    return Err(mapped);
                }
            }
        } else {
            match self.repository.delete_file(path).await {
                Ok(()) => tracing::info!(user = %user_identity, path = %path, "file deleted"),
                Err(error) => {
                    let mapped = self.map_delete_error(error);
                    tracing::warn!(user = %user_identity, path = %path, status = %mapped.status, error = %mapped.message, "file delete failed");
                    return Err(mapped);
                }
            }
        }

        self.run_trigger(PluginTrigger::Delete, path, user_identity).await?;
        Ok(StatusCode::NO_CONTENT.into_response())
    }

    pub async fn inspect_policy(&self, path: &str) -> Result<Json<PolicyInspection>, WorkspaceError> {
        let inspection = self.inspect_policy_rules(path).map_err(WorkspaceError::internal)?;
        Ok(Json(inspection))
    }

    async fn directory_response(&self, path: &str, user_identity: &str) -> Result<Response, WorkspaceError> {
        let entries = match self.repository.list_directory(path).await {
            Ok(entries) => entries,
            Err(error) => {
                let message = error.to_string();
                let mapped = if message.contains("not a directory") || message.contains("No such file") {
                    WorkspaceError::not_found("directory not found")
                } else {
                    WorkspaceError::internal(error)
                };
                tracing::warn!(user = %user_identity, path = %path, status = %mapped.status, error = %mapped.message, "directory listing failed");
                return Err(mapped);
            }
        };

        self.run_trigger(PluginTrigger::Get, path, user_identity).await?;
        Ok(text_response(StatusCode::OK, entries.join("\n")))
    }

    async fn mounted_get_response(&self, path: &str, user_identity: &str) -> Result<Option<Response>, WorkspaceError> {
        let request_path = if path.is_empty() { "/".to_owned() } else { format!("/{path}") };
        let Some(mount) = self.config.mount.iter().find(|mount| request_path.starts_with(&mount.url_prefix)) else {
            return Ok(None);
        };

        let relative = request_path
            .strip_prefix(&mount.url_prefix)
            .unwrap_or_default()
            .trim_end_matches('/');
        let mut target = self.repository.repository_root().to_owned();
        target.push(mount.source_relative_path());
        if !relative.is_empty() {
            target.push(relative);
        }

        if target.is_dir() {
            let entries = list_internal_directory(&target).map_err(WorkspaceError::internal)?;
            tracing::info!(user = %user_identity, path = %request_path, mount = %mount.url_prefix, "mounted directory served");
            return Ok(Some(text_response(StatusCode::OK, entries.join("\n"))));
        }

        if target.is_file() {
            let content = tokio::fs::read_to_string(target.as_std_path())
                .await
                .context("failed to read mounted file")
                .map_err(WorkspaceError::internal)?;
            tracing::info!(user = %user_identity, path = %request_path, mount = %mount.url_prefix, "mounted file served");
            return Ok(Some(text_response(StatusCode::OK, content)));
        }

        Ok(None)
    }

    async fn run_trigger(&self, trigger: PluginTrigger, path: &str, user_identity: &str) -> Result<(), WorkspaceError> {
        match self.plugin_runner().run_hook_if_matched(trigger, path, user_identity).await {
            Ok(()) => Ok(()),
            Err(error) => {
                tracing::warn!(user = %user_identity, path = %path, trigger = %trigger.as_str(), error = %error, "plugin hook failed");
                Err(WorkspaceError::internal(error))
            }
        }
    }

    fn plugin_runner(&self) -> PluginRunner<'_> {
        PluginRunner::new(self.repository.repository_root(), &self.repository_name, &self.config)
    }

    fn enforce_policy(&self, method: MethodKind, path: &str) -> Result<(), WorkspaceError> {
        let normalized = if path.is_empty() { "" } else { path };
        let allowed = self
            .resolve_policy(method, normalized)
            .map_err(WorkspaceError::internal)?
            .unwrap_or(false);

        if allowed {
            Ok(())
        } else {
            Err(WorkspaceError::forbidden("operation denied by policy"))
        }
    }

    fn resolve_policy(&self, method: MethodKind, path: &str) -> Result<Option<bool>> {
        let inspection = self.inspect_policy_rules(path)?;
        Ok(Some(match method {
            MethodKind::Get => inspection.effective.get,
            MethodKind::Post => inspection.effective.post,
            MethodKind::Put => inspection.effective.put,
            MethodKind::Delete => inspection.effective.delete,
        })
        .filter(|allowed| *allowed || inspection.selected.is_some()))
    }

    fn inspect_policy_rules(&self, path: &str) -> Result<PolicyInspection> {
        let mut matches = Vec::new();
        let mut selected: Option<(PolicyMatchInfo, String)> = None;

        for (index, rule) in self.config.policy.iter().enumerate() {
            if !glob::Pattern::new(&rule.path)?.matches(path) {
                continue;
            }

            let candidate = PolicyMatchInfo {
                index,
                pattern: rule.path.clone(),
                specificity: policy_specificity(&rule.path),
                permissions: PolicyPermissions::from_rule(rule),
            };

            match selected {
                Some((ref best, _))
                    if best.specificity > candidate.specificity
                        || (best.specificity == candidate.specificity && best.index > candidate.index) => {}
                Some((ref best, _)) if best.specificity == candidate.specificity => {
                    selected = Some((candidate.clone(), "later_rule".to_owned()));
                }
                Some((ref best, _)) if best.specificity < candidate.specificity => {
                    selected = Some((candidate.clone(), "more_specific".to_owned()));
                }
                None => {
                    selected = Some((candidate.clone(), "first_match".to_owned()));
                }
                _ => {}
            }

            matches.push(candidate);
        }

        let effective = selected
            .as_ref()
            .map(|(selected, _)| selected.permissions.clone())
            .unwrap_or_else(PolicyPermissions::deny_all);
        let selected = selected.map(|(selected, reason)| SelectedPolicyInfo {
            index: selected.index,
            pattern: selected.pattern,
            reason,
        });

        Ok(PolicyInspection {
            path: path.to_owned(),
            matches,
            selected,
            effective,
        })
    }

    fn map_create_error(&self, error: anyhow::Error) -> WorkspaceError {
        let message = error.to_string();
        if message.contains("file already exists") {
            return WorkspaceError::conflict("file already exists");
        }
        if message.contains("parent directory not found") {
            return WorkspaceError::not_found("parent directory not found");
        }
        if message.contains("parent path is not a directory") {
            return WorkspaceError::bad_request("parent path is not a directory");
        }
        map_path_error(error)
    }

    fn map_create_directory_error(&self, error: anyhow::Error) -> WorkspaceError {
        let message = error.to_string();
        if message.contains("directory already exists") {
            return WorkspaceError::conflict("directory already exists");
        }
        if message.contains("parent directory not found") {
            return WorkspaceError::not_found("parent directory not found");
        }
        if message.contains("parent path is not a directory") {
            return WorkspaceError::bad_request("parent path is not a directory");
        }
        map_path_error(error)
    }

    fn map_write_error(&self, error: anyhow::Error) -> WorkspaceError {
        let message = error.to_string();
        if message.contains("file not found") {
            return WorkspaceError::not_found("file not found");
        }
        if message.contains("path is a directory") {
            return WorkspaceError::bad_request("path is a directory");
        }
        map_path_error(error)
    }

    fn map_read_error(&self, error: anyhow::Error) -> WorkspaceError {
        let message = error.to_string();
        if message.contains("No such file") || message.contains("os error 2") {
            return WorkspaceError::not_found("path not found");
        }
        if message.contains("Is a directory") || message.contains("os error 21") {
            return WorkspaceError::bad_request("path is a directory");
        }
        map_path_error(error)
    }

    fn map_delete_error(&self, error: anyhow::Error) -> WorkspaceError {
        let message = error.to_string();
        if message.contains("file not found") {
            return WorkspaceError::not_found("file not found");
        }
        if message.contains("path is a directory") {
            return WorkspaceError::bad_request("path is a directory");
        }
        map_path_error(error)
    }

    fn map_delete_directory_error(&self, error: anyhow::Error) -> WorkspaceError {
        let message = error.to_string();
        if message.contains("directory not found") {
            return WorkspaceError::not_found("directory not found");
        }
        if message.contains("path is not a directory") {
            return WorkspaceError::bad_request("path is not a directory");
        }
        if message.contains("directory is not empty") {
            return WorkspaceError::conflict("directory is not empty");
        }
        map_path_error(error)
    }
}

fn list_internal_directory(path: &Utf8Path) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path.as_std_path())? {
        let entry = entry?;
        let path = Utf8PathBuf::from_path_buf(entry.path()).map_err(|_| anyhow!("non-UTF-8 path"))?;
        let mut name = path
            .file_name()
            .ok_or_else(|| anyhow!("invalid directory entry"))?
            .to_owned();
        if path.is_dir() {
            name.push('/');
        }
        entries.push(name);
    }
    entries.sort();
    Ok(entries)
}

fn reject_directory_path(path: &str, message: &'static str) -> Result<(), WorkspaceError> {
    if path.ends_with('/') {
        return Err(WorkspaceError::bad_request(message));
    }
    Ok(())
}

fn map_path_error(error: anyhow::Error) -> WorkspaceError {
    let message = error.to_string();
    if message.contains("path escapes repository root")
        || message.contains("absolute paths are not allowed")
        || message.contains("reserved path")
    {
        return WorkspaceError::bad_request(message);
    }
    WorkspaceError::internal(error)
}

fn text_response(status: StatusCode, body: String) -> Response {
    (
        status,
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    ).into_response()
}

fn policy_specificity(pattern: &str) -> PolicySpecificity {
    let normalized = pattern.trim_end_matches('/');
    if normalized.is_empty() {
        return PolicySpecificity { depth: 0, chars: 0 };
    }

    let mut depth = 0;
    let mut chars = 0;
    for segment in normalized.split('/') {
        if segment.contains('*') || segment.contains('?') || segment.contains('[') {
            break;
        }
        depth += 1;
        chars += segment.len();
    }

    PolicySpecificity { depth, chars }
}

impl PolicyPermissions {
    fn from_rule(rule: &crate::config::PolicyRule) -> Self {
        Self {
            get: rule.get,
            post: rule.post,
            put: rule.put,
            delete: rule.delete,
        }
    }

    fn deny_all() -> Self {
        Self {
            get: false,
            post: false,
            put: false,
            delete: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        config::{PolicyRule, RepositoryConfig, ServeSettings},
        repository::FsRepository,
    };

    fn service(policy: Vec<PolicyRule>) -> WorkspaceService {
        WorkspaceService::new(
            Arc::new(FsRepository::open(".".to_owned()).unwrap()),
            Arc::new(RepositoryConfig {
                serve: ServeSettings::default(),
                policy,
                mount: Vec::new(),
                plugin: Vec::new(),
                task: Vec::new(),
            }),
            "test".to_owned(),
        )
    }

    #[test]
    fn more_specific_child_policy_wins() {
        let service = service(vec![
            PolicyRule {
                path: "docs/**".to_owned(),
                get: false,
                post: false,
                put: false,
                delete: false,
            },
            PolicyRule {
                path: "docs/public/**".to_owned(),
                get: true,
                post: false,
                put: false,
                delete: false,
            },
        ]);

        assert_eq!(
            service.resolve_policy(MethodKind::Get, "docs/public/index.md").unwrap(),
            Some(true)
        );
    }

    #[test]
    fn equal_specificity_uses_later_rule() {
        let service = service(vec![
            PolicyRule {
                path: "docs/**".to_owned(),
                get: true,
                post: false,
                put: false,
                delete: false,
            },
            PolicyRule {
                path: "docs/**".to_owned(),
                get: false,
                post: false,
                put: false,
                delete: false,
            },
        ]);

        assert_eq!(service.resolve_policy(MethodKind::Get, "docs/a.md").unwrap(), Some(false));
    }

    #[test]
    fn no_matching_policy_denies_by_default() {
        let service = service(vec![PolicyRule {
            path: "docs/**".to_owned(),
            get: true,
            post: false,
            put: false,
            delete: false,
        }]);

        assert_eq!(service.resolve_policy(MethodKind::Get, "notes/a.md").unwrap(), None);
    }

    #[test]
    fn inspection_reports_matches_and_selected_rule() {
        let service = service(vec![
            PolicyRule {
                path: "**/*.md".to_owned(),
                get: true,
                post: false,
                put: true,
                delete: false,
            },
            PolicyRule {
                path: "docs/private/**".to_owned(),
                get: false,
                post: false,
                put: false,
                delete: false,
            },
        ]);

        let inspection = service.inspect_policy_rules("docs/private/a.md").unwrap();

        assert_eq!(inspection.matches.len(), 2);
        let selected = inspection.selected.unwrap();
        assert_eq!(selected.pattern, "docs/private/**");
        assert_eq!(selected.reason, "more_specific");
        assert_eq!(inspection.effective.get, false);
    }
}
