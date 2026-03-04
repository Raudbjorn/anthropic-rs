use serde::{Deserialize, Serialize};

/// A citation within a text block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TextCitation {
    #[serde(rename = "char_location")]
    CharLocation {
        cited_text: String,
        document_index: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        document_title: Option<String>,
        start_char_index: u64,
        end_char_index: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    },
    #[serde(rename = "page_location")]
    PageLocation {
        cited_text: String,
        document_index: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        document_title: Option<String>,
        start_page_number: u64,
        end_page_number: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    },
    #[serde(rename = "content_block_location")]
    ContentBlockLocation {
        cited_text: String,
        document_index: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        document_title: Option<String>,
        start_block_index: u64,
        end_block_index: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    },
    #[serde(rename = "web_search_result_location")]
    WebSearchResultLocation {
        cited_text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        encrypted_index: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
}

/// Delta for streaming citation updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationsDelta {
    pub citation: TextCitation,
}
