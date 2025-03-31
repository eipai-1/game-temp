use crate::item::{Item, ItemType};
use crate::realm::{BlockInfo, BlockType};
use crate::ui::inventory_renderer;

pub struct Player {
    pub slected_hotbar: i32,
    pub hotbar: Vec<Item>,
    pub all_item_inventory: Vec<Vec<Item>>,
}

impl Player {
    pub fn new(all_block: &Vec<BlockInfo>) -> Self {
        let mut hotbar = vec![Item::new(ItemType::Empty); 10];
        hotbar[0] = Item::new(ItemType::Block(all_block[BlockType::Grass as usize]));
        hotbar[1] = Item::new(ItemType::Block(all_block[BlockType::Dirt as usize]));
        hotbar[2] = Item::new(ItemType::Block(all_block[BlockType::UnderStone as usize]));
        hotbar[3] = Item::new(ItemType::Block(all_block[BlockType::BirchLog as usize]));
        let mut all_item_inventory = vec![vec![Item::new(ItemType::Empty); 10]; 4];

        all_item_inventory[0][0] =
            Item::new(ItemType::Block(all_block[BlockType::UnderStone as usize]));
        all_item_inventory[0][1] =
            Item::new(ItemType::Block(all_block[BlockType::BirchLog as usize]));
        all_item_inventory[0][2] =
            Item::new(ItemType::Block(all_block[BlockType::BirchLeaves as usize]));
        all_item_inventory[0][3] = Item::new(ItemType::Block(all_block[BlockType::Grass as usize]));
        all_item_inventory[0][4] = Item::new(ItemType::Block(all_block[BlockType::Dirt as usize]));
        all_item_inventory[0][5] = Item::new(ItemType::Block(all_block[BlockType::Stone as usize]));
        all_item_inventory[0][6] =
            Item::new(ItemType::Block(all_block[BlockType::BirchPlank as usize]));

        Self {
            slected_hotbar: 0,
            hotbar,
            all_item_inventory,
        }
    }

    pub fn update_selected_hotbar(&mut self, offset: i32) {
        self.slected_hotbar += offset;
        self.slected_hotbar += inventory_renderer::SLOTS_PER_ROW as i32;
        self.slected_hotbar %= inventory_renderer::SLOTS_PER_ROW as i32;
    }
}
