use super::*;
use suncode_agent::domain::ProjectRecord;

impl AgentSdk {
    pub fn list_projects(&self) -> SdkResult<ProjectsResult> {
        Ok(ProjectsResult {
            projects: self.state.store.projects(false)?,
        })
    }

    pub fn open_project(&self, path: &str, display_name: Option<&str>) -> SdkResult<ProjectRecord> {
        let result = self
            .state
            .operations
            .open_project(std::path::Path::new(path))
            .map_err(operation_error)?;
        let root = result
            .get("canonical_path")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                BusinessError::unavailable("project/open did not return a canonical path")
            })?;
        let display_name = display_name
            .or_else(|| result.get("display_name").and_then(Value::as_str))
            .unwrap_or("Project");
        let project = self.state.store.project(root, display_name)?;
        if let Ok(mut active) = self.state.active_project.lock() {
            *active = Some(project.project_id.clone());
        }
        self.runtime.block_on(
            self.state
                .agent
                .activate_mcp_project(&project.project_id, Path::new(&project.canonical_root)),
        )?;
        Ok(project)
    }

    pub fn select_project(&self, project_id: &str) -> SdkResult<ProjectRecord> {
        let project = self
            .state
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| BusinessError::missing("project"))?;
        self.state
            .operations
            .open_project(std::path::Path::new(&project.canonical_root))
            .map_err(operation_error)?;
        if let Ok(mut active) = self.state.active_project.lock() {
            *active = Some(project.project_id.clone());
        }
        self.runtime.block_on(
            self.state
                .agent
                .activate_mcp_project(&project.project_id, Path::new(&project.canonical_root)),
        )?;
        Ok(project)
    }

    pub fn list_project_dependencies(
        &self,
        project_id: &str,
    ) -> SdkResult<ProjectDependenciesResult> {
        if self.state.store.project_by_id(project_id)?.is_none() {
            return Err(BusinessError::missing("project"));
        }
        Ok(ProjectDependenciesResult {
            project_id: project_id.to_string(),
            dependencies: self
                .state
                .store
                .project_dependencies(project_id)?
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }

    pub fn add_project_dependency(
        &self,
        project_id: &str,
        path: &str,
    ) -> SdkResult<ProjectDependencyDto> {
        let project = self
            .state
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| BusinessError::missing("project"))?;
        let opened = self
            .state
            .operations
            .open_project(Path::new(path))
            .map_err(operation_error)?;
        let canonical_root = opened
            .get("canonical_path")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::unavailable("dependency root was not canonicalized"))?;
        let display_name = opened
            .get("display_name")
            .and_then(Value::as_str)
            .unwrap_or("Dependency");
        let dependency_root = Path::new(canonical_root);
        let project_root = Path::new(&project.canonical_root);
        if dependency_root.starts_with(project_root) || project_root.starts_with(dependency_root) {
            return Err(BusinessError::invalid(
                "dependency root must not equal, contain, or be contained by the project root",
            ));
        }
        if self
            .state
            .store
            .project_dependencies(project_id)?
            .iter()
            .map(|dependency| Path::new(&dependency.canonical_root))
            .any(|existing| {
                dependency_root.starts_with(existing) || existing.starts_with(dependency_root)
            })
        {
            return Err(BusinessError::invalid(
                "dependency roots must not equal, contain, or be contained by each other",
            ));
        }
        self.state
            .store
            .add_project_dependency(project_id, canonical_root, display_name)
            .map(ProjectDependencyDto::from)
    }

    pub fn remove_project_dependency(
        &self,
        project_id: &str,
        dependency_id: &str,
    ) -> SdkResult<DependencyRemoval> {
        let removed = self
            .state
            .store
            .remove_project_dependency(project_id, dependency_id)?;
        if !removed {
            return Err(BusinessError::missing("dependency"));
        }
        Ok(DependencyRemoval {
            dependency_id: dependency_id.to_string(),
            removed,
        })
    }

    pub fn list_project_directory(
        &self,
        project_id: &str,
        dependency_id: Option<&str>,
        path: &str,
    ) -> SdkResult<Value> {
        let root = if let Some(dependency_id) = dependency_id {
            self.state
                .store
                .project_dependency_by_id(project_id, dependency_id)?
                .ok_or_else(|| BusinessError::missing("dependency"))?
                .canonical_root
        } else {
            self.state
                .store
                .project_by_id(project_id)?
                .ok_or_else(|| BusinessError::missing("project"))?
                .canonical_root
        };
        let mut value = self
            .state
            .operations
            .list_directory(Path::new(&root), path, 500)
            .map_err(operation_error)?;
        value["projectId"] = json!(project_id);
        value["dependencyId"] = dependency_id.map_or(Value::Null, |value| json!(value));
        Ok(value)
    }
}
