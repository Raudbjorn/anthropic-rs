//! Client-side conversation item cache for the Realtime API.
//!
//! Mirrors the server's conversation state by processing item lifecycle
//! events (`conversation.item.created`, `conversation.item.deleted`,
//! `conversation.item.truncated`).

use std::collections::HashMap;

use super::types::{ConversationItem, ItemType};

/// Client-side cache of conversation items.
///
/// Mirrors the server's conversation state by processing server events.
/// Items are indexed by their ID for O(1) lookup.
#[derive(Debug, Clone, Default)]
pub struct ConversationState {
    /// Items in insertion order.
    items: Vec<ConversationItem>,
    /// Index from item ID to position in `items`.
    index: HashMap<String, usize>,
}

impl ConversationState {
    /// Create an empty conversation state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item to the end of the conversation.
    pub fn add_item(&mut self, item: ConversationItem) {
        self.add_item_after(item, None);
    }

    /// Add an item after the given `previous_item_id`, or at the end if `None`.
    ///
    /// This respects the ordering hint from `conversation.item.created` events.
    pub fn add_item_after(&mut self, item: ConversationItem, previous_item_id: Option<&str>) {
        let insert_pos = previous_item_id
            .and_then(|prev_id| self.index.get(prev_id).map(|&pos| pos + 1))
            .unwrap_or(self.items.len());

        if let Some(id) = item.id.as_ref() {
            self.index.insert(id.clone(), insert_pos);
        }

        if insert_pos >= self.items.len() {
            self.items.push(item);
        } else {
            self.items.insert(insert_pos, item);
            self.rebuild_index_from(insert_pos + 1);
        }
    }

    /// Remove an item by ID (from `conversation.item.deleted`).
    ///
    /// Returns the removed item if found.
    pub fn remove_item(&mut self, item_id: &str) -> Option<ConversationItem> {
        let &pos = self.index.get(item_id)?;
        self.index.remove(item_id);
        let item = self.items.remove(pos);
        // Rebuild index for items after the removed one
        self.rebuild_index_from(pos);
        Some(item)
    }

    /// Truncate an item's audio (from `conversation.item.truncated`).
    ///
    /// Clears the audio and transcript data for the specified content part.
    /// The `_audio_end_ms` parameter is intentionally ignored because this
    /// client-side cache lacks the audio format/sample rate information needed
    /// to compute byte offsets. As a conservative fallback, the entire audio
    /// and transcript are cleared — matching the OpenAI reference SDK behavior.
    pub fn truncate_item(&mut self, item_id: &str, content_index: u32, _audio_end_ms: u32) {
        if let Some(item) = self.get_item_mut(item_id) {
            if let Some(content) = item.content.as_mut() {
                if let Some(part) = content.get_mut(content_index as usize) {
                    part.audio = None;
                    part.transcript = None;
                }
            }
        }
    }

    /// Get an item by ID.
    pub fn get_item(&self, item_id: &str) -> Option<&ConversationItem> {
        self.index.get(item_id).and_then(|&pos| self.items.get(pos))
    }

    /// Get a mutable reference to an item by ID.
    pub fn get_item_mut(&mut self, item_id: &str) -> Option<&mut ConversationItem> {
        self.index
            .get(item_id)
            .copied()
            .and_then(|pos| self.items.get_mut(pos))
    }

