use clap::{Parser, Subcommand};
use factory_content::{ContentDatabase, ScenarioId, IRON_BARS_SCENARIO};
use factory_sim::{
  CompactEditError, CompactGame, CompactRecipe, CompactSnapshot, GameState, GridPosition,
  LivenessSummary, RunMetricsSnapshot,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "factory_cli", about = "Headless factory simulation runner")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
  Run {
    #[arg(long, default_value = IRON_BARS_SCENARIO.as_str())]
    scenario: String,
    #[arg(long, default_value_t = 6)]
    ticks: u32,
    #[arg(long)]
    summary_only: bool,
    #[arg(long)]
    exhaust_batteries_at: Option<u32>,
  },
  Play {
    #[arg(long)]
    load: Option<PathBuf>,
    #[arg(long)]
    save: Option<PathBuf>,
    #[arg(long, default_value_t = 2000)]
    max_ticks: u64,
  },
}

fn main() {
  if let Err(error) = run() {
    eprintln!("{error}");
    std::process::exit(1);
  }
}

fn run() -> Result<(), String> {
  let cli = Cli::parse();
  match cli.command {
    Command::Run {
      scenario,
      ticks,
      summary_only,
      exhaust_batteries_at,
    } => {
      if exhaust_batteries_at.is_some() && !summary_only {
        return Err("--exhaust-batteries-at requires --summary-only".into());
      }
      if exhaust_batteries_at.is_some_and(|tick| tick == 0 || tick > ticks) {
        return Err("--exhaust-batteries-at must be between 1 and --ticks".into());
      }
      let content = ContentDatabase::starter();
      let scenario_id = parse_scenario(&content, &scenario)?;
      let mut state = GameState::new(content, scenario_id).map_err(|error| error.to_string())?;
      let mut stdout = io::BufWriter::new(io::stdout().lock());
      for tick in 1..=ticks {
        if summary_only {
          state.advance_without_snapshot();
        } else {
          let snapshot = state.step();
          serde_json::to_writer(&mut stdout, &snapshot).map_err(|error| error.to_string())?;
          stdout.write_all(b"\n").map_err(|error| error.to_string())?;
        }
        if exhaust_batteries_at == Some(tick) {
          state.exhaust_non_generator_batteries();
        }
      }
      let summary = SummaryLine {
        summary: state.metrics(),
        liveness: state.liveness_summary(),
      };
      serde_json::to_writer(&mut stdout, &summary).map_err(|error| error.to_string())?;
      stdout.write_all(b"\n").map_err(|error| error.to_string())?;
      stdout.flush().map_err(|error| error.to_string())?;
      Ok(())
    }
    Command::Play {
      load,
      save,
      max_ticks,
    } => play(load, save, max_ticks),
  }
}

/// One request line in, one response line out, so a driver never has to guess
/// how many lines an action produced. See docs/headless-play.md.
fn play(load: Option<PathBuf>, save: Option<PathBuf>, max_ticks: u64) -> Result<(), String> {
  let mut game = match load {
    Some(path) => {
      let raw =
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
      CompactGame::from_save_string(raw.trim()).map_err(|error| error.to_string())?
    }
    None => CompactGame::new(),
  };

  let mut spent = 0;
  let stdin = io::stdin();
  let mut stdout = io::BufWriter::new(io::stdout().lock());
  for line in stdin.lock().lines() {
    let line = line.map_err(|error| error.to_string())?;
    if line.trim().is_empty() {
      continue;
    }
    let response = match serde_json::from_str::<PlayRequest>(&line) {
      Ok(PlayRequest::Quit) => break,
      Ok(request) => apply(&mut game, request, &mut spent, max_ticks),
      Err(error) => PlayResponse::failed("malformed_request", error.to_string(), &mut game),
    };
    serde_json::to_writer(&mut stdout, &response).map_err(|error| error.to_string())?;
    stdout.write_all(b"\n").map_err(|error| error.to_string())?;
    stdout.flush().map_err(|error| error.to_string())?;
  }

  if let Some(path) = save {
    let raw = game.to_save_string().map_err(|error| error.to_string())?;
    fs::write(&path, raw).map_err(|error| format!("{}: {error}", path.display()))?;
  }
  Ok(())
}

