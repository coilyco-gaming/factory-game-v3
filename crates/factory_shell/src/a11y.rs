//! Accessible control surface for the compact game.
//!
//! Bevy draws the whole viewer into one canvas, and AccessKit ships no web
//! adapter, so a browser exposes nothing to a screen reader and nothing to
//! query. This module builds a real DOM surface beside the canvas: focusable
//! controls, and the world state as text. Native builds keep AccessKit and
//! compile this away. See docs/accessible-play.md.

use crate::{ControlAction, ToolMode};
use factory_sim::{CompactSnapshot, GridPosition};

/// What the DOM asks of the simulation. Every variant lands in the same host
/// paths the pointer and keyboard use. Only the browser backend constructs one.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
  Control(ControlAction),
  EditAt(ToolMode, GridPosition),
  SelectBuilding(u16),
  Focus(GridPosition),
}

/// One frame of world state, flattened for the text projection.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub struct Report<'a> {
  pub snapshot: &'a CompactSnapshot,
  pub paused: bool,
  pub speed: f32,
  pub focus: GridPosition,
  pub selected_building: Option<u16>,
  pub feedback: &'a str,
  pub events: &'a [String],
}

/// A live region that reads a hundred lines is worse than one reading a few.
pub const LOG_LIMIT: usize = 6;

/// Most ticks emit nothing, so the current tick alone blanks the region a
/// moment after it speaks. See docs/accessible-play.md.
pub fn remember(log: &mut Vec<String>, events: &[String]) {
  log.extend(events.iter().cloned());
  if log.len() > LOG_LIMIT {
    let excess = log.len() - LOG_LIMIT;
    log.drain(..excess);
  }
}

/// Screen readers announce a status line, not a chart, so the numbers a
/// player needs to decide their next action are spelled out in one sentence.
pub fn summary_line(report: &Report<'_>) -> String {
  let snapshot = report.snapshot;
  let market = &snapshot.market;
  let allowance = &snapshot.allowance;
  let state = if report.paused { "Paused" } else { "Running" };
  let unlock = match allowance.next_unlock_at_sales {
    Some(at) => format!(", next building at {at} sold"),
    None => String::new(),
  };
  let selected = match report.selected_building {
    Some(id) => format!(" Factory {id} is selected."),
    None => " No factory is selected.".to_string(),
  };
  format!(
    "{state} at tick {}, speed {}x. Sold {} for {} revenue. \
     Demand {} of {} remaining this cycle. Buildings {} of {}{}.{selected}",
    snapshot.tick,
    report.speed / 2.0,
    market.sold_total,
    market.revenue,
    market.remaining_demand,
    market.demand_per_cycle,
    allowance.used,
    allowance.limit,
    unlock
  )
}

/// A player who cannot see the grid still has to choose a cell, so the
/// focused cell describes itself and its road frontage.
pub fn describe_cell(snapshot: &CompactSnapshot, cell: GridPosition) -> String {
  if cell.x < 0 || cell.y < 0 || cell.x >= snapshot.width || cell.y >= snapshot.height {
    return format!("Cell {},{} is outside the world.", cell.x, cell.y);
  }
  let what = if cell == snapshot.warehouse_position {
    "the warehouse".to_string()
  } else if let Some(building) = snapshot
    .buildings
    .iter()
    .find(|building| building.position == cell)
  {
    match building.recipe {
      Some(recipe) => format!(
        "factory {} making {}, {} input and {} output in stock",
        building.id,
        recipe.name(),
        building.input_stock,
        building.output_stock
      ),
      None => format!("factory {}, no recipe assigned", building.id),
    }
  } else if let Some(deposit) = snapshot
    .deposits
    .iter()
    .find(|deposit| deposit.position == cell)
  {
    format!(
      "a {} deposit, {} left in the ground and {} mined",
      deposit.item.as_str(),
      deposit.remaining,
      deposit.stockpile
    )
  } else if snapshot.roads.contains(&cell) {
    "road".to_string()
  } else {
    "empty".to_string()
  };
  let frontage = if has_road_frontage(snapshot, cell) {
    "has road frontage"
  } else {
    "has no road frontage"
  };
  format!("Cell {},{} is {what}. It {frontage}.", cell.x, cell.y)
}

fn has_road_frontage(snapshot: &CompactSnapshot, cell: GridPosition) -> bool {
  [(0, 1), (0, -1), (1, 0), (-1, 0)].iter().any(|(dx, dy)| {
    snapshot.roads.contains(&GridPosition {
      x: cell.x + dx,
      y: cell.y + dy,
    })
  })
}

