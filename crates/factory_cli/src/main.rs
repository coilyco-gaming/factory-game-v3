use clap::{Parser, Subcommand};
use factory_content::{ContentDatabase, ScenarioId, IRON_BARS_SCENARIO};
use factory_sim::GameState;
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
    Command::Run { scenario, ticks } => {
      let scenario_id = parse_scenario(&scenario)?;
      let mut state = GameState::new(ContentDatabase::starter(), scenario_id)
        .map_err(|error| error.to_string())?;
      let mut stdout = io::BufWriter::new(io::stdout().lock());
      for _ in 0..ticks {
        let snapshot = state.step();
        serde_json::to_writer(&mut stdout, &snapshot).map_err(|error| error.to_string())?;
        stdout.write_all(b"\n").map_err(|error| error.to_string())?;
      }
      stdout.flush().map_err(|error| error.to_string())?;
      Ok(())
    }
  }
}

fn parse_scenario(value: &str) -> Result<ScenarioId, String> {
  match value {
    "iron-bars" => Ok(IRON_BARS_SCENARIO),
    other => Err(format!("unknown scenario: {other}")),
  }
}
