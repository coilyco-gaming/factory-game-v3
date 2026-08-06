use std::process::Command;

#[test]
fn cli_produces_stable_json_lines() {
  let output = Command::new(env!("CARGO_BIN_EXE_factory_cli"))
    .args(["run", "--scenario", "iron-bars", "--ticks", "3"])
    .output()
    .expect("cli runs");

  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("utf8");
  let lines: Vec<_> = stdout.lines().collect();
  assert_eq!(4, lines.len());
  let (tick_lines, summary_lines) = lines.split_at(3);
  for line in tick_lines {
    let value: serde_json::Value = serde_json::from_str(line).expect("json");
    assert_eq!(Some("iron-bars"), value["scenario"]["id"].as_str());
    assert!(value["sources"][0]["alerts"]["entries"].is_array());
    assert!(value["haulers"][0]["alerts"]["entries"].is_array());
    assert!(value["factories"][0]["alerts"]["entries"].is_array());
  }
  let second_tick: serde_json::Value = serde_json::from_str(tick_lines[1]).expect("json");
  assert_eq!(
    Some("collect => deliver"),
    second_tick["haulers"][0]["alerts"]["entries"][0]["message"].as_str()
  );
  let summary: serde_json::Value = serde_json::from_str(summary_lines[0]).expect("json");
  assert_eq!(Some(3), summary["summary"]["ticks"].as_u64());
  assert_eq!(Some(9), summary["summary"]["mined"]["iron_ore"].as_u64());
}

#[test]
fn cli_serializes_each_hybrid_grid_generator() {
  let output = Command::new(env!("CARGO_BIN_EXE_factory_cli"))
    .args(["run", "--scenario", "hybrid-grid", "--ticks", "1"])
    .output()
    .expect("cli runs");

  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("utf8");
  let tick: serde_json::Value =
    serde_json::from_str(stdout.lines().next().expect("tick line")).expect("json");
  let generators = tick["power"]["generators"]
    .as_array()
    .expect("generator snapshots");

  assert_eq!(2, generators.len());
  assert_eq!(Some("coal"), generators[0]["fuel_item"].as_str());
  assert!(generators[1]["fuel_item"].is_null());
  assert_eq!(Some(40), generators[1]["gain_rate"].as_u64());
}

#[test]
fn cli_retains_every_v2_world_object_and_wide_source_id() {
  let output = Command::new(env!("CARGO_BIN_EXE_factory_cli"))
    .args(["run", "--scenario", "v2-world", "--ticks", "1"])
    .output()
    .expect("cli runs");

  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("utf8");
  let tick: serde_json::Value =
    serde_json::from_str(stdout.lines().next().expect("tick line")).expect("json");

  assert_eq!(Some(424), tick["sources"].as_array().map(Vec::len));
  assert_eq!(Some(15), tick["factories"].as_array().map(Vec::len));
  assert_eq!(Some(15), tick["haulers"].as_array().map(Vec::len));
  assert_eq!(Some("source-423"), tick["sources"][423]["node"].as_str());
}
