use super::draggable_card::CardTransform;
/// Global registry for DraggableCard transforms.
///
/// Each DraggableCard registers its transform with a unique UUID.
/// Child charts can look up their parent card's transform by ID.
use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Global storage of card transforms by UUID.
/// This provides a reliable way for child components to access their parent card's dimensions
/// without relying on `provide_context`, which has isolation issues between sibling instances.
pub static CARD_TRANSFORMS: LazyLock<RwSignal<HashMap<String, CardTransform>>> =
    LazyLock::new(|| RwSignal::new(HashMap::new()));

/// Register a card's transform in the global registry.
pub fn register_card_transform(id: &str, transform: CardTransform) {
    CARD_TRANSFORMS.update(|transforms| {
        transforms.insert(id.to_string(), transform);
    });
}

/// Update a card's transform in the global registry.
pub fn update_card_transform(id: &str, transform: CardTransform) {
    register_card_transform(id, transform); // Same operation
}

/// Look up a card's transform by ID.
pub fn get_card_transform(id: &str) -> Option<CardTransform> {
    CARD_TRANSFORMS.with(|transforms| transforms.get(id).copied())
}

/// Remove a card's transform from the registry (cleanup).
pub fn unregister_card_transform(id: &str) {
    CARD_TRANSFORMS.update(|transforms| {
        transforms.remove(id);
    });
}

/// Get a reactive Signal for a card's transform by ID.
/// Returns None if the card ID is not registered.
pub fn get_card_transform_signal(id: String) -> Signal<Option<CardTransform>> {
    Signal::derive(move || CARD_TRANSFORMS.with(|transforms| transforms.get(&id).copied()))
}