    /// Get all items in order.
    pub fn items(&self) -> &[ConversationItem] {
        &self.items
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all items.
    pub fn clear(&mut self) {
        self.items.clear();
        self.index.clear();
    }

    /// Get the last function call item (useful for function calling flow).
    pub fn last_function_call(&self) -> Option<&ConversationItem> {
        self.items
            .iter()
            .rev()
            .find(|item| item.item_type == Some(ItemType::FunctionCall))
    }

    /// Rebuild the index for all items starting at `start`.
    fn rebuild_index_from(&mut self, start: usize) {
        for i in start..self.items.len() {
            if let Some(id) = self.items[i].id.as_ref() {
                self.index.insert(id.clone(), i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realtime::types::{ContentPart, ContentType, ItemStatus, Role};

    fn make_item(id: &str, item_type: ItemType) -> ConversationItem {
        ConversationItem {
            id: Some(id.into()),
            item_type: Some(item_type),
            ..Default::default()
        }
    }

    fn make_message(id: &str, role: Role) -> ConversationItem {
        ConversationItem {
            id: Some(id.into()),
            item_type: Some(ItemType::Message),
            role: Some(role),
            content: Some(vec![ContentPart::input_text("hello")]),
            ..Default::default()
        }
    }

    #[test]
    fn new_state_is_empty() {
        let state = ConversationState::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert!(state.items().is_empty());
    }

    #[test]
    fn add_and_get_item() {
        let mut state = ConversationState::new();
        let item = make_message("msg_1", Role::User);
        state.add_item(item);

        assert_eq!(state.len(), 1);
        assert!(!state.is_empty());

        let retrieved = state.get_item("msg_1").unwrap();
        assert_eq!(retrieved.id.as_deref(), Some("msg_1"));
        assert_eq!(retrieved.role, Some(Role::User));
    }

    #[test]
    fn add_multiple_items_preserves_order() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        state.add_item(make_message("msg_2", Role::Assistant));
        state.add_item(make_message("msg_3", Role::User));

        assert_eq!(state.len(), 3);
        let items = state.items();
        assert_eq!(items[0].id.as_deref(), Some("msg_1"));
        assert_eq!(items[1].id.as_deref(), Some("msg_2"));
        assert_eq!(items[2].id.as_deref(), Some("msg_3"));
    }

    #[test]
    fn remove_item_returns_removed() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        state.add_item(make_message("msg_2", Role::Assistant));
        state.add_item(make_message("msg_3", Role::User));

        let removed = state.remove_item("msg_2");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id.as_deref(), Some("msg_2"));

        assert_eq!(state.len(), 2);
        assert!(state.get_item("msg_2").is_none());
    }

    #[test]
    fn remove_item_rebuilds_index() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        state.add_item(make_message("msg_2", Role::Assistant));
        state.add_item(make_message("msg_3", Role::User));

        state.remove_item("msg_1");

        // Items after the removed one should still be accessible
        assert!(state.get_item("msg_2").is_some());
        assert!(state.get_item("msg_3").is_some());
        assert_eq!(state.items()[0].id.as_deref(), Some("msg_2"));
        assert_eq!(state.items()[1].id.as_deref(), Some("msg_3"));
    }

    #[test]
    fn remove_nonexistent_returns_none() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        assert!(state.remove_item("nonexistent").is_none());
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn truncate_item_clears_audio_and_transcript() {
        let mut state = ConversationState::new();
        let item = ConversationItem {
            id: Some("msg_1".into()),
            item_type: Some(ItemType::Message),
            role: Some(Role::Assistant),
            content: Some(vec![ContentPart {
                content_type: Some(ContentType::Audio),
                audio: Some("base64audiodata".into()),
                transcript: Some("Hello there".into()),
                ..Default::default()
            }]),
            ..Default::default()
        };
        state.add_item(item);

        state.truncate_item("msg_1", 0, 1500);

        let truncated = state.get_item("msg_1").unwrap();
        let part = &truncated.content.as_ref().unwrap()[0];
        assert!(part.audio.is_none());
        assert!(part.transcript.is_none());
    }

    #[test]
    fn truncate_nonexistent_item_is_noop() {
        let mut state = ConversationState::new();
        // Should not panic
        state.truncate_item("nonexistent", 0, 1000);
    }

    #[test]
    fn truncate_out_of_bounds_content_index_is_noop() {
        let mut state = ConversationState::new();
        let item = ConversationItem {
            id: Some("msg_1".into()),
            item_type: Some(ItemType::Message),
            content: Some(vec![ContentPart::input_text("hello")]),
            ..Default::default()
        };
        state.add_item(item);
        // content_index 5 is out of bounds, should not panic
        state.truncate_item("msg_1", 5, 1000);
    }

    #[test]
    fn get_item_mut_allows_modification() {
        let mut state = ConversationState::new();
        state.add_item(ConversationItem {
            id: Some("msg_1".into()),
            item_type: Some(ItemType::Message),
            status: Some(ItemStatus::InProgress),
            ..Default::default()
        });

        let item = state.get_item_mut("msg_1").unwrap();
        item.status = Some(ItemStatus::Completed);

        assert_eq!(
            state.get_item("msg_1").unwrap().status,
            Some(ItemStatus::Completed)
        );
    }

    #[test]
    fn clear_removes_all() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        state.add_item(make_message("msg_2", Role::Assistant));

        state.clear();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert!(state.get_item("msg_1").is_none());
    }

    #[test]
    fn last_function_call_finds_most_recent() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        state.add_item(make_item("fc_1", ItemType::FunctionCall));
        state.add_item(make_message("msg_2", Role::Assistant));
        state.add_item(make_item("fc_2", ItemType::FunctionCall));
        state.add_item(make_item("fco_1", ItemType::FunctionCallOutput));

        let last = state.last_function_call().unwrap();
        assert_eq!(last.id.as_deref(), Some("fc_2"));
    }

    #[test]
    fn last_function_call_returns_none_when_no_calls() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        assert!(state.last_function_call().is_none());
    }

    #[test]
    fn add_item_without_id() {
        let mut state = ConversationState::new();
        let item = ConversationItem {
            id: None,
            item_type: Some(ItemType::Message),
            ..Default::default()
        };
        state.add_item(item);
        assert_eq!(state.len(), 1);
        // Can't look up by ID since there is none
        assert!(state.items()[0].id.is_none());
    }

    #[test]
    fn default_is_same_as_new() {
        let state1 = ConversationState::new();
        let state2 = ConversationState::default();
        assert!(state1.is_empty());
        assert!(state2.is_empty());
    }

    #[test]
    fn add_item_after_inserts_at_correct_position() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        state.add_item(make_message("msg_3", Role::User));

        // Insert msg_2 after msg_1
        state.add_item_after(make_message("msg_2", Role::Assistant), Some("msg_1"));

        let items = state.items();
        assert_eq!(items[0].id.as_deref(), Some("msg_1"));
        assert_eq!(items[1].id.as_deref(), Some("msg_2"));
        assert_eq!(items[2].id.as_deref(), Some("msg_3"));

        // All lookups still work
        assert!(state.get_item("msg_1").is_some());
        assert!(state.get_item("msg_2").is_some());
        assert!(state.get_item("msg_3").is_some());
    }

    #[test]
    fn add_item_after_unknown_id_appends() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));

        // Unknown previous_item_id falls back to append
        state.add_item_after(make_message("msg_2", Role::Assistant), Some("nonexistent"));

        assert_eq!(state.len(), 2);
        assert_eq!(state.items()[1].id.as_deref(), Some("msg_2"));
    }

    #[test]
    fn add_item_after_none_appends() {
        let mut state = ConversationState::new();
        state.add_item(make_message("msg_1", Role::User));
        state.add_item_after(make_message("msg_2", Role::Assistant), None);

        assert_eq!(state.len(), 2);
        assert_eq!(state.items()[1].id.as_deref(), Some("msg_2"));
    }
}
