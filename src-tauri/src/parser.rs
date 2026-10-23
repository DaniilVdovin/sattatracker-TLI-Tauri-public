use crate::models::ParsedEvent;
use regex::Regex;

pub struct LogParser {
    level_regex: Regex,
    scene_regex: Regex,

    init_bag_regex: Regex,
    modify_bag_regex: Regex,
    remove_bag_regex: Regex,
    item_add_regex: Regex,

    proto_start_regex: Regex,
    proto_end_regex: Regex,

    ui_page_regex: Regex,

    xchg_id_regex: Regex,
    xchg_price_regex: Regex,
    pending_xchg_item: Option<String>,
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            level_regex:       Regex::new(r"LevelMgr@ LevelUid, LevelType, LevelId = (\d+) (\d+) (\d+)").unwrap(),
            scene_regex:       Regex::new(r"PageApplyBase@ _UpdateGameEnd:.*NextSceneName = (\S+)").unwrap(),

            init_bag_regex:    Regex::new(r"BagMgr@:InitBagData\s+PageId\s*=\s*(\d+)\s+SlotId\s*=\s*(\d+)\s+ConfigBaseId\s*=\s*(\d+)\s+Num\s*=\s*(\d+)").unwrap(),
            modify_bag_regex:  Regex::new(r"BagMgr@:Modfy BagItem\s+PageId\s*=\s*(\d+)\s+SlotId\s*=\s*(\d+)\s+ConfigBaseId\s*=\s*(\d+)\s+Num\s*=\s*(\d+)").unwrap(),
            remove_bag_regex:  Regex::new(r"BagMgr@:RemoveBagItem\s+PageId\s*=\s*(\d+)\s+SlotId\s*=\s*(\d+)").unwrap(),
            item_add_regex:    Regex::new(r"ItemChange@ Add Id=(\d+)[\d_a-z-]* BagNum=(\d+)\s+(?:in\s+)?PageId=(-?\d+) SlotId=(-?\d+)").unwrap(),

            proto_start_regex: Regex::new(r"ItemChange@ ProtoName=(\w+) start").unwrap(),
            proto_end_regex:   Regex::new(r"ItemChange@ ProtoName=(\w+) end").unwrap(),

            ui_page_regex:     Regex::new(r"TipMsgShowMgr@DispatchPageRunChange PageName = (\w+) , PageRunState = (\w+)").unwrap(),

            xchg_id_regex:     Regex::new(r"\+refer\s*\[(\d+)\]").unwrap(),
            xchg_price_regex:  Regex::new(r"\+(?:unitPrices\+)?\d+\s*\[([0-9.]+)\]").unwrap(),

            pending_xchg_item: None,
        }
    }

    pub fn parse_line(&mut self, line: &str) -> Option<ParsedEvent> {
        if line.contains("LevelMgr@ LevelUid, LevelType, LevelId") {
            if let Some(caps) = self.level_regex.captures(line) {
                let level_uid: u32 = caps[1].parse().unwrap_or(0);
                let level_type: u32 = caps[2].parse().unwrap_or(0);
                let level_id: u32 = caps[3].parse().unwrap_or(0);
                return Some(ParsedEvent::LevelChange {
                    level_uid,
                    level_type,
                    level_id,
                });
            }
        }

        if line.contains("PageApplyBase@ _UpdateGameEnd") {
            if let Some(caps) = self.scene_regex.captures(line) {
                let scene_name = caps[1].to_string();
                return Some(ParsedEvent::SceneChange { scene_name });
            }
        }

        if line.contains("ItemChange@ ProtoName=") && line.contains(" start") {
            if let Some(caps) = self.proto_start_regex.captures(line) {
                return Some(ParsedEvent::ProtoStart {
                    proto_name: caps[1].to_string(),
                });
            }
        }

        if line.contains("ItemChange@ ProtoName=") && line.contains(" end") {
            if let Some(caps) = self.proto_end_regex.captures(line) {
                return Some(ParsedEvent::ProtoEnd {
                    proto_name: caps[1].to_string(),
                });
            }
        }

        if line.contains("ItemChange@ Add ") {
            if let Some(caps) = self.item_add_regex.captures(line) {
                let item_id = caps[1].to_string();
                let qty: u32 = caps[2].parse().unwrap_or(0);
                let page_id: i32 = caps[3].parse().unwrap_or(-1);
                let slot_id: i32 = caps[4].parse().unwrap_or(-1);
                if qty > 0 {
                    return Some(ParsedEvent::ItemAdd {
                        item_id,
                        page_id,
                        slot_id,
                        qty,
                    });
                }
            }
        }

        if line.contains("BagMgr@:InitBagData") {
            if let Some(caps) = self.init_bag_regex.captures(line) {
                let page_id: u32 = caps[1].parse().unwrap_or(0);
                let slot_id: u32 = caps[2].parse().unwrap_or(0);
                let item_id = caps[3].to_string();
                let qty: u32 = caps[4].parse().unwrap_or(0);
                return Some(ParsedEvent::BagInit {
                    page_id,
                    slot_id,
                    item_id,
                    qty,
                });
            }
        }

        if line.contains("BagMgr@:Modfy BagItem") {
            if let Some(caps) = self.modify_bag_regex.captures(line) {
                let page_id: u32 = caps[1].parse().unwrap_or(0);
                let slot_id: u32 = caps[2].parse().unwrap_or(0);
                let item_id = caps[3].to_string();
                let qty: u32 = caps[4].parse().unwrap_or(0);
                return Some(ParsedEvent::BagModify {
                    page_id,
                    slot_id,
                    item_id,
                    qty,
                });
            }
        }

        if line.contains("BagMgr@:RemoveBagItem") {
            if let Some(caps) = self.remove_bag_regex.captures(line) {
                let page_id: u32 = caps[1].parse().unwrap_or(0);
                let slot_id: u32 = caps[2].parse().unwrap_or(0);
                return Some(ParsedEvent::BagRemove { page_id, slot_id });
            }
        }

        if line.contains("TipMsgShowMgr@DispatchPageRunChange") {
            if let Some(caps) = self.ui_page_regex.captures(line) {
                let page_name = caps[1].to_string();
                let state = caps[2].to_string();
                if state == "Run" {
                    return Some(ParsedEvent::UiPageChange { page_name, state });
                }
            }
        }

        if line.contains("XchgSearchPrice")
            || line.contains("+refer")
            || line.contains("unitPrices")
        {
            if let Some(id_caps) = self.xchg_id_regex.captures(line) {
                self.pending_xchg_item = Some(id_caps[1].to_string());
            }
            if let Some(price_caps) = self.xchg_price_regex.captures(line) {
                if let Ok(price) = price_caps[1].parse::<f64>() {
                    if let Some(item_id) = self.pending_xchg_item.take() {
                        crate::pricing::PricingManager::update_local_price(&item_id, price);
                        return Some(ParsedEvent::PriceUpdate { item_id, price });
                    }
                }
            }
        }

        None
    }
}
