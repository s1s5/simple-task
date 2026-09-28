use rmcp::ErrorData;
use rmcp::handler::server::tool::IntoCallToolResult;
use rmcp::model::CallToolResponse;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("設定エラー: {0}")]
    Config(String),
    #[error("タスクが見つかりません: {0}")]
    SlugNotFound(String),
    #[error("slug は既に登録済みです: {0}")]
    SlugExists(String),
    #[error("invalid slug: {0:?}（許可: ^[a-z0-9][a-z0-9._-]{{0,63}}$）")]
    InvalidSlug(String),
    #[error("MCP サーバーエラー: {0}")]
    Mcp(String),
    #[error("worker の実行に失敗しました: {0}")]
    Handoff(String),
    #[error("worker がタイムアウトしました（{0} 秒）")]
    Timeout(u64),
    #[error("IO エラー: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON エラー: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<Error> for ErrorData {
    fn from(value: Error) -> Self {
        match value {
            Error::SlugNotFound(msg) | Error::SlugExists(msg) | Error::InvalidSlug(msg) => {
                Self::invalid_params(msg, None)
            }
            other => Self::internal_error(other.to_string(), None),
        }
    }
}

impl IntoCallToolResult for Error {
    fn into_call_tool_result(self) -> std::result::Result<CallToolResponse, ErrorData> {
        Err(ErrorData::from(self))
    }
}
