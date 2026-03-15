use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const CART_STORAGE_KEY: &str = "yame_cart";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CartItem {
    pub variant_id: Uuid,
    pub product_name: String,
    pub variant_name: String,
    pub unit_price: i64,
    pub quantity: i32,
}

impl CartItem {
    pub fn line_total(&self) -> i64 {
        self.unit_price * self.quantity as i64
    }
}

/// Global cart state with LocalStorage persistence.
#[derive(Clone, Copy)]
pub struct CartState {
    pub items: RwSignal<Vec<CartItem>>,
}

impl CartState {
    pub fn new() -> Self {
        let stored: Vec<CartItem> = LocalStorage::get(CART_STORAGE_KEY).unwrap_or_default();
        let items = RwSignal::new(stored);
        Self { items }
    }

    fn persist(&self) {
        let current = self.items.get_untracked();
        let _ = LocalStorage::set(CART_STORAGE_KEY, &current);
    }

    pub fn add_item(&self, item: CartItem) {
        self.items.update(|items| {
            if let Some(existing) = items.iter_mut().find(|i| i.variant_id == item.variant_id) {
                existing.quantity += item.quantity;
            } else {
                items.push(item);
            }
        });
        self.persist();
    }

    pub fn remove_item(&self, variant_id: Uuid) {
        self.items.update(|items| {
            items.retain(|i| i.variant_id != variant_id);
        });
        self.persist();
    }

    pub fn update_quantity(&self, variant_id: Uuid, quantity: i32) {
        if quantity <= 0 {
            self.remove_item(variant_id);
            return;
        }
        self.items.update(|items| {
            if let Some(item) = items.iter_mut().find(|i| i.variant_id == variant_id) {
                item.quantity = quantity;
            }
        });
        self.persist();
    }

    pub fn clear(&self) {
        self.items.set(vec![]);
        self.persist();
    }

    pub fn total_items(&self) -> usize {
        self.items
            .get_untracked()
            .iter()
            .map(|i| i.quantity as usize)
            .sum()
    }

    pub fn total_price(&self) -> i64 {
        self.items
            .get_untracked()
            .iter()
            .map(|i| i.line_total())
            .sum()
    }
}
