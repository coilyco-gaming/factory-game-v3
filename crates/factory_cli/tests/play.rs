use std::io::Write;
use std::process::{Command, Stdio};

/// Feed request lines to `play` and collect one parsed response per line.
fn session(args: &[&str], requests: &[&str]) -> Vec<serde_json::Value> {
  let mut child = Command::new(env!("CARGO_BIN_EXE_factory_cli"))
    .arg("play")
    .args(args)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("cli starts");
  let mut stdin = child.stdin.take().expect("stdin");
  for request in requests {
    writeln!(stdin, "{request}").expect("write request");
  }
  drop(stdin);
  let output = child.wait_with_output().expect("cli exits");
  assert!(output.status.success());
  String::from_utf8(output.stdout)
    .expect("utf8")
    .lines()
    .map(|line| serde_json::from_str(line).expect("json"))
    .collect()
}

/// Roads reaching the iron deposit at 2,2, then a factory with frontage on
/// them. Trucks traverse road cells only. See docs/headless-play.md.
fn iron_route() -> Vec<String> {
  let mut requests: Vec<String> = (2..=8)
    .rev()
    .map(|y| format!(r#"{{"action":"place_road","x":7,"y":{y}}}"#))
    .collect();
  requests.extend(
    (3..=6)
      .rev()
      .map(|x| format!(r#"{{"action":"place_road","x":{x},"y":2}}"#)),
  );
  requests.push(r#"{"action":"place_building","x":6,"y":3}"#.into());
  requests.push(r#"{"action":"configure_building","building":0,"recipe":"IronBars"}"#.into());
  requests
}

#[test]
fn play_answers_every_request_with_one_line() {
  let responses = session(
    &[],
    &[
      r#"{"action":"observe"}"#,
      r#"{"action":"place_road","x":8,"y":7}"#,
      r#"{"action":"step","ticks":3}"#,
    ],
  );

  assert_eq!(3, responses.len());
  for response in &responses {
    assert_eq!(Some(true), response["ok"].as_bool());
    assert!(response["snapshot"]["tick"].is_u64());
    assert!(response["snapshot"]["market"]["sold_total"].is_u64());
  }
  assert_eq!(Some(0), responses[1]["snapshot"]["tick"].as_u64());
  assert_eq!(Some(true), responses[1]["changed"].as_bool());
  assert_eq!(Some(3), responses[2]["snapshot"]["tick"].as_u64());
}

#[test]
fn play_reports_a_refused_edit_without_ending_the_session() {
  let responses = session(
    &[],
    &[
      r#"{"action":"place_road","x":99,"y":99}"#,
      r#"{"action":"place_building","x":0,"y":0}"#,
      r#"{"action":"configure_building","building":404,"recipe":"IronBars"}"#,
      r#"{"action":"observe"}"#,
    ],
  );

  assert_eq!(4, responses.len());
  assert_eq!(Some("out_of_bounds"), responses[0]["error_kind"].as_str());
  assert_eq!(Some("road_required"), responses[1]["error_kind"].as_str());
  assert_eq!(Some("unknown_building"), responses[2]["error_kind"].as_str());
  for response in &responses[..3] {
    assert_eq!(Some(false), response["ok"].as_bool());
    assert!(response["error"].as_str().is_some_and(|e| !e.is_empty()));
    assert_eq!(Some(3), response["snapshot"]["roads"].as_array().map(Vec::len));
  }
  assert_eq!(Some(true), responses[3]["ok"].as_bool());
}

#[test]
fn play_survives_a_malformed_request() {
  let responses = session(
    &[],
    &[
      "not json at all",
      r#"{"action":"nonexistent"}"#,
      r#"{"action":"observe"}"#,
    ],
  );

  assert_eq!(3, responses.len());
  for response in &responses[..2] {
    assert_eq!(Some(false), response["ok"].as_bool());
    assert_eq!(Some("malformed_request"), response["error_kind"].as_str());
  }
  assert_eq!(Some(true), responses[2]["ok"].as_bool());
}

#[test]
fn play_refuses_to_step_past_the_tick_budget() {
  let responses = session(
    &["--max-ticks", "5"],
    &[
      r#"{"action":"step","ticks":5}"#,
      r#"{"action":"step","ticks":1}"#,
    ],
  );

  assert_eq!(Some(5), responses[0]["snapshot"]["tick"].as_u64());
  assert_eq!(Some(false), responses[1]["ok"].as_bool());
  assert_eq!(
    Some("tick_budget_exhausted"),
    responses[1]["error_kind"].as_str()
  );
  assert_eq!(Some(5), responses[1]["snapshot"]["tick"].as_u64());
}

#[test]
fn play_runs_the_compact_loop_to_revenue() {
  let mut requests = iron_route();
  requests.push(r#"{"action":"step","ticks":600}"#.into());
  let borrowed: Vec<&str> = requests.iter().map(String::as_str).collect();
  let responses = session(&["--max-ticks", "600"], &borrowed);

  let placement = &responses[11];
  assert_eq!(Some(0), placement["building"].as_u64());
  assert_eq!(Some(true), placement["ok"].as_bool());

  let final_snapshot = &responses.last().expect("final response")["snapshot"];
  assert_eq!(Some(600), final_snapshot["tick"].as_u64());
  assert_eq!(Some(589), final_snapshot["market"]["sold_total"].as_u64());
  assert_eq!(Some(5890), final_snapshot["market"]["revenue"].as_u64());
  assert_eq!(Some(6), final_snapshot["allowance"]["limit"].as_u64());
  assert!(final_snapshot["events"]
    .as_array()
    .is_some_and(|events| !events.is_empty()));
}

#[test]
fn play_round_trips_a_save_string_into_a_later_session() {
  let mut requests = iron_route();
  requests.push(r#"{"action":"step","ticks":300}"#.into());
  requests.push(r#"{"action":"save"}"#.into());
  let borrowed: Vec<&str> = requests.iter().map(String::as_str).collect();
  let first = session(&["--max-ticks", "300"], &borrowed);

  let saved = first.last().expect("save response")["save"]
    .as_str()
    .expect("save string")
    .to_string();
  let banked = first.last().expect("save response")["snapshot"]["market"]["revenue"]
    .as_u64()
    .expect("revenue");
  assert!(banked > 0);

  let directory = std::env::temp_dir().join("factory-play-round-trip");
  std::fs::create_dir_all(&directory).expect("temp dir");
  let path = directory.join("save.txt");
  std::fs::write(&path, &saved).expect("write save");

  let resumed = session(
    &["--load", path.to_str().expect("path"), "--max-ticks", "10"],
    &[r#"{"action":"observe"}"#],
  );
  let snapshot = &resumed[0]["snapshot"];
  assert_eq!(Some(300), snapshot["tick"].as_u64());
  assert_eq!(Some(banked), snapshot["market"]["revenue"].as_u64());
  std::fs::remove_dir_all(&directory).ok();
}
