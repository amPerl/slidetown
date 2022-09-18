use std::collections::HashMap;

use crate::parsers::xlt::{
    vehicle_list::{VehicleKind, VehicleList, VehicleListEntry},
    visual_item_list::{VisualItemCategory, VisualItemList, VisualItemListEntry},
    vshop_item_list::VShopItemList,
    Xlt,
};

pub struct InitConfiguration {
    pub vehicle_list: VehicleList,
    pub visual_item_list: VisualItemList,
    pub vshop_item_list: VShopItemList,
}

impl InitConfiguration {
    pub fn from_xlts(
        vehicle_list_xlt: &Xlt,
        visual_item_list_xlt: &Xlt,
        vshop_item_list_xlt: &Xlt,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            vehicle_list: VehicleList::from_xlt(vehicle_list_xlt)?,
            visual_item_list: VisualItemList::from_xlt(visual_item_list_xlt)?,
            vshop_item_list: VShopItemList::from_xlt(vshop_item_list_xlt)?,
        })
    }

    pub fn player_vehicles(&self) -> Vec<&VehicleListEntry> {
        self.vehicle_list
            .entries
            .iter()
            .filter(|vehicle| vehicle.kind == VehicleKind::PlayerCar)
            .collect()
    }

    pub fn vehicle_compatible_items(&self, vehicle_id: usize) -> Vec<&VisualItemListEntry> {
        if let Some((vehicle_index, _)) = self
            .visual_item_list
            .vehicle_definitions
            .ids
            .iter()
            .enumerate()
            .find(|(_idx, id)| **id == vehicle_id)
        {
            self.visual_item_list
                .entries
                .iter()
                .filter(|e| e.enabled_vehicle_id_indices.contains(&vehicle_index))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn vehicle_default_items(
        &self,
        vehicle_id: usize,
    ) -> HashMap<VisualItemCategory, &VisualItemListEntry> {
        let mut defaults = HashMap::new();

        for item in self.vehicle_compatible_items(vehicle_id) {
            if !self.is_default_visual_item(&item.item_id) {
                continue;
            }

            defaults.entry(item.category).or_insert(item);
        }

        defaults
    }

    pub fn is_default_visual_item(&self, visual_item_id: &str) -> bool {
        self.vshop_item_list
            .entries
            .iter()
            .find(|entry| entry.item_id == visual_item_id)
            .map(|e| e.default_part.trim())
            .unwrap_or_default()
            == "1"
    }
}
