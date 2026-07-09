use crate::resources::Inventory;
use factory_content::{ContentDatabase, ItemId};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct RecipeRuntime {
  pub input_item: ItemId,
  pub input_quantity: u32,
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
  pub input_item: ItemId,
  pub output_item: ItemId,
  pub input_quantity: u32,
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

  pub fn advance(&mut self, content: &ContentDatabase, events: &mut Vec<String>) {
    let mut completed_this_tick = false;
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
        events.push(format!(
          "craft {} completed produced {}",
          self.recipe.output_item, produced
        ));
      }
    }

    if !completed_this_tick
      && !self.crafting
      && self.inventory.count(self.recipe.input_item) >= self.recipe.input_quantity
    {
      if self
        .inventory
        .remove_exact(self.recipe.input_item, self.recipe.input_quantity)
        .is_ok()
      {
        self.crafting = true;
        self.craft_progress = 0;
        events.push(format!("craft {} started", self.recipe.output_item));
      }
    }
  }

  pub fn craft_snapshot(&self) -> CraftSnapshot {
    CraftSnapshot {
      input_item: self.recipe.input_item,
      output_item: self.recipe.output_item,
      input_quantity: self.recipe.input_quantity,
      output_quantity: self.recipe.output_quantity,
      craft_time: self.recipe.craft_time,
      craft_progress: self.craft_progress,
      crafting: self.crafting,
    }
  }
}
