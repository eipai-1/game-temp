use crate::realm::BlockInfo;

#[derive(Debug, Clone)]
pub enum ItemType {
    Block(BlockInfo),
    Empty,
}

impl ItemType {
    pub fn get_type(&self) -> u32 {
        match self {
            Self::Block(tp) => tp.block_type as u32,
            Self::Empty => 0,
        }
    }
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
