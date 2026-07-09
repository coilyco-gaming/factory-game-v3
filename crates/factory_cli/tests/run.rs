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
  assert_eq!(3, lines.len());
  for line in lines {
    let value: serde_json::Value = serde_json::from_str(line).expect("json");
    assert_eq!(Some("iron-bars"), value["scenario"]["id"].as_str());
  }
}
