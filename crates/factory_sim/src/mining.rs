use crate::resources::Inventory;
use factory_content::{ContentDatabase, ItemId};
use serde::Serialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "remaining")]
pub enum Deposit {
  Finite(u32),
  Manifest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MiningExtractor {
  pub item: ItemId,
  pub speed: u32,
  pub deposit: Deposit,
}

impl MiningExtractor {
  pub fn for_item(content: &ContentDatabase, item: ItemId, speed: u32, deposit_size: u32) -> Self {
    let deposit = if content.item(item).create_from_nothing {
      Deposit::Manifest
    } else {
      Deposit::Finite(deposit_size)
    };
    Self {
      item,
      speed,
      deposit,
    }
  }

  pub fn advance(&mut self, content: &ContentDatabase, stockpile: &mut Inventory) -> u32 {
    let available = match self.deposit {
      Deposit::Finite(remaining) => remaining.min(self.speed),
      Deposit::Manifest => self.speed,
    };
    if available == 0 {
      return 0;
    }
    let mined = stockpile.insert_up_to(content, self.item, available);
    if let Deposit::Finite(remaining) = &mut self.deposit {
      *remaining -= mined;
    }
    mined
  }

  pub fn is_depleted(&self) -> bool {
    matches!(self.deposit, Deposit::Finite(0))
  }
}
