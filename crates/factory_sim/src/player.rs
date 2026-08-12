use crate::GridPosition;
use factory_content::{ItemId, COPPER_BARS, COPPER_ORE, IRON_BARS, IRON_ORE};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

pub const COMPACT_WORLD_WIDTH: i32 = 16;
pub const COMPACT_WORLD_HEIGHT: i32 = 16;
pub const COMPACT_SCENARIO_NAME: &str = "Compact Freight Yard";
pub const COMPACT_SAVE_VERSION: u32 = 1;
const MARKET_CYCLE_TICKS: u64 = 20;
const BUILDING_UNLOCK_SALES: [u32; 4] = [20, 50, 90, 140];
const TRUCK_CAPACITY: u32 = 10;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactRecipe {
  IronBars,
  CopperBars,
}

impl CompactRecipe {
  pub const ALL: [Self; 2] = [Self::IronBars, Self::CopperBars];

  pub const fn input(self) -> ItemId {
    match self {
      Self::IronBars => IRON_ORE,
      Self::CopperBars => COPPER_ORE,
    }
  }

  pub const fn output(self) -> ItemId {
    match self {
      Self::IronBars => IRON_BARS,
      Self::CopperBars => COPPER_BARS,
    }
  }

  pub const fn name(self) -> &'static str {
    match self {
      Self::IronBars => "Iron bars",
      Self::CopperBars => "Copper bars",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CompactDepositSnapshot {
  pub position: GridPosition,
  pub item: ItemId,
  pub remaining: u32,
  pub stockpile: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CompactBuildingSnapshot {
  pub id: u16,
  pub position: GridPosition,
  pub recipe: Option<CompactRecipe>,
  pub input_stock: u32,
  pub output_stock: u32,
  pub craft_progress: u8,
  pub road_connected: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactTruckTask {
  Idle,
  CollectRaw { deposit: u16, building: u16 },
  DeliverRaw { building: u16 },
  CollectOutput { building: u16 },
  DeliverToWarehouse,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CompactTruckSnapshot {
  pub id: u16,
  pub position: GridPosition,
  pub cargo_item: Option<ItemId>,
  pub cargo_quantity: u32,
  pub task: CompactTruckTask,
  pub route: Vec<GridPosition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactMarketSnapshot {
  pub cycle: u32,
  pub demand_per_cycle: u32,
  pub remaining_demand: u32,
  pub sold_total: u32,
  pub revenue: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CompactAllowanceSnapshot {
  pub used: u16,
  pub limit: u16,
  pub next_unlock_at_sales: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CompactSnapshot {
  pub tick: u64,
  pub name: &'static str,
  pub width: i32,
  pub height: i32,
  pub warehouse_position: GridPosition,
  pub roads: BTreeSet<GridPosition>,
  pub deposits: Vec<CompactDepositSnapshot>,
  pub buildings: Vec<CompactBuildingSnapshot>,
  pub trucks: Vec<CompactTruckSnapshot>,
  pub warehouse_stock: BTreeMap<ItemId, u32>,
  pub market: CompactMarketSnapshot,
  pub allowance: CompactAllowanceSnapshot,
  pub events: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompactEditError {
  OutOfBounds(GridPosition),
  CellOccupied(GridPosition),
  RoadInUse(GridPosition),
  RoadRequired(GridPosition),
  BuildingAllowanceExhausted { used: u16, limit: u16 },
  UnknownBuilding(u16),
}

impl fmt::Display for CompactEditError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::OutOfBounds(position) => {
        write!(
          f,
          "cell {},{} is outside the compact world",
          position.x, position.y
        )
      }
      Self::CellOccupied(position) => write!(f, "cell {},{} is occupied", position.x, position.y),
      Self::RoadInUse(position) => {
        write!(f, "road {},{} is in use by a truck", position.x, position.y)
      }
      Self::RoadRequired(position) => write!(
        f,
        "building at {},{} needs road frontage",
        position.x, position.y
      ),
      Self::BuildingAllowanceExhausted { used, limit } => {
        write!(f, "building allowance exhausted ({used}/{limit})")
      }
      Self::UnknownBuilding(id) => write!(f, "unknown building {id}"),
    }
  }
}

impl std::error::Error for CompactEditError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompactSaveError {
  UnsupportedVersion { found: u32, supported: u32 },
  Malformed(String),
}

impl fmt::Display for CompactSaveError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnsupportedVersion { found, supported } => {
        write!(
          f,
          "save format version {found} is not supported ({supported})"
        )
      }
      Self::Malformed(detail) => write!(f, "save is malformed: {detail}"),
    }
  }
}

impl std::error::Error for CompactSaveError {}

#[derive(Serialize)]
struct CompactSaveRef<'a> {
  version: u32,
  game: &'a CompactGame,
}

// CompactSaveVersionProbe owns the version, so the body reader ignores it.
#[derive(Deserialize)]
struct CompactSaveEnvelope {
  game: CompactGame,
}

#[derive(Deserialize)]
struct CompactSaveVersionProbe {
  version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CompactDeposit {
  position: GridPosition,
  item: ItemId,
  remaining: u32,
  stockpile: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CompactBuilding {
  id: u16,
  position: GridPosition,
  recipe: Option<CompactRecipe>,
  input_stock: u32,
  output_stock: u32,
  craft_progress: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CompactTruck {
  id: u16,
  position: GridPosition,
  cargo_item: Option<ItemId>,
  cargo_quantity: u32,
  task: CompactTruckTask,
  route: VecDeque<GridPosition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactGame {
  tick: u64,
  roads: BTreeSet<GridPosition>,
  deposits: Vec<CompactDeposit>,
  buildings: Vec<CompactBuilding>,
  trucks: Vec<CompactTruck>,
  warehouse_stock: BTreeMap<ItemId, u32>,
  market: CompactMarketSnapshot,
  building_limit: u16,
  events: Vec<String>,
}

impl Default for CompactGame {
  fn default() -> Self {
    Self::new()
  }
}

impl CompactGame {
  pub const WAREHOUSE_POSITION: GridPosition = GridPosition { x: 8, y: 8 };

  // Persistence is string in, string out so the format stays testable without
  // a browser. See docs/compact-persistence.md.
  pub fn to_save_string(&self) -> Result<String, CompactSaveError> {
    let envelope = CompactSaveRef {
      version: COMPACT_SAVE_VERSION,
      game: self,
    };
    serde_json::to_string(&envelope).map_err(|error| CompactSaveError::Malformed(error.to_string()))
  }

  pub fn from_save_string(raw: &str) -> Result<Self, CompactSaveError> {
    // Read the version before the body, so a future format reports its version
    // rather than a field-shaped parse error from the current one.
    let probe: CompactSaveVersionProbe =
      serde_json::from_str(raw).map_err(|error| CompactSaveError::Malformed(error.to_string()))?;
    if probe.version != COMPACT_SAVE_VERSION {
      return Err(CompactSaveError::UnsupportedVersion {
        found: probe.version,
        supported: COMPACT_SAVE_VERSION,
      });
    }
    let envelope: CompactSaveEnvelope =
      serde_json::from_str(raw).map_err(|error| CompactSaveError::Malformed(error.to_string()))?;
    Ok(envelope.game)
  }

  pub fn new() -> Self {
    let roads = BTreeSet::from([
      GridPosition { x: 7, y: 9 },
      GridPosition { x: 8, y: 9 },
      GridPosition { x: 9, y: 9 },
    ]);
    Self {
      tick: 0,
      roads,
      deposits: vec![
        CompactDeposit {
          position: GridPosition { x: 2, y: 2 },
          item: IRON_ORE,
          remaining: 600,
          stockpile: 0,
        },
        CompactDeposit {
          position: GridPosition { x: 13, y: 3 },
          item: COPPER_ORE,
          remaining: 600,
          stockpile: 0,
        },
        CompactDeposit {
          position: GridPosition { x: 2, y: 13 },
          item: IRON_ORE,
          remaining: 600,
          stockpile: 0,
        },
        CompactDeposit {
          position: GridPosition { x: 13, y: 12 },
          item: COPPER_ORE,
          remaining: 600,
          stockpile: 0,
        },
      ],
      buildings: Vec::new(),
      trucks: [7, 8, 9]
        .into_iter()
        .enumerate()
        .map(|(id, x)| CompactTruck {
          id: u16::try_from(id).expect("starter truck id fits u16"),
          position: GridPosition { x, y: 9 },
          cargo_item: None,
          cargo_quantity: 0,
          task: CompactTruckTask::Idle,
          route: VecDeque::new(),
        })
        .collect(),
      warehouse_stock: BTreeMap::new(),
      market: CompactMarketSnapshot {
        cycle: 0,
        demand_per_cycle: 4,
        remaining_demand: 4,
        sold_total: 0,
        revenue: 0,
      },
      building_limit: 2,
      events: Vec::new(),
    }
  }

  pub fn place_road(&mut self, position: GridPosition) -> Result<bool, CompactEditError> {
    self.validate_cell(position)?;
    if self.object_occupies(position) {
      return Err(CompactEditError::CellOccupied(position));
    }
    let inserted = self.roads.insert(position);
    if inserted {
      self
        .events
        .push(format!("road placed {},{}", position.x, position.y));
    }
    Ok(inserted)
  }

  pub fn remove_road(&mut self, position: GridPosition) -> Result<bool, CompactEditError> {
    self.validate_cell(position)?;
    if self
      .trucks
      .iter()
      .any(|truck| truck.position == position || truck.route.contains(&position))
    {
      return Err(CompactEditError::RoadInUse(position));
    }
    let removed = self.roads.remove(&position);
    if removed {
      self
        .events
        .push(format!("road removed {},{}", position.x, position.y));
    }
    Ok(removed)
  }

  pub fn place_building(&mut self, position: GridPosition) -> Result<u16, CompactEditError> {
    self.validate_cell(position)?;
    if self.object_occupies(position) || self.roads.contains(&position) {
      return Err(CompactEditError::CellOccupied(position));
    }
    if !self.has_road_frontage(position) {
      return Err(CompactEditError::RoadRequired(position));
    }
    let used = u16::try_from(self.buildings.len()).expect("compact building count fits u16");
    if used >= self.building_limit {
      return Err(CompactEditError::BuildingAllowanceExhausted {
        used,
        limit: self.building_limit,
      });
    }
    let id = self
      .buildings
      .last()
      .map_or(0, |building| building.id.saturating_add(1));
    self.buildings.push(CompactBuilding {
      id,
      position,
      recipe: None,
      input_stock: 0,
      output_stock: 0,
      craft_progress: 0,
    });
    self.events.push(format!(
      "building {id} placed {},{}",
      position.x, position.y
    ));
    Ok(id)
  }

  pub fn configure_building(
    &mut self,
    building_id: u16,
    recipe: CompactRecipe,
  ) -> Result<(), CompactEditError> {
    let building = self
      .buildings
      .iter_mut()
      .find(|building| building.id == building_id)
      .ok_or(CompactEditError::UnknownBuilding(building_id))?;
    building.recipe = Some(recipe);
    building.input_stock = 0;
    building.output_stock = 0;
    building.craft_progress = 0;
    self.events.push(format!(
      "building {building_id} configured for {}",
      recipe.name()
    ));
    Ok(())
  }

  pub fn step(&mut self) -> CompactSnapshot {
    self.tick += 1;
    self.advance_market_cycle();
    self.advance_mining();
    self.advance_trucks();
    self.advance_production();
    self.sell_to_market();
    self.advance_allowance();
    self.assign_idle_trucks();
    self.snapshot()
  }

  pub fn snapshot(&mut self) -> CompactSnapshot {
    let events = std::mem::take(&mut self.events);
    CompactSnapshot {
      tick: self.tick,
      name: COMPACT_SCENARIO_NAME,
      width: COMPACT_WORLD_WIDTH,
      height: COMPACT_WORLD_HEIGHT,
      warehouse_position: Self::WAREHOUSE_POSITION,
      roads: self.roads.clone(),
      deposits: self
        .deposits
        .iter()
        .map(|deposit| CompactDepositSnapshot {
          position: deposit.position,
          item: deposit.item,
          remaining: deposit.remaining,
          stockpile: deposit.stockpile,
        })
        .collect(),
      buildings: self
        .buildings
        .iter()
        .map(|building| CompactBuildingSnapshot {
          id: building.id,
          position: building.position,
          recipe: building.recipe,
          input_stock: building.input_stock,
          output_stock: building.output_stock,
          craft_progress: building.craft_progress,
          road_connected: self.has_road_frontage(building.position),
        })
        .collect(),
      trucks: self
        .trucks
        .iter()
        .map(|truck| CompactTruckSnapshot {
          id: truck.id,
          position: truck.position,
          cargo_item: truck.cargo_item,
          cargo_quantity: truck.cargo_quantity,
          task: truck.task,
          route: truck.route.iter().copied().collect(),
        })
        .collect(),
      warehouse_stock: self.warehouse_stock.clone(),
      market: self.market.clone(),
      allowance: CompactAllowanceSnapshot {
        used: u16::try_from(self.buildings.len()).expect("compact building count fits u16"),
        limit: self.building_limit,
        next_unlock_at_sales: BUILDING_UNLOCK_SALES
          .get(usize::from(self.building_limit.saturating_sub(2)))
          .copied(),
      },
      events,
    }
  }

  fn validate_cell(&self, position: GridPosition) -> Result<(), CompactEditError> {
    if position.x < 0
      || position.y < 0
      || position.x >= COMPACT_WORLD_WIDTH
      || position.y >= COMPACT_WORLD_HEIGHT
    {
      return Err(CompactEditError::OutOfBounds(position));
    }
    Ok(())
  }

  fn object_occupies(&self, position: GridPosition) -> bool {
    position == Self::WAREHOUSE_POSITION
      || self
        .deposits
        .iter()
        .any(|deposit| deposit.position == position)
      || self
        .buildings
        .iter()
        .any(|building| building.position == position)
  }

  fn has_road_frontage(&self, position: GridPosition) -> bool {
    cardinal_neighbors(position)
      .into_iter()
      .any(|neighbor| self.roads.contains(&neighbor))
  }

  fn advance_market_cycle(&mut self) {
    if self.tick % MARKET_CYCLE_TICKS != 0 {
      return;
    }
    self.market.cycle += 1;
    self.market.demand_per_cycle = 4 + self.market.cycle;
    self.market.remaining_demand = self
      .market
      .remaining_demand
      .saturating_add(self.market.demand_per_cycle);
    self.events.push(format!(
      "market cycle {} demand +{}",
      self.market.cycle, self.market.demand_per_cycle
    ));
  }

  fn advance_mining(&mut self) {
    if self.tick % 2 != 0 {
      return;
    }
    for deposit in &mut self.deposits {
      if deposit.remaining > 0 && deposit.stockpile < 60 {
        deposit.remaining -= 1;
        deposit.stockpile += 1;
      }
    }
  }

  fn advance_trucks(&mut self) {
    for truck_index in 0..self.trucks.len() {
      if let Some(next) = self.trucks[truck_index].route.pop_front() {
        if self.roads.contains(&next) {
          self.trucks[truck_index].position = next;
        } else {
          self.trucks[truck_index].route.clear();
          self.trucks[truck_index].task = CompactTruckTask::Idle;
        }
        continue;
      }
      self.complete_truck_task(truck_index);
    }
  }

  fn complete_truck_task(&mut self, truck_index: usize) {
    let task = self.trucks[truck_index].task;
    match task {
      CompactTruckTask::Idle => {}
      CompactTruckTask::CollectRaw { deposit, building } => {
        let deposit_index = usize::from(deposit);
        let Some(deposit) = self.deposits.get_mut(deposit_index) else {
          self.trucks[truck_index].task = CompactTruckTask::Idle;
          return;
        };
        let moved = deposit.stockpile.min(TRUCK_CAPACITY);
        if moved == 0 {
          self.trucks[truck_index].task = CompactTruckTask::Idle;
          return;
        }
        deposit.stockpile -= moved;
        let item = deposit.item;
        self.trucks[truck_index].cargo_item = Some(item);
        self.trucks[truck_index].cargo_quantity = moved;
        if let Some(route) = self.route_to_building(self.trucks[truck_index].position, building) {
          self.trucks[truck_index].route = route.into();
          self.trucks[truck_index].task = CompactTruckTask::DeliverRaw { building };
        } else {
          self.deposits[deposit_index].stockpile += moved;
          self.trucks[truck_index].cargo_item = None;
          self.trucks[truck_index].cargo_quantity = 0;
          self.trucks[truck_index].task = CompactTruckTask::Idle;
        }
      }
      CompactTruckTask::DeliverRaw { building } => {
        let Some(target) = self
          .buildings
          .iter_mut()
          .find(|target| target.id == building)
        else {
          self.trucks[truck_index].task = CompactTruckTask::Idle;
          return;
        };
        if target.recipe.map(CompactRecipe::input) == self.trucks[truck_index].cargo_item {
          target.input_stock = target
            .input_stock
            .saturating_add(self.trucks[truck_index].cargo_quantity);
          self.trucks[truck_index].cargo_item = None;
          self.trucks[truck_index].cargo_quantity = 0;
        }
        self.trucks[truck_index].task = CompactTruckTask::Idle;
      }
      CompactTruckTask::CollectOutput { building } => {
        let Some(building_index) = self
          .buildings
          .iter()
          .position(|target| target.id == building)
        else {
          self.trucks[truck_index].task = CompactTruckTask::Idle;
          return;
        };
        let Some(recipe) = self.buildings[building_index].recipe else {
          self.trucks[truck_index].task = CompactTruckTask::Idle;
          return;
        };
        let moved = self.buildings[building_index]
          .output_stock
          .min(TRUCK_CAPACITY);
        if moved == 0 {
          self.trucks[truck_index].task = CompactTruckTask::Idle;
          return;
        }
        self.buildings[building_index].output_stock -= moved;
        self.trucks[truck_index].cargo_item = Some(recipe.output());
        self.trucks[truck_index].cargo_quantity = moved;
        if let Some(route) =
          self.route_to_object(self.trucks[truck_index].position, Self::WAREHOUSE_POSITION)
        {
          self.trucks[truck_index].route = route.into();
          self.trucks[truck_index].task = CompactTruckTask::DeliverToWarehouse;
        } else {
          self.buildings[building_index].output_stock += moved;
          self.trucks[truck_index].cargo_item = None;
          self.trucks[truck_index].cargo_quantity = 0;
          self.trucks[truck_index].task = CompactTruckTask::Idle;
        }
      }
      CompactTruckTask::DeliverToWarehouse => {
        if let Some(item) = self.trucks[truck_index].cargo_item {
          *self.warehouse_stock.entry(item).or_default() += self.trucks[truck_index].cargo_quantity;
        }
        self.trucks[truck_index].cargo_item = None;
        self.trucks[truck_index].cargo_quantity = 0;
        self.trucks[truck_index].task = CompactTruckTask::Idle;
      }
    }
  }

  fn advance_production(&mut self) {
    for building in &mut self.buildings {
      let Some(recipe) = building.recipe else {
        continue;
      };
      if building.craft_progress == 0 && building.input_stock >= 3 {
        building.input_stock -= 3;
        building.craft_progress = 4;
      }
      if building.craft_progress > 0 {
        building.craft_progress -= 1;
        if building.craft_progress == 0 {
          building.output_stock = building.output_stock.saturating_add(10);
          self.events.push(format!(
            "building {} produced 10 {}",
            building.id,
            recipe.output()
          ));
        }
      }
    }
  }

  fn sell_to_market(&mut self) {
    for item in [IRON_BARS, COPPER_BARS] {
      if self.market.remaining_demand == 0 {
        break;
      }
      let stock = self.warehouse_stock.entry(item).or_default();
      let sold = (*stock).min(self.market.remaining_demand);
      if sold == 0 {
        continue;
      }
      *stock -= sold;
      self.market.remaining_demand -= sold;
      self.market.sold_total += sold;
      self.market.revenue += sold.saturating_mul(10);
      self.events.push(format!("market sold {sold} {item}"));
    }
  }

  fn advance_allowance(&mut self) {
    let unlocked = 2
      + BUILDING_UNLOCK_SALES
        .iter()
        .take_while(|threshold| self.market.sold_total >= **threshold)
        .count();
    let unlocked = u16::try_from(unlocked).expect("compact building allowance fits u16");
    if unlocked > self.building_limit {
      self.building_limit = unlocked;
      self
        .events
        .push(format!("building allowance increased to {unlocked}"));
    }
  }

  fn assign_idle_trucks(&mut self) {
    for truck_index in 0..self.trucks.len() {
      if self.trucks[truck_index].task != CompactTruckTask::Idle
        || self.trucks[truck_index].cargo_quantity > 0
      {
        continue;
      }
      if self.assign_output_job(truck_index) {
        continue;
      }
      self.assign_raw_job(truck_index);
    }
  }

  fn assign_output_job(&mut self, truck_index: usize) -> bool {
    let position = self.trucks[truck_index].position;
    let job = self
      .buildings
      .iter()
      .filter(|building| building.output_stock > 0)
      .find_map(|building| {
        self
          .route_to_object(position, building.position)
          .map(|route| (building.id, route))
      });
    let Some((building, route)) = job else {
      return false;
    };
    self.trucks[truck_index].task = CompactTruckTask::CollectOutput { building };
    self.trucks[truck_index].route = route.into();
    true
  }

  fn assign_raw_job(&mut self, truck_index: usize) -> bool {
    let position = self.trucks[truck_index].position;
    let job = self.buildings.iter().find_map(|building| {
      let recipe = building.recipe?;
      if building.input_stock >= 6 {
        return None;
      }
      self
        .deposits
        .iter()
        .enumerate()
        .filter(|(_, deposit)| deposit.item == recipe.input() && deposit.stockpile > 0)
        .find_map(|(deposit_index, deposit)| {
          let route = self.route_to_object(position, deposit.position)?;
          self.route_to_object(*route.last().unwrap_or(&position), building.position)?;
          Some((
            u16::try_from(deposit_index).expect("compact deposit index fits u16"),
            building.id,
            route,
          ))
        })
    });
    let Some((deposit, building, route)) = job else {
      return false;
    };
    self.trucks[truck_index].task = CompactTruckTask::CollectRaw { deposit, building };
    self.trucks[truck_index].route = route.into();
    true
  }

  fn route_to_building(&self, from: GridPosition, building_id: u16) -> Option<Vec<GridPosition>> {
    let position = self
      .buildings
      .iter()
      .find(|building| building.id == building_id)?
      .position;
    self.route_to_object(from, position)
  }

  fn route_to_object(&self, from: GridPosition, object: GridPosition) -> Option<Vec<GridPosition>> {
    let targets = cardinal_neighbors(object)
      .into_iter()
      .filter(|position| self.roads.contains(position))
      .collect::<BTreeSet<_>>();
    self.route_on_roads(from, &targets)
  }

  fn route_on_roads(
    &self,
    from: GridPosition,
    targets: &BTreeSet<GridPosition>,
  ) -> Option<Vec<GridPosition>> {
    if !self.roads.contains(&from) || targets.is_empty() {
      return None;
    }
    if targets.contains(&from) {
      return Some(Vec::new());
    }
    let mut frontier = VecDeque::from([from]);
    let mut previous = BTreeMap::from([(from, None)]);
    let mut reached = None;
    while let Some(current) = frontier.pop_front() {
      for next in cardinal_neighbors(current) {
        if !self.roads.contains(&next) || previous.contains_key(&next) {
          continue;
        }
        previous.insert(next, Some(current));
        if targets.contains(&next) {
          reached = Some(next);
          break;
        }
        frontier.push_back(next);
      }
      if reached.is_some() {
        break;
      }
    }
    let mut current = reached?;
    let mut path = vec![current];
    while let Some(Some(parent)) = previous.get(&current) {
      if *parent == from {
        break;
      }
      current = *parent;
      path.push(current);
    }
    path.reverse();
    Some(path)
  }
}

fn cardinal_neighbors(position: GridPosition) -> [GridPosition; 4] {
  [
    GridPosition {
      x: position.x,
      y: position.y - 1,
    },
    GridPosition {
      x: position.x - 1,
      y: position.y,
    },
    GridPosition {
      x: position.x + 1,
      y: position.y,
    },
    GridPosition {
      x: position.x,
      y: position.y + 1,
    },
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  fn road_line(game: &mut CompactGame, points: impl IntoIterator<Item = GridPosition>) {
    for point in points {
      game.place_road(point).expect("test road is valid");
    }
  }

  #[test]
  fn compact_world_starts_as_a_bounded_planning_problem() {
    let mut game = CompactGame::new();
    let snapshot = game.snapshot();

    assert_eq!((16, 16), (snapshot.width, snapshot.height));
    assert_eq!(4, snapshot.deposits.len());
    assert_eq!(snapshot.warehouse_position, CompactGame::WAREHOUSE_POSITION);
    assert_eq!(0, snapshot.buildings.len());
    assert_eq!((0, 2), (snapshot.allowance.used, snapshot.allowance.limit));
    assert_eq!(Some(20), snapshot.allowance.next_unlock_at_sales);
  }

  #[test]
  fn roads_are_free_edits_but_cannot_overwrite_world_objects() {
    let mut game = CompactGame::new();
    let road = GridPosition { x: 8, y: 10 };

    assert_eq!(Ok(true), game.place_road(road));
    assert_eq!(Ok(false), game.place_road(road));
    assert_eq!(Ok(true), game.remove_road(road));
    assert_eq!(
      Err(CompactEditError::CellOccupied(
        CompactGame::WAREHOUSE_POSITION
      )),
      game.place_road(CompactGame::WAREHOUSE_POSITION)
    );
  }

  #[test]
  fn building_placement_requires_frontage_and_consumes_allowance() {
    let mut game = CompactGame::new();
    let isolated = GridPosition { x: 3, y: 3 };
    assert_eq!(
      Err(CompactEditError::RoadRequired(isolated)),
      game.place_building(isolated)
    );

    let first = game
      .place_building(GridPosition { x: 7, y: 10 })
      .expect("starter apron provides frontage");
    let second = game
      .place_building(GridPosition { x: 9, y: 10 })
      .expect("second permit is available");
    assert_ne!(first, second);
    assert!(matches!(
      game.place_building(GridPosition { x: 8, y: 10 }),
      Err(CompactEditError::BuildingAllowanceExhausted { used: 2, limit: 2 })
    ));
  }

  #[test]
  fn trucks_never_leave_authored_roads_and_shared_cells_do_not_block_motion() {
    let mut game = CompactGame::new();
    road_line(
      &mut game,
      (2..=9)
        .map(|y| GridPosition { x: 7, y })
        .chain((3..=7).map(|x| GridPosition { x, y: 2 })),
    );
    game.place_road(GridPosition { x: 2, y: 3 }).unwrap();
    let building = game
      .place_building(GridPosition { x: 6, y: 3 })
      .expect("building fronts the trunk road");
    game
      .configure_building(building, CompactRecipe::IronBars)
      .unwrap();

    for _ in 0..120 {
      let snapshot = game.step();
      assert!(snapshot
        .trucks
        .iter()
        .all(|truck| snapshot.roads.contains(&truck.position)));
    }
  }

  #[test]
  fn active_truck_routes_cannot_be_erased_out_from_under_cargo() {
    let mut game = CompactGame::new();
    road_line(
      &mut game,
      (2..=9)
        .map(|y| GridPosition { x: 7, y })
        .chain((3..=7).map(|x| GridPosition { x, y: 2 }))
        .chain([GridPosition { x: 2, y: 3 }]),
    );
    let building = game
      .place_building(GridPosition { x: 6, y: 3 })
      .expect("building fronts the route");
    game
      .configure_building(building, CompactRecipe::IronBars)
      .unwrap();

    let route_cell = (0..40)
      .find_map(|_| {
        game
          .step()
          .trucks
          .into_iter()
          .find_map(|truck| truck.route.first().copied())
      })
      .expect("a truck receives an active route");

    assert_eq!(
      Err(CompactEditError::RoadInUse(route_cell)),
      game.remove_road(route_cell)
    );
    assert!(game.snapshot().roads.contains(&route_cell));
  }

  #[test]
  fn connected_factory_sells_finished_goods_and_unlocks_capacity() {
    let mut game = CompactGame::new();
    road_line(
      &mut game,
      (2..=9)
        .map(|y| GridPosition { x: 7, y })
        .chain((3..=7).map(|x| GridPosition { x, y: 2 }))
        .chain([GridPosition { x: 2, y: 3 }]),
    );
    let building = game
      .place_building(GridPosition { x: 6, y: 3 })
      .expect("building fronts the route");
    game
      .configure_building(building, CompactRecipe::IronBars)
      .unwrap();

    let mut snapshot = game.snapshot();
    for _ in 0..800 {
      snapshot = game.step();
      if snapshot.allowance.limit > 2 {
        break;
      }
    }

    assert!(snapshot.market.sold_total >= 20);
    assert!(snapshot.market.demand_per_cycle > 4);
    assert!(snapshot.allowance.limit > 2);
  }

  fn played_game() -> CompactGame {
    let mut game = CompactGame::new();
    road_line(
      &mut game,
      (2..=9)
        .map(|y| GridPosition { x: 7, y })
        .chain((3..=7).map(|x| GridPosition { x, y: 2 }))
        .chain([GridPosition { x: 2, y: 3 }]),
    );
    let building = game
      .place_building(GridPosition { x: 6, y: 3 })
      .expect("building fronts the route");
    game
      .configure_building(building, CompactRecipe::IronBars)
      .unwrap();
    for _ in 0..400 {
      game.step();
    }
    game
  }

  #[test]
  fn a_played_game_round_trips_through_a_save() {
    let mut game = played_game();
    let restored = CompactGame::from_save_string(&game.to_save_string().unwrap()).unwrap();

    assert_eq!(game, restored);
  }

  #[test]
  fn a_restored_game_keeps_stepping_identically() {
    // Equality alone would still pass if a save dropped state the projection
    // hides, so the restored game has to stay identical under simulation.
    let mut game = played_game();
    let mut restored = CompactGame::from_save_string(&game.to_save_string().unwrap()).unwrap();

    for _ in 0..200 {
      assert_eq!(game.step(), restored.step());
    }
    assert_eq!(game, restored);
  }

  #[test]
  fn a_save_from_another_version_is_refused() {
    let mut game = CompactGame::new();
    let raw = game.to_save_string().unwrap();
    let future = raw.replacen(
      &format!("\"version\":{COMPACT_SAVE_VERSION}"),
      "\"version\":9999",
      1,
    );

    assert_eq!(
      Err(CompactSaveError::UnsupportedVersion {
        found: 9999,
        supported: COMPACT_SAVE_VERSION,
      }),
      CompactGame::from_save_string(&future)
    );
  }

  #[test]
  fn a_save_naming_an_unknown_item_is_refused() {
    let mut game = played_game();
    let raw = game.to_save_string().unwrap();
    let corrupt = raw.replace("iron_ore", "unobtanium");

    assert!(matches!(
      CompactGame::from_save_string(&corrupt),
      Err(CompactSaveError::Malformed(_))
    ));
  }

  #[test]
  fn a_truncated_save_is_refused_rather_than_partially_loaded() {
    let mut game = played_game();
    let raw = game.to_save_string().unwrap();

    assert!(matches!(
      CompactGame::from_save_string(&raw[..raw.len() / 2]),
      Err(CompactSaveError::Malformed(_))
    ));
  }
}
