use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::bash_tool::BashTool;
use super::cache_control::CacheControl;
use super::computer_use::ComputerUseTool;
use super::web_search::WebSearchTool;
use super::web_fetch::WebFetchTool;
use super::code_execution::CodeExecutionTool;
use super::text_editor::TextEditorTool;
use super::tool_search::ToolSearchTool;
use super::memory_tool::MemoryTool;

/// A user-defined tool with a JSON Schema input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub input_schema: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,
    /// Enforce strict JSON Schema validation on tool input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    /// Load tool definition on demand rather than eagerly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
    /// Which callers may invoke this tool (e.g. `"direct"`, `"code_execution_20260120"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<String>>,
    /// Stream tool input as it is generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eager_input_streaming: Option<bool>,
    /// Example inputs for the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_examples: Option<Vec<serde_json::Value>>,
}

impl Tool {
    pub fn new(name: impl Into<String>, input_schema: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            input_schema,
            description: None,
            cache_control: None,
            tool_type: None,
            strict: None,
            defer_loading: None,
            allowed_callers: None,
            eager_input_streaming: None,
            input_examples: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    pub fn with_defer_loading(mut self, defer: bool) -> Self {
        self.defer_loading = Some(defer);
        self
    }

    pub fn with_allowed_callers(mut self, callers: Vec<String>) -> Self {
        self.allowed_callers = Some(callers);
        self
    }

    pub fn with_eager_input_streaming(mut self, eager: bool) -> Self {
        self.eager_input_streaming = Some(eager);
        self
    }

    pub fn with_input_examples(mut self, examples: Vec<serde_json::Value>) -> Self {
        self.input_examples = Some(examples);
        self
    }
}

/// Union of all tool types that can be passed to the API.
///
/// Server tools use versioned type strings (e.g. `web_search_20250305`),
/// so we use a custom deserializer instead of `#[serde(untagged)]` which
/// is fragile with overlapping shapes.
#[derive(Debug, Clone)]
pub enum ToolUnion {
    /// User-defined tool with JSON Schema.
    Custom(Tool),
    /// Web search server tool.
    WebSearch(WebSearchTool),
    /// Web fetch server tool.
    WebFetch(WebFetchTool),
    /// Code execution server tool.
    CodeExecution(CodeExecutionTool),
    /// Bash server tool.
    Bash(BashTool),
    /// Text editor server tool.
    TextEditor(TextEditorTool),
    /// Computer use server tool.
    ComputerUse(ComputerUseTool),
    /// Tool search server tool.
    ToolSearch(ToolSearchTool),
    /// Memory server tool.
    Memory(MemoryTool),
}

impl Serialize for ToolUnion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Custom(t) => t.serialize(serializer),
            Self::WebSearch(t) => t.serialize(serializer),
            Self::WebFetch(t) => t.serialize(serializer),
            Self::CodeExecution(t) => t.serialize(serializer),
            Self::Bash(t) => t.serialize(serializer),
            Self::TextEditor(t) => t.serialize(serializer),
            Self::ComputerUse(t) => t.serialize(serializer),
            Self::ToolSearch(t) => t.serialize(serializer),
            Self::Memory(t) => t.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ToolUnion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let type_str = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match type_str {
            s if s.starts_with("web_search_") => {
                let t: WebSearchTool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::WebSearch(t))
            }
            s if s.starts_with("web_fetch_") => {
                let t: WebFetchTool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::WebFetch(t))
            }
            s if s.starts_with("code_execution_") => {
                let t: CodeExecutionTool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::CodeExecution(t))
            }
            s if s.starts_with("bash_") => {
                let t: BashTool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Bash(t))
            }
            s if s.starts_with("text_editor_") => {
                let t: TextEditorTool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::TextEditor(t))
            }
            s if s.starts_with("computer_") => {
                let t: ComputerUseTool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::ComputerUse(t))
            }
            s if s.starts_with("tool_search_tool_") => {
                let t: ToolSearchTool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::ToolSearch(t))
            }
            s if s.starts_with("memory_") => {
                let t: MemoryTool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Memory(t))
            }
            _ => {
                // Default: user-defined custom tool
                let t: Tool = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Custom(t))
            }
        }
    }
}

// Convenient From impls
impl From<Tool> for ToolUnion {
    fn from(t: Tool) -> Self { Self::Custom(t) }
}
impl From<WebSearchTool> for ToolUnion {
    fn from(t: WebSearchTool) -> Self { Self::WebSearch(t) }
}
impl From<WebFetchTool> for ToolUnion {
    fn from(t: WebFetchTool) -> Self { Self::WebFetch(t) }
}
impl From<CodeExecutionTool> for ToolUnion {
    fn from(t: CodeExecutionTool) -> Self { Self::CodeExecution(t) }
}
impl From<BashTool> for ToolUnion {
    fn from(t: BashTool) -> Self { Self::Bash(t) }
}
impl From<TextEditorTool> for ToolUnion {
    fn from(t: TextEditorTool) -> Self { Self::TextEditor(t) }
}
impl From<ComputerUseTool> for ToolUnion {
    fn from(t: ComputerUseTool) -> Self { Self::ComputerUse(t) }
}
impl From<ToolSearchTool> for ToolUnion {
    fn from(t: ToolSearchTool) -> Self { Self::ToolSearch(t) }
}
impl From<MemoryTool> for ToolUnion {
    fn from(t: MemoryTool) -> Self { Self::Memory(t) }
}
