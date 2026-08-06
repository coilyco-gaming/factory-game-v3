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
  pub output_buffer: u32,
  pub craft_progress: u32,
  pub crafting: bool,
  pub blocked: Option<ProductionBlockReason>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionBlockReason {
  OutputFull,
  NoOutputSpace,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CraftSnapshot {
  pub inputs: BTreeMap<String, u32>,
  pub output_item: ItemId,
  pub output_quantity: u32,
  pub output_buffer: u32,
  pub craft_time: u32,
  pub craft_progress: u32,
  pub crafting: bool,
  pub blocked: Option<ProductionBlockReason>,
}

impl FactoryProduction {
  pub fn new(inventory: Inventory, recipe: RecipeRuntime, output_buffer: u32) -> Self {
    Self {
      inventory,
      recipe,
      output_buffer,
      craft_progress: 0,
      crafting: false,
      blocked: None,
    }
  }

  pub fn inputs_ready(&self) -> bool {
    self
      .recipe
      .inputs
      .iter()
      .all(|(item, quantity)| self.inventory.count(*item) >= *quantity)
  }

  pub fn can_start(&self, content: &ContentDatabase) -> bool {
    self.inputs_ready()
      && self.inventory.count(self.recipe.output_item) < self.output_buffer
      && self
        .inventory
        .max_insertable(content, self.recipe.output_item, 1)
        > 0
  }

  pub fn wants_power(&self, content: &ContentDatabase) -> bool {
    self.crafting || self.can_start(content)
  }

  pub fn advance(&mut self, content: &ContentDatabase, events: &mut Vec<String>) -> u32 {
    self.blocked = None;
    if self.crafting {
      self.craft_progress += 1;
      events.push(format!(
        "craft {} progress {}/{}",
        self.recipe.output_item, self.craft_progress, self.recipe.craft_time
      ));
      if self.craft_progress >= self.recipe.craft_time {
        return self.complete(events);
      }
      return 0;
    }

    if !self.inputs_ready() {
      return 0;
    }
    if self.inventory.count(self.recipe.output_item) >= self.output_buffer {
      self.blocked = Some(ProductionBlockReason::OutputFull);
      return 0;
    }
    if self
      .inventory
      .max_insertable(content, self.recipe.output_item, 1)
      == 0
    {
      self.blocked = Some(ProductionBlockReason::NoOutputSpace);
      return 0;
    }

    for (item, quantity) in &self.recipe.inputs {
      self
        .inventory
        .remove_exact(*item, *quantity)
        .expect("inputs_ready proves every recipe input is available");
    }
    self.crafting = true;
    self.craft_progress = 1;
    events.push(format!("craft {} started", self.recipe.output_item));
    if self.craft_progress >= self.recipe.craft_time {
      self.complete(events)
    } else {
      0
    }
  }

  fn complete(&mut self, events: &mut Vec<String>) -> u32 {
    let produced = self.recipe.output_quantity;
    self
      .inventory
      .force_insert(self.recipe.output_item, produced);
    self.crafting = false;
    self.craft_progress = 0;
    events.push(format!(
      "craft {} completed produced {}",
      self.recipe.output_item, produced
    ));
    produced
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
      output_buffer: self.output_buffer,
      craft_time: self.recipe.craft_time,
      craft_progress: self.craft_progress,
      crafting: self.crafting,
      blocked: self.blocked,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use factory_content::{BUILDING_MATERIALS, IRON_BARS, IRON_ORE, STONE};

  fn stocked_production(
    content: &ContentDatabase,
    output_item: ItemId,
    output_buffer: u32,
    capacity: u32,
  ) -> FactoryProduction {
    let item = content.item(output_item);
    let mut inventory = Inventory::new(capacity, capacity);
    for (input, quantity) in &item.ingredients {
      inventory.reserve(*input, *quantity);
      inventory
        .insert_exact(content, *input, *quantity)
        .expect("test inventory holds the recipe inputs");
    }
    inventory.reserve(output_item, output_buffer);
    FactoryProduction::new(
      inventory,
      RecipeRuntime {
        inputs: item.ingredients.clone(),
        output_item,
        output_quantity: item.craft_output,
        craft_time: item.craft_time,
      },
      output_buffer,
    )
  }

  #[test]
  fn craft_time_one_completes_on_the_start_tick() {
    let content = ContentDatabase::starter();
    let mut production = stocked_production(&content, IRON_BARS, 20, 500);
    let mut events = Vec::new();

    assert_eq!(10, production.advance(&content, &mut events));
    assert_eq!(0, production.inventory.count(IRON_ORE));
    assert_eq!(10, production.inventory.count(IRON_BARS));
    assert!(!production.crafting);
    assert_eq!(0, production.craft_progress);
  }

  #[test]
  fn a_long_craft_counts_its_start_tick_as_progress() {
    let content = ContentDatabase::starter();
    let mut production = stocked_production(&content, BUILDING_MATERIALS, 20, 500);

    assert_eq!(0, production.advance(&content, &mut Vec::new()));
    assert!(production.crafting);
    assert_eq!(1, production.craft_progress);
  }

  #[test]
  fn a_full_output_buffer_preserves_recipe_inputs() {
    let content = ContentDatabase::starter();
    let mut production = stocked_production(&content, IRON_BARS, 20, 500);
    production.inventory.force_insert(IRON_BARS, 20);
    assert_eq!(0, production.advance(&content, &mut Vec::new()));
    assert_eq!(3, production.inventory.count(IRON_ORE));
    assert!(!production.crafting);
    assert_eq!(Some(ProductionBlockReason::OutputFull), production.blocked);
  }

  #[test]
  fn insufficient_output_space_preserves_recipe_inputs() {
    let content = ContentDatabase::starter();
    let mut production = stocked_production(&content, BUILDING_MATERIALS, 20, 8);
    assert_eq!(0, production.advance(&content, &mut Vec::new()));
    assert_eq!(4, production.inventory.count(IRON_BARS));
    assert_eq!(4, production.inventory.count(STONE));
    assert!(!production.crafting);
    assert_eq!(
      Some(ProductionBlockReason::NoOutputSpace),
      production.blocked
    );
  }
}
