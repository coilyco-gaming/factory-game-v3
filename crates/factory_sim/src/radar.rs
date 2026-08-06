use crate::{AlertHistory, DispatchBoard, DispatchIntent, NodeId};
use factory_content::{ItemId, RadarSpec};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct DeploymentRadar {
  pub node: NodeId,
  pub deployment_item: ItemId,
  pub target_item: ItemId,
  pub claimed_target: Option<NodeId>,
  pub dispatch: DispatchBoard,
  pub alerts: AlertHistory,
}

impl DeploymentRadar {
  pub fn new(node: NodeId, spec: &RadarSpec) -> Self {
    Self {
      node,
      deployment_item: spec.deployment_item,
      target_item: spec.target_item,
      claimed_target: None,
      dispatch: DispatchBoard::new(),
      alerts: AlertHistory::default(),
    }
  }

  pub fn refresh_dispatch(&mut self) {
    self.dispatch.intents = self
      .claimed_target
      .map(|target| DispatchIntent::deploy(self.deployment_item, self.node, target))
      .into_iter()
      .collect();
  }

  pub fn snapshot(&self) -> RadarSnapshot {
    RadarSnapshot {
      node: self.node,
      deployment_item: self.deployment_item,
      target_item: self.target_item,
      claimed_target: self.claimed_target,
      dispatch: self.dispatch.clone(),
      alerts: self.alerts.clone(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RadarSnapshot {
  pub node: NodeId,
  pub deployment_item: ItemId,
  pub target_item: ItemId,
  pub claimed_target: Option<NodeId>,
  pub dispatch: DispatchBoard,
  pub alerts: AlertHistory,
}