fn apply(
  game: &mut CompactGame,
  request: PlayRequest,
  spent: &mut u64,
  max_ticks: u64,
) -> PlayResponse {
  match request {
    PlayRequest::Quit => unreachable!("quit ends the loop before dispatch"),
    PlayRequest::Observe => PlayResponse::ok(game),
    PlayRequest::Step { ticks } => step(game, ticks, spent, max_ticks),
    PlayRequest::PlaceRoad { x, y } => match game.place_road(GridPosition { x, y }) {
      Ok(changed) => PlayResponse::ok(game).with_changed(changed),
      Err(error) => PlayResponse::rejected(error, game),
    },
    PlayRequest::RemoveRoad { x, y } => match game.remove_road(GridPosition { x, y }) {
      Ok(changed) => PlayResponse::ok(game).with_changed(changed),
      Err(error) => PlayResponse::rejected(error, game),
    },
    PlayRequest::PlaceBuilding { x, y } => match game.place_building(GridPosition { x, y }) {
      Ok(building) => PlayResponse::ok(game).with_building(building),
      Err(error) => PlayResponse::rejected(error, game),
    },
    PlayRequest::ConfigureBuilding { building, recipe } => {
      match game.configure_building(building, recipe) {
        Ok(()) => PlayResponse::ok(game),
        Err(error) => PlayResponse::rejected(error, game),
      }
    }
    PlayRequest::Save => match game.to_save_string() {
      Ok(raw) => PlayResponse::ok(game).with_save(raw),
      Err(error) => PlayResponse::failed("save_failed", error.to_string(), game),
    },
  }
}

/// Events drain on every snapshot, so a multi-tick step re-attaches the ones
/// the intermediate ticks would otherwise have thrown away.
fn step(game: &mut CompactGame, ticks: u64, spent: &mut u64, max_ticks: u64) -> PlayResponse {
  let budget = max_ticks.saturating_sub(*spent);
  if ticks > budget {
    let detail = format!("step of {ticks} exceeds the remaining budget of {budget}");
    return PlayResponse::failed("tick_budget_exhausted", detail, game);
  }
  if ticks == 0 {
    return PlayResponse::ok(game);
  }
  let mut events = Vec::new();
  let mut snapshot = game.snapshot();
  for _ in 0..ticks {
    snapshot = game.step();
    events.append(&mut snapshot.events);
  }
  *spent += ticks;
  snapshot.events = events;
  PlayResponse::base(snapshot)
}

fn default_ticks() -> u64 {
  1
}

#[derive(Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum PlayRequest {
  Observe,
  Step {
    #[serde(default = "default_ticks")]
    ticks: u64,
  },
  PlaceRoad {
    x: i32,
    y: i32,
  },
  RemoveRoad {
    x: i32,
    y: i32,
  },
  PlaceBuilding {
    x: i32,
    y: i32,
  },
  ConfigureBuilding {
    building: u16,
    recipe: CompactRecipe,
  },
  Save,
  Quit,
}

#[derive(Serialize)]
struct PlayResponse {
  ok: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  error_kind: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  changed: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  building: Option<u16>,
  #[serde(skip_serializing_if = "Option::is_none")]
  save: Option<String>,
  snapshot: CompactSnapshot,
}

impl PlayResponse {
  fn base(snapshot: CompactSnapshot) -> Self {
    Self {
      ok: true,
      error: None,
      error_kind: None,
      changed: None,
      building: None,
      save: None,
      snapshot,
    }
  }

  fn ok(game: &mut CompactGame) -> Self {
    Self::base(game.snapshot())
  }

  /// A refused edit is an ordinary turn result, not a transport failure, so the
  /// driver reads the reason and the unchanged world from the same line.
  fn rejected(error: CompactEditError, game: &mut CompactGame) -> Self {
    Self::failed(edit_error_kind(&error), error.to_string(), game)
  }

  fn failed(kind: &str, detail: String, game: &mut CompactGame) -> Self {
    Self {
      ok: false,
      error: Some(detail),
      error_kind: Some(kind.to_string()),
      ..Self::base(game.snapshot())
    }
  }

  fn with_changed(mut self, changed: bool) -> Self {
    self.changed = Some(changed);
    self
  }

  fn with_building(mut self, building: u16) -> Self {
    self.building = Some(building);
    self
  }

  fn with_save(mut self, raw: String) -> Self {
    self.save = Some(raw);
    self
  }
}

fn edit_error_kind(error: &CompactEditError) -> &'static str {
  match error {
    CompactEditError::OutOfBounds(_) => "out_of_bounds",
    CompactEditError::CellOccupied(_) => "cell_occupied",
    CompactEditError::RoadInUse(_) => "road_in_use",
    CompactEditError::RoadRequired(_) => "road_required",
    CompactEditError::BuildingAllowanceExhausted { .. } => "building_allowance_exhausted",
    CompactEditError::UnknownBuilding(_) => "unknown_building",
  }
}

#[derive(Serialize)]
struct SummaryLine {
  summary: RunMetricsSnapshot,
  liveness: LivenessSummary,
}

fn parse_scenario(content: &ContentDatabase, value: &str) -> Result<ScenarioId, String> {
  content
    .scenarios
    .keys()
    .find(|id| id.as_str() == value)
    .copied()
    .ok_or_else(|| format!("unknown scenario: {value}"))
}
