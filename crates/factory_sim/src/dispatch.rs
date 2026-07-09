use crate::Location;
use factory_content::ItemId;
use serde::Serialize;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchVerb {
  Collect,
  Deliver,
}

impl fmt::Display for DispatchVerb {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Collect => f.write_str("collect"),
      Self::Deliver => f.write_str("deliver"),
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchPhase {
  Collect,
  Deliver,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchIntent {
  pub verb: DispatchVerb,
  pub item: ItemId,
  pub from: Location,
  pub to: Location,
}

impl DispatchIntent {
  pub fn collect(item: ItemId, from: Location, to: Location) -> Self {
    Self {
      verb: DispatchVerb::Collect,
      item,
      from,
      to,
    }
  }

  pub fn deliver(item: ItemId, from: Location, to: Location) -> Self {
    Self {
      verb: DispatchVerb::Deliver,
      item,
      from,
      to,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchAssignment {
  pub item: ItemId,
  pub source: Location,
  pub destination: Location,
  pub phase: DispatchPhase,
}

impl DispatchAssignment {
  pub fn collect(item: ItemId, source: Location, destination: Location) -> Self {
    Self {
      item,
      source,
      destination,
      phase: DispatchPhase::Collect,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DispatchBoard {
  pub intent: Option<DispatchIntent>,
}

impl DispatchBoard {
  pub fn new() -> Self {
    Self { intent: None }
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
