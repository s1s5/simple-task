use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::store;

#[derive(Debug, Clone)]
pub struct McpServer {
    config: Config,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateTaskInput {
    #[schemars(description = "呼び出し側が強制指定する一意キー。")]
    slug: String,
    #[schemars(description = "worker に渡す指示本文そのもの。整形しない。")]
    content: String,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct CreateTaskOutput {
    slug: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GetTaskInput {
    #[schemars(description = "取得対象の slug。")]
    slug: String,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct GetTaskOutput {
    slug: String,
    content: String,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct ListTasksOutput {
    slugs: Vec<String>,
}

impl McpServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[tool_router]
impl McpServer {
    #[tool(description = "タスク内容をファイルに登録する（実行はしない。即座に返る）。")]
    async fn create_task(
        &self,
        Parameters(input): Parameters<CreateTaskInput>,
    ) -> Result<Json<CreateTaskOutput>, Error> {
        let CreateTaskInput { slug, content } = input;
        store::write(&self.config.dir, &slug, &content)?;
        Ok(Json(CreateTaskOutput { slug }))
    }

    #[tool(description = "slug に対応する content を取得する。")]
    async fn get_task(
        &self,
        Parameters(input): Parameters<GetTaskInput>,
    ) -> Result<Json<GetTaskOutput>, Error> {
        Ok(Json(GetTaskOutput {
            slug: input.slug.clone(),
            content: store::read(&self.config.dir, &input.slug)?,
        }))
    }

    #[tool(description = "登録済みタスクの slug 一覧を取得する。")]
    async fn list_tasks(&self) -> Result<Json<ListTasksOutput>, Error> {
        Ok(Json(ListTasksOutput {
            slugs: store::list_slugs(&self.config.dir)?,
        }))
    }
}

#[tool_handler]
impl ServerHandler for McpServer {}

pub async fn run(config: Config) -> Result<()> {
    let server = McpServer::new(config);
    let (rt, wt) = rmcp::transport::stdio();
    let running = rmcp::serve_server(server, (rt, wt))
        .await
        .map_err(|e| Error::Mcp(e.to_string()))?;
    running
        .waiting()
        .await
        .map_err(|e| Error::Mcp(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_three_tools() {
        let tools = McpServer::tool_router().list_all();
        let names: Vec<_> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert_eq!(names.len(), 3, "expected 3 tools, got {names:?}");
        for expected in ["create_task", "get_task", "list_tasks"] {
            assert!(names.contains(&expected), "missing tool {expected}");
        }
    }

    #[test]
    fn every_tool_has_description_and_output_schema() {
        for tool in McpServer::tool_router().list_all() {
            assert!(
                tool.description.is_some(),
                "tool {} needs a description",
                tool.name
            );
            assert!(
                tool.output_schema.is_some(),
                "tool {} needs an output schema",
                tool.name
            );
        }
    }

    #[test]
    fn tool_schemas_expose_no_filesystem_paths() {
        for tool in McpServer::tool_router().list_all() {
            let schema = serde_json::to_string(&tool.input_schema).unwrap_or_default();
            let output = serde_json::to_string(&tool.output_schema).unwrap_or_default();
            for key in ["dir", "path", "file"] {
                assert!(
                    !schema.contains(key) && !output.contains(key),
                    "tool {} leaks {key}",
                    tool.name
                );
            }
        }
    }

    #[test]
    fn create_task_output_schema_is_slug_only() {
        let tools = McpServer::tool_router().list_all();
        let create = tools
            .iter()
            .find(|t| t.name.as_ref() == "create_task")
            .expect("create_task tool");
        let output = serde_json::to_string(&create.output_schema).unwrap_or_default();
        assert!(output.contains("slug"), "expected slug: {output}");
        assert!(!output.contains("result"), "result must be gone: {output}");
    }
}