/// The world inventory a sighted player reads off the map in one glance.
pub fn world_lines(snapshot: &CompactSnapshot) -> Vec<String> {
  let mut lines = vec![format!(
    "World {}x{}, warehouse at {},{}.",
    snapshot.width, snapshot.height, snapshot.warehouse_position.x, snapshot.warehouse_position.y
  )];
  for deposit in &snapshot.deposits {
    lines.push(format!(
      "Deposit {} at {},{}: {} left, {} mined and waiting.",
      deposit.item.as_str(),
      deposit.position.x,
      deposit.position.y,
      deposit.remaining,
      deposit.stockpile
    ));
  }
  for building in &snapshot.buildings {
    let recipe = building
      .recipe
      .map_or("no recipe".to_string(), |recipe| recipe.name().to_string());
    lines.push(format!(
      "Factory {} at {},{}: {}, {}.",
      building.id,
      building.position.x,
      building.position.y,
      recipe,
      if building.road_connected {
        "connected to a road"
      } else {
        "not connected to a road"
      }
    ));
  }
  if snapshot.warehouse_stock.is_empty() {
    lines.push("Warehouse is empty.".to_string());
  } else {
    let stock = snapshot
      .warehouse_stock
      .iter()
      .map(|(item, count)| format!("{} {}", count, item.as_str()))
      .collect::<Vec<_>>()
      .join(", ");
    lines.push(format!("Warehouse holds {stock}."));
  }
  lines
}

#[cfg(target_arch = "wasm32")]
mod backend;

#[cfg(target_arch = "wasm32")]
pub use backend::{drain, install, publish};

#[cfg(not(target_arch = "wasm32"))]
mod backend {
  use super::{Command, Report};

  pub fn install() {}

  pub fn publish(_report: &Report<'_>) {}

  pub fn drain() -> Vec<Command> {
    Vec::new()
  }
}

#[cfg(not(target_arch = "wasm32"))]
pub use backend::{drain, install, publish};

#[cfg(test)]
mod tests {
  use super::*;
  use factory_sim::{CompactGame, CompactRecipe};

  fn snapshot_with_road() -> CompactSnapshot {
    let mut game = CompactGame::new();
    game.place_road(GridPosition { x: 7, y: 8 }).expect("road");
    game.snapshot()
  }

  #[test]
  fn the_summary_names_every_number_a_decision_needs() {
    let snapshot = snapshot_with_road();
    let report = Report {
      snapshot: &snapshot,
      paused: true,
      speed: 2.0,
      focus: GridPosition { x: 0, y: 0 },
      selected_building: None,
      feedback: "",
      events: &[],
    };
    let line = summary_line(&report);
    assert!(line.starts_with("Paused at tick 0"));
    assert!(line.contains("Sold 0 for 0 revenue"));
    assert!(line.contains("Buildings 0 of 2"));
    assert!(line.contains("next building at 20 sold"));
    assert!(line.contains("No factory is selected"));
  }

  #[test]
  fn a_cell_describes_its_contents_and_its_frontage() {
    let snapshot = snapshot_with_road();
    let warehouse = describe_cell(&snapshot, snapshot.warehouse_position);
    assert!(warehouse.contains("the warehouse"));

    let road = describe_cell(&snapshot, GridPosition { x: 7, y: 8 });
    assert!(road.contains("is road"));

    let beside_road = describe_cell(&snapshot, GridPosition { x: 6, y: 8 });
    assert!(beside_road.contains("is empty"));
    assert!(beside_road.contains("has road frontage"));

    let far = describe_cell(&snapshot, GridPosition { x: 0, y: 0 });
    assert!(far.contains("has no road frontage"));

    let outside = describe_cell(&snapshot, GridPosition { x: 99, y: 99 });
    assert!(outside.contains("outside the world"));
  }

  #[test]
  fn a_deposit_and_a_factory_report_their_own_state() {
    let mut game = CompactGame::new();
    game.place_road(GridPosition { x: 7, y: 8 }).expect("road");
    let building = game
      .place_building(GridPosition { x: 6, y: 8 })
      .expect("building");
    game
      .configure_building(building, CompactRecipe::IronBars)
      .expect("recipe");
    let snapshot = game.snapshot();

    let factory = describe_cell(&snapshot, GridPosition { x: 6, y: 8 });
    assert!(factory.contains("factory 0 making Iron bars"));

    let deposit_cell = snapshot.deposits[0].position;
    let deposit = describe_cell(&snapshot, deposit_cell);
    assert!(deposit.contains("deposit"));
    assert!(deposit.contains("left in the ground"));

    let lines = world_lines(&snapshot);
    assert!(lines[0].contains("World 16x16"));
    assert!(lines.iter().any(|line| line.contains("Factory 0 at 6,8")));
    assert!(lines.iter().any(|line| line.contains("Warehouse is empty")));
  }

  #[test]
  fn the_event_log_keeps_the_recent_past_and_drops_the_oldest() {
    let mut log = Vec::new();
    remember(&mut log, &["first".to_string()]);
    remember(&mut log, &[]);
    assert_eq!(vec!["first".to_string()], log, "a quiet tick keeps the past");

    let many: Vec<String> = (0..LOG_LIMIT).map(|n| format!("event {n}")).collect();
    remember(&mut log, &many);
    assert_eq!(LOG_LIMIT, log.len());
    assert_eq!("event 0", log[0]);
    assert_eq!(format!("event {}", LOG_LIMIT - 1), log[LOG_LIMIT - 1]);
  }
}
