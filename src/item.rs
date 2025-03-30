use crate::realm::BlockInfo;

#[derive(Debug, Clone)]
pub enum ItemType {
    Block(BlockInfo),
    Empty,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub item_type: ItemType,
}

impl Item {
    pub fn new(item_type: ItemType) -> Self {
        Self { item_type }
    }
}
