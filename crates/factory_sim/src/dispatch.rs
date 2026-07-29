use crate::NodeId;
use factory_content::ItemId;
use serde::Serialize;
use std::fmt;

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
}

impl DispatchIntent {
  pub fn collect(item: ItemId, from: NodeId, to: NodeId) -> Self {
    Self {
      verb: DispatchVerb::Collect,
      item,
      from,
      to,
    }
  }

  pub fn deliver(item: ItemId, from: NodeId, to: NodeId) -> Self {
    Self {
      verb: DispatchVerb::Deliver,
      item,
      from,
      to,
    }
  }

  pub fn retrieve(item: ItemId, from: NodeId, to: NodeId) -> Self {
    Self {
      verb: DispatchVerb::Retrieve,
      item,
      from,
      to,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchAssignment {
  pub item: ItemId,
  pub source: NodeId,
  pub destination: NodeId,
  pub phase: DispatchPhase,
}

impl DispatchAssignment {
  pub fn collect(item: ItemId, source: NodeId, destination: NodeId) -> Self {
    Self {
      item,
      source,
      destination,
      phase: DispatchPhase::Collect,
    }
  }

  pub fn retrieve(item: ItemId, source: NodeId, destination: NodeId) -> Self {
    Self {
      item,
      source,
      destination,
      phase: DispatchPhase::Retrieve,
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
