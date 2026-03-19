use std::process::Stdio;

use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use glob::Pattern;
use tokio::process::Command;

use crate::{
    config::{PluginConfig, RepositoryConfig, TaskConfig},
    identity::UserIdentity,
    path::WorkspacePath,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginTrigger {
    Get,
    Post,
    Put,
    Delete,
    Manual,
}

impl PluginTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginContext {
    pub repository_root: Utf8PathBuf,
    pub repository_name: String,
    pub plugin_name: String,
    pub output_directory: Utf8PathBuf,
    pub cache_directory: Utf8PathBuf,
    pub path: Option<WorkspacePath>,
    pub user_identity: UserIdentity,
}

pub struct PluginRunner<'a> {
    repository_root: &'a camino::Utf8Path,
    repository_name: &'a str,
    config: &'a RepositoryConfig,
}

impl<'a> PluginRunner<'a> {
    pub fn new(
        repository_root: &'a camino::Utf8Path,
        repository_name: &'a str,
        config: &'a RepositoryConfig,
    ) -> Self {
        Self {
            repository_root,
            repository_name,
            config,
        }
    }

    pub async fn run_task(&self, task_name: &str) -> Result<()> {
        let task = self
            .config
            .find_task(task_name)
            .with_context(|| format!("task not found: {task_name}"))?;

        for step in &task.steps {
            let plugin = self
                .config
                .find_plugin(step)
                .with_context(|| format!("plugin not found for task step: {step}"))?;
            self.run_plugin(plugin, PluginTrigger::Manual, None, &UserIdentity::new(""))
                .await?;
        }

        Ok(())
    }

    pub async fn run_manual_plugin(
        &self,
        plugin_name: &str,
        user_identity: &UserIdentity,
    ) -> Result<()> {
        let plugin = self
            .config
            .find_plugin(plugin_name)
            .with_context(|| format!("plugin not found: {plugin_name}"))?;

        if plugin.trigger != "manual" {
            bail!("plugin is not manual: {plugin_name}");
        }

        self.run_plugin(plugin, PluginTrigger::Manual, None, user_identity)
            .await
    }

    pub async fn run_hook_if_matched(
        &self,
        trigger: PluginTrigger,
        path: &WorkspacePath,
        user_identity: &UserIdentity,
    ) -> Result<()> {
        for plugin in &self.config.plugin {
            if parse_trigger(&plugin.trigger)? != trigger {
                continue;
            }

            if !Pattern::new(&plugin.name)
                .with_context(|| format!("invalid plugin path pattern: {}", plugin.name))?
                .matches(path.as_str())
            {
                continue;
            }

            self.run_plugin(plugin, trigger, Some(path), user_identity)
                .await?;
        }

        Ok(())
    }

