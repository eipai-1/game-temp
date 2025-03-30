use crate::item::{Item, ItemType};
use crate::realm;

pub struct Player {
    pub slected_hotbar: u8,
    pub hotbar: Vec<Item>,
    pub inventory: Vec<Item>,
}

impl Player {
    pub fn new() -> Self {
        let hotbar = vec![Item::new(ItemType::Empty); 9];
        let inventory = vec![Item::new(ItemType::Empty); 27];
        Self {
            slected_hotbar: 0,
            hotbar,
            inventory,
        }
    }
}
