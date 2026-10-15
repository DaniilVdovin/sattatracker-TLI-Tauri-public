use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SlotState {
    pub item_id: String,
    pub qty: u32,
}

#[derive(Debug)]
pub enum SlotDelta {
    Gained(String, u32),
    Lost(String, u32),
    Swapped { new_item_id: String, new_qty: u32 },
    None,
}

pub struct BagManager {
    slots: HashMap<String, SlotState>,
}

impl BagManager {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    pub fn init_slot(&mut self, page_id: u32, slot_id: u32, item_id: &str, qty: u32) {
        let key = Self::key(page_id, slot_id);
        self.slots.insert(
            key,
            SlotState {
                item_id: item_id.to_string(),
                qty,
            },
        );
    }

    pub fn update_slot(
        &mut self,
        page_id: u32,
        slot_id: u32,
        item_id: &str,
        qty: u32,
    ) -> SlotDelta {
        let key = Self::key(page_id, slot_id);

        let delta = match self.slots.get(&key) {
            None => {
                if qty > 0 {
                    SlotDelta::Gained(item_id.to_string(), qty)
                } else {
                    SlotDelta::None
                }
            }
            Some(old) if old.item_id == item_id => {
                if qty > old.qty {
                    SlotDelta::Gained(item_id.to_string(), qty - old.qty)
                } else if qty < old.qty {
                    SlotDelta::Lost(item_id.to_string(), old.qty - qty)
                } else {
                    SlotDelta::None
                }
            }
            Some(_) => {
                if qty > 0 {
                    SlotDelta::Swapped {
                        new_item_id: item_id.to_string(),
                        new_qty: qty,
                    }
                } else {
                    SlotDelta::None
                }
            }
        };

        self.slots.insert(
            key,
            SlotState {
                item_id: item_id.to_string(),
                qty,
            },
        );
        delta
    }

    pub fn remove_slot(&mut self, page_id: u32, slot_id: u32) -> Option<SlotState> {
        let key = Self::key(page_id, slot_id);
        self.slots.remove(&key)
    }

    pub fn clear_page(&mut self, page_id: u32) {
        self.slots
            .retain(|k, _| !k.starts_with(&format!("{}:", page_id)));
    }

    pub fn reset(&mut self) {
        self.slots.clear();
    }

    fn key(page_id: u32, slot_id: u32) -> String {
        format!("{}:{}", page_id, slot_id)
    }

    pub fn get_all_main_inventory_items(&self) -> HashMap<String, u32> {
        let mut result: HashMap<String, u32> = HashMap::new();
        for (key, slot) in &self.slots {
            if slot.qty == 0 {
                continue;
            }
            let page_id: u32 = key
                .split(':')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            if Self::is_main_inventory(page_id) {
                *result.entry(slot.item_id.clone()).or_insert(0) += slot.qty;
            }
        }
        result
    }

    pub fn is_main_inventory(page_id: u32) -> bool {
        (100..=104).contains(&page_id)
    }
}
