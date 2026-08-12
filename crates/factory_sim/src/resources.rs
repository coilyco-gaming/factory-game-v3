use factory_content::{ContentDatabase, ItemId};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InventoryError {
  UnknownItem(ItemId),
  CapacityExceeded {
    item: ItemId,
    requested: u32,
    accepted: u32,
  },
  InsufficientQuantity {
    item: ItemId,
    requested: u32,
    available: u32,
  },
  ReservationBlocked(ItemId),
}

impl fmt::Display for InventoryError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownItem(id) => write!(f, "unknown item: {id}"),
      Self::CapacityExceeded {
        item,
        requested,
        accepted,
      } => write!(
        f,
        "capacity exceeded for {item}: requested {requested}, accepted {accepted}"
      ),
      Self::InsufficientQuantity {
        item,
        requested,
        available,
      } => write!(
        f,
        "insufficient quantity for {item}: requested {requested}, available {available}"
      ),
      Self::ReservationBlocked(item) => write!(f, "reservation blocks {item}"),
    }
  }
}

impl std::error::Error for InventoryError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
  items: BTreeMap<ItemId, u32>,
  weight_capacity: u32,
  volume_capacity: u32,
  reserved_capacity: BTreeMap<ItemId, u32>,
}

impl Inventory {
  pub fn new(weight_capacity: u32, volume_capacity: u32) -> Self {
    Self {
      items: BTreeMap::new(),
      weight_capacity,
      volume_capacity,
      reserved_capacity: BTreeMap::new(),
    }
  }

  pub fn with_reserved_capacity(mut self, item: ItemId, quantity: u32) -> Self {
    self.reserved_capacity.insert(item, quantity);
    self
  }

  pub fn reserve(&mut self, item: ItemId, quantity: u32) {
    self.reserved_capacity.insert(item, quantity);
  }

  pub fn count(&self, item: ItemId) -> u32 {
    self.items.get(&item).copied().unwrap_or(0)
  }

  pub fn is_empty(&self) -> bool {
    self.items.values().all(|quantity| *quantity == 0)
  }

  fn item_allowed(&self, item: ItemId) -> bool {
    self.reserved_capacity.is_empty() || self.reserved_capacity.contains_key(&item)
  }

  fn used_weight(&self, content: &ContentDatabase) -> u32 {
    self
      .items
      .iter()
      .map(|(item, quantity)| content.item(*item).weight.saturating_mul(*quantity))
      .sum()
  }

  fn used_volume(&self, content: &ContentDatabase) -> u32 {
    self
      .items
      .iter()
      .map(|(item, quantity)| content.item(*item).volume.saturating_mul(*quantity))
      .sum()
  }

  pub fn remaining_weight(&self, content: &ContentDatabase) -> u32 {
    self
      .weight_capacity
      .saturating_sub(self.used_weight(content))
  }

  pub fn remaining_volume(&self, content: &ContentDatabase) -> u32 {
    self
      .volume_capacity
      .saturating_sub(self.used_volume(content))
  }

  pub fn max_insertable(&self, content: &ContentDatabase, item: ItemId, requested: u32) -> u32 {
    if requested == 0 || !self.item_allowed(item) {
      return 0;
    }
    let definition = content.item(item);
    let weight_limit = if definition.weight == 0 {
      requested
    } else {
      self.remaining_weight(content) / definition.weight
    };
    let volume_limit = if definition.volume == 0 {
      requested
    } else {
      self.remaining_volume(content) / definition.volume
    };
    requested.min(weight_limit.min(volume_limit))
  }

  pub fn insert_up_to(&mut self, content: &ContentDatabase, item: ItemId, requested: u32) -> u32 {
    let accepted = self.max_insertable(content, item, requested);
    if accepted == 0 {
      return 0;
    }
    *self.items.entry(item).or_insert(0) += accepted;
    accepted
  }

  pub fn insert_exact(
    &mut self,
    content: &ContentDatabase,
    item: ItemId,
    quantity: u32,
  ) -> Result<(), InventoryError> {
    let accepted = self.insert_up_to(content, item, quantity);
    if accepted == quantity {
      Ok(())
    } else if !self.item_allowed(item) {
      Err(InventoryError::ReservationBlocked(item))
    } else {
      Err(InventoryError::CapacityExceeded {
        item,
        requested: quantity,
        accepted,
      })
    }
  }

  pub fn force_insert(&mut self, item: ItemId, quantity: u32) {
    if quantity > 0 {
      *self.items.entry(item).or_insert(0) += quantity;
    }
  }

  pub fn remove_up_to(&mut self, item: ItemId, requested: u32) -> u32 {
    let available = self.count(item);
    let removed = requested.min(available);
    if removed == 0 {
      return 0;
    }
    let remaining = available - removed;
    if remaining == 0 {
      self.items.remove(&item);
    } else {
      self.items.insert(item, remaining);
    }
    removed
  }

  pub fn remove_exact(&mut self, item: ItemId, quantity: u32) -> Result<(), InventoryError> {
    let available = self.count(item);
    if available < quantity {
      return Err(InventoryError::InsufficientQuantity {
        item,
        requested: quantity,
        available,
      });
    }
    self.remove_up_to(item, quantity);
    Ok(())
  }

  pub fn transfer_up_to(
    &mut self,
    content: &ContentDatabase,
    target: &mut Inventory,
    item: ItemId,
    requested: u32,
  ) -> u32 {
    let available = self.count(item);
    let max_target = target.max_insertable(content, item, requested.min(available));
    if max_target == 0 {
      return 0;
    }
    self.remove_up_to(item, max_target);
    target.insert_up_to(content, item, max_target)
  }

  pub fn snapshot(&self) -> InventorySnapshot {
    InventorySnapshot {
      items: self
        .items
        .iter()
        .map(|(item, quantity)| (item.to_string(), *quantity))
        .collect(),
      reserved_capacity: self
        .reserved_capacity
        .iter()
        .map(|(item, quantity)| (item.to_string(), *quantity))
        .collect(),
      weight_capacity: self.weight_capacity,
      volume_capacity: self.volume_capacity,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InventorySnapshot {
  pub items: BTreeMap<String, u32>,
  pub reserved_capacity: BTreeMap<String, u32>,
  pub weight_capacity: u32,
  pub volume_capacity: u32,
}