    async fn run_plugin(
        &self,
        plugin: &PluginConfig,
        trigger: PluginTrigger,
        path: Option<&WorkspacePath>,
        user_identity: &UserIdentity,
    ) -> Result<()> {
        let path_str = path.map(WorkspacePath::as_str);
        let context = PluginContext {
            repository_root: self.repository_root.to_owned(),
            repository_name: self.repository_name.to_owned(),
            plugin_name: plugin.name.clone(),
            output_directory: self
                .repository_root
                .join(".repo")
                .join(&plugin.name)
                .join("generated"),
            cache_directory: self
                .repository_root
                .join(".repo")
                .join(&plugin.name)
                .join("cache"),
            path: path.cloned(),
            user_identity: user_identity.clone(),
        };

        tokio::fs::create_dir_all(context.output_directory.as_std_path())
            .await
            .context("failed to create plugin output directory")?;
        tokio::fs::create_dir_all(context.cache_directory.as_std_path())
            .await
            .context("failed to create plugin cache directory")?;

        let program = expand_placeholder(&plugin.command[0], &context, trigger)?;
        let args = plugin.command[1..]
            .iter()
            .map(|arg| expand_placeholder(arg, &context, trigger))
            .collect::<Result<Vec<_>>>()?;

        tracing::info!(
            plugin = %plugin.name,
            trigger = %trigger.as_str(),
            path = %path_str.unwrap_or(""),
            "running plugin"
        );

        let mut command = Command::new(&program);
        command
            .args(&args)
            .current_dir(context.repository_root.as_std_path())
            .env(
                "WORKSPACE_FS_REPOSITORY_ROOT",
                context.repository_root.as_str(),
            )
            .env("WORKSPACE_FS_REPOSITORY_NAME", &context.repository_name)
            .env("WORKSPACE_FS_PLUGIN_NAME", &context.plugin_name)
            .env(
                "WORKSPACE_FS_OUTPUT_DIRECTORY",
                context.output_directory.as_str(),
            )
            .env(
                "WORKSPACE_FS_CACHE_DIRECTORY",
                context.cache_directory.as_str(),
            )
            .env("WORKSPACE_FS_TRIGGER", trigger.as_str())
            .env("WORKSPACE_FS_USER_IDENTITY", context.user_identity.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        if let Some(path) = path_str {
            command.env("WORKSPACE_FS_PATH", path);
        }

        let status = command
            .status()
            .await
            .with_context(|| format!("failed to run plugin: {}", plugin.name))?;

        if !status.success() {
            bail!("plugin failed: {}: {}", plugin.name, status);
        }

        Ok(())
    }
}

fn parse_trigger(trigger: &str) -> Result<PluginTrigger> {
    match trigger {
        "GET" => Ok(PluginTrigger::Get),
        "POST" => Ok(PluginTrigger::Post),
        "PUT" => Ok(PluginTrigger::Put),
        "DELETE" => Ok(PluginTrigger::Delete),
        "manual" => Ok(PluginTrigger::Manual),
        _ => bail!("unsupported plugin trigger: {trigger}"),
    }
}

fn expand_placeholder(
    input: &str,
    context: &PluginContext,
    trigger: PluginTrigger,
) -> Result<String> {
    let mut value = input.to_owned();
    if context.path.is_none() && contains_path_placeholder(input) {
        bail!("path placeholder requires request path: {input}");
    }

    let replacements = [
        ("{REPOSITORY_ROOT}", context.repository_root.as_str()),
        ("{REPOSITORY_NAME}", context.repository_name.as_str()),
        ("{PLUGIN_NAME}", context.plugin_name.as_str()),
        ("{OUTPOST_DIRECTORY}", context.output_directory.as_str()),
        ("{OUTPUT_DIRECTORY}", context.output_directory.as_str()),
    ];

    for (from, to) in replacements {
        value = value.replace(from, to);
    }

    if let Some((path_placeholder, user_placeholder)) = request_placeholders(trigger) {
        let path = context.path.as_ref().map(WorkspacePath::as_str).unwrap_or("");
        value = value.replace(path_placeholder, path);
        value = value.replace(user_placeholder, context.user_identity.as_str());
    }

    if value.contains('{') || value.contains('}') {
        bail!("unknown placeholder in plugin command: {input}");
    }

    Ok(value)
}

fn contains_path_placeholder(input: &str) -> bool {
    ["{GET.PATH}", "{POST.PATH}", "{PUT.PATH}", "{DELETE.PATH}"]
        .into_iter()
        .any(|placeholder| input.contains(placeholder))
}

fn request_placeholders(trigger: PluginTrigger) -> Option<(&'static str, &'static str)> {
    match trigger {
        PluginTrigger::Get => Some(("{GET.PATH}", "{GET.USER-IDENTITY}")),
        PluginTrigger::Post => Some(("{POST.PATH}", "{POST.USER-IDENTITY}")),
        PluginTrigger::Put => Some(("{PUT.PATH}", "{PUT.USER-IDENTITY}")),
        PluginTrigger::Delete => Some(("{DELETE.PATH}", "{DELETE.USER-IDENTITY}")),
        PluginTrigger::Manual => None,
    }
}

#[allow(dead_code)]
fn _task_reference(_task: &TaskConfig) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn plugin_context(path: Option<&str>) -> PluginContext {
        PluginContext {
            repository_root: Utf8PathBuf::from("/repo"),
            repository_name: "repo".into(),
            plugin_name: "plugin".into(),
            output_directory: Utf8PathBuf::from("/repo/.repo/plugin/generated"),
            cache_directory: Utf8PathBuf::from("/repo/.repo/plugin/cache"),
            path: path.map(|value| WorkspacePath::from_url(&format!("/{value}")).unwrap()),
            user_identity: UserIdentity::new("user"),
        }
    }

    #[test]
    fn expand_placeholder_rejects_path_placeholder_without_path() {
        let error = expand_placeholder("{GET.PATH}", &plugin_context(None), PluginTrigger::Manual)
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("path placeholder requires request path")
        );
    }

    #[test]
    fn expand_placeholder_replaces_trigger_specific_values() {
        let value = expand_placeholder(
            "{REPOSITORY_ROOT}:{POST.PATH}:{POST.USER-IDENTITY}",
            &plugin_context(Some("docs/a.md")),
            PluginTrigger::Post,
        )
        .unwrap();

        assert_eq!(value, "/repo:docs/a.md:user");
    }

    #[test]
    fn expand_placeholder_rejects_mismatched_trigger_placeholder() {
        let error = expand_placeholder(
            "{GET.USER-IDENTITY}",
            &plugin_context(Some("docs/a.md")),
            PluginTrigger::Manual,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown placeholder"));
    }
}
