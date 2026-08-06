use clap::{Parser, Subcommand};
use factory_content::{ContentDatabase, ScenarioId, IRON_BARS_SCENARIO};
use factory_sim::{GameState, LivenessSummary, RunMetricsSnapshot};
use serde::Serialize;
use std::io::{self, Write};

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
    } => {
      let content = ContentDatabase::starter();
      let scenario_id = parse_scenario(&content, &scenario)?;
      let mut state =
        GameState::new(content, scenario_id).map_err(|error| error.to_string())?;
      let mut stdout = io::BufWriter::new(io::stdout().lock());
      for _ in 0..ticks {
        if summary_only {
          state.advance_without_snapshot();
        } else {
          let snapshot = state.step();
          serde_json::to_writer(&mut stdout, &snapshot).map_err(|error| error.to_string())?;
          stdout.write_all(b"\n").map_err(|error| error.to_string())?;
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
