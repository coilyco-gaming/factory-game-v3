use factory_content::ItemId;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunMetrics {
  pub ticks: u64,
  pub mined: BTreeMap<ItemId, u32>,
  pub crafted: BTreeMap<ItemId, u32>,
  pub dispatches_assigned: u32,
  pub units_collected: u32,
  pub units_delivered: u32,
  pub fuel_burned: u32,
  pub energy_generated: u32,
  pub energy_balanced: u64,
  pub energy_consumed: u32,
  pub power_starvations: u32,
  pub deployments: u32,
  pub generators_deployed: u32,
  pub world_deletions: u32,
  pub idle_ticks: u64,
}

impl RunMetrics {
  pub fn record_mined(&mut self, item: ItemId, quantity: u32) {
    if quantity > 0 {
      *self.mined.entry(item).or_insert(0) += quantity;
    }
  }

  pub fn record_crafted(&mut self, item: ItemId, quantity: u32) {
    if quantity > 0 {
      *self.crafted.entry(item).or_insert(0) += quantity;
    }
  }

  pub fn snapshot(&self) -> RunMetricsSnapshot {
    RunMetricsSnapshot {
      ticks: self.ticks,
      mined: self
        .mined
        .iter()
        .map(|(item, quantity)| (item.to_string(), *quantity))
        .collect(),
      crafted: self
        .crafted
        .iter()
        .map(|(item, quantity)| (item.to_string(), *quantity))
        .collect(),
      dispatches_assigned: self.dispatches_assigned,
      units_collected: self.units_collected,
      units_delivered: self.units_delivered,
      fuel_burned: self.fuel_burned,
      energy_generated: self.energy_generated,
      energy_balanced: self.energy_balanced,
      energy_consumed: self.energy_consumed,
      power_starvations: self.power_starvations,
      deployments: self.deployments,
      generators_deployed: self.generators_deployed,
      world_deletions: self.world_deletions,
      idle_ticks: self.idle_ticks,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RunMetricsSnapshot {
  pub ticks: u64,
  pub mined: BTreeMap<String, u32>,
  pub crafted: BTreeMap<String, u32>,
  pub dispatches_assigned: u32,
  pub units_collected: u32,
  pub units_delivered: u32,
  pub fuel_burned: u32,
  pub energy_generated: u32,
  pub energy_balanced: u64,
  pub energy_consumed: u32,
  pub power_starvations: u32,
  pub deployments: u32,
  pub generators_deployed: u32,
  pub world_deletions: u32,
  pub idle_ticks: u64,
}
