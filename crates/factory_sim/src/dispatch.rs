use crate::NodeId;
use factory_content::ItemId;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct DispatchPriority(u8);

impl DispatchPriority {
  pub const LOW: Self = Self(64);
  pub const NORMAL: Self = Self(128);
  pub const HIGH: Self = Self(192);

  pub const fn new(value: u8) -> Self {
    Self(value)
  }

  pub const fn value(self) -> u8 {
    self.0
  }
}

impl Default for DispatchPriority {
  fn default() -> Self {
    Self::NORMAL
  }
}

impl fmt::Display for DispatchPriority {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DispatchPolicy {
  priorities: BTreeMap<(NodeId, ItemId), DispatchPriority>,
}

impl DispatchPolicy {
  pub fn priority(&self, destination: NodeId, item: ItemId) -> DispatchPriority {
    self
      .priorities
      .get(&(destination, item))
      .copied()
      .unwrap_or_default()
  }

  pub fn set_priority(
    &mut self,
    destination: NodeId,
    item: ItemId,
    priority: DispatchPriority,
  ) -> Option<DispatchPriority> {
    if priority == DispatchPriority::NORMAL {
      self.priorities.remove(&(destination, item))
    } else {
      self.priorities.insert((destination, item), priority)
    }
  }

  pub fn clear_priority(
    &mut self,
    destination: NodeId,
    item: ItemId,
  ) -> Option<DispatchPriority> {
    self.priorities.remove(&(destination, item))
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchVerb {
  Collect,
  Deliver,
  Retrieve,
  Deploy,
}

impl fmt::Display for DispatchVerb {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Collect => f.write_str("collect"),
      Self::Deliver => f.write_str("deliver"),
      Self::Retrieve => f.write_str("retrieve"),
      Self::Deploy => f.write_str("deploy"),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchPhase {
  Collect,
  Deliver,
  Retrieve,
  Deploy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchIntent {
  pub verb: DispatchVerb,
  pub item: ItemId,
  pub from: NodeId,
  pub to: NodeId,
  pub priority: DispatchPriority,
}

impl DispatchIntent {
  pub fn collect(item: ItemId, from: NodeId, to: NodeId) -> Self {
    Self {
      verb: DispatchVerb::Collect,
      item,
      from,
      to,
      priority: DispatchPriority::NORMAL,
    }
  }

  pub fn deliver(item: ItemId, from: NodeId, to: NodeId) -> Self {
    Self {
      verb: DispatchVerb::Deliver,
      item,
      from,
      to,
      priority: DispatchPriority::NORMAL,
    }
  }

  pub fn retrieve(item: ItemId, from: NodeId, to: NodeId) -> Self {
    Self {
      verb: DispatchVerb::Retrieve,
      item,
      from,
      to,
      priority: DispatchPriority::NORMAL,
    }
  }

  pub fn with_priority(mut self, priority: DispatchPriority) -> Self {
    self.priority = priority;
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchAssignment {
  pub item: ItemId,
  pub source: NodeId,
  pub destination: NodeId,
  pub phase: DispatchPhase,
  pub priority: DispatchPriority,
}

impl DispatchAssignment {
  pub fn collect(item: ItemId, source: NodeId, destination: NodeId) -> Self {
    Self {
      item,
      source,
      destination,
      phase: DispatchPhase::Collect,
      priority: DispatchPriority::NORMAL,
    }
  }

  pub fn collect_with_priority(
    item: ItemId,
    source: NodeId,
    destination: NodeId,
    priority: DispatchPriority,
  ) -> Self {
    Self {
      item,
      source,
      destination,
      phase: DispatchPhase::Collect,
      priority,
    }
  }

  pub fn retrieve(item: ItemId, source: NodeId, destination: NodeId) -> Self {
    Self {
      item,
      source,
      destination,
      phase: DispatchPhase::Retrieve,
      priority: DispatchPriority::NORMAL,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchBoard {
  pub intents: Vec<DispatchIntent>,
}

impl DispatchBoard {
  pub fn new() -> Self {
    Self { intents: Vec::new() }
  }
}

impl Default for DispatchBoard {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum DispatchReceiverState {
  Unassigned,
  Assigned(DispatchAssignment),
}

impl Default for DispatchReceiverState {
  fn default() -> Self {
    Self::Unassigned
  }
}
