use crate::resources::Inventory;
use factory_content::{ContentDatabase, ItemId};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct RecipeRuntime {
  pub inputs: BTreeMap<ItemId, u32>,
  pub output_item: ItemId,
  pub output_quantity: u32,
  pub craft_time: u32,
}

#[derive(Clone, Debug)]
pub struct FactoryProduction {
  pub inventory: Inventory,
  pub recipe: RecipeRuntime,
  pub craft_progress: u32,
  pub crafting: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CraftSnapshot {
  pub inputs: BTreeMap<String, u32>,
  pub output_item: ItemId,
  pub output_quantity: u32,
  pub craft_time: u32,
  pub craft_progress: u32,
  pub crafting: bool,
}

impl FactoryProduction {
  pub fn new(inventory: Inventory, recipe: RecipeRuntime) -> Self {
    Self {
      inventory,
      recipe,
      craft_progress: 0,
      crafting: false,
    }
  }

  pub fn inputs_ready(&self) -> bool {
    self
      .recipe
      .inputs
      .iter()
      .all(|(item, quantity)| self.inventory.count(*item) >= *quantity)
  }

  pub fn wants_power(&self) -> bool {
    self.crafting || self.inputs_ready()
  }

  pub fn advance(&mut self, content: &ContentDatabase, events: &mut Vec<String>) -> u32 {
    let mut completed_this_tick = false;
    let mut produced_this_tick = 0;
    if self.crafting {
      self.craft_progress += 1;
      events.push(format!(
        "craft {} progress {}/{}",
        self.recipe.output_item, self.craft_progress, self.recipe.craft_time
      ));
      if self.craft_progress >= self.recipe.craft_time {
        let produced = self.inventory.insert_up_to(
          content,
          self.recipe.output_item,
          self.recipe.output_quantity,
        );
        self.crafting = false;
        self.craft_progress = 0;
        completed_this_tick = true;
        produced_this_tick = produced;
        events.push(format!(
          "craft {} completed produced {}",
          self.recipe.output_item, produced
        ));
      }
    }

    if !completed_this_tick && !self.crafting && self.inputs_ready() {
      let consumed_all = self
        .recipe
        .inputs
        .clone()
        .iter()
        .all(|(item, quantity)| self.inventory.remove_exact(*item, *quantity).is_ok());
      if consumed_all {
        self.crafting = true;
        self.craft_progress = 0;
        events.push(format!("craft {} started", self.recipe.output_item));
      }
    }
    produced_this_tick
  }

  pub fn craft_snapshot(&self) -> CraftSnapshot {
    CraftSnapshot {
      inputs: self
        .recipe
        .inputs
        .iter()
        .map(|(item, quantity)| (item.to_string(), *quantity))
        .collect(),
      output_item: self.recipe.output_item,
      output_quantity: self.recipe.output_quantity,
      craft_time: self.recipe.craft_time,
      craft_progress: self.craft_progress,
      crafting: self.crafting,
    }
  }
}
