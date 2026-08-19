#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum EguiCommand {
    Ping,
    HighlightMember(i64),
}
