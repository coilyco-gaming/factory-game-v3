use serde::Serialize;

pub const MAX_OBJECT_ALERTS: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AlertEntry {
  pub tick: u64,
  pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AlertHistory {
  pub entries: Vec<AlertEntry>,
}

impl AlertHistory {
  pub fn record(&mut self, tick: u64, message: impl Into<String>) {
    let message = message.into();
    if let Some(index) = self
      .entries
      .iter()
      .position(|alert| alert.message == message)
    {
      self.entries.remove(index);
    }
    self.entries.push(AlertEntry { tick, message });
    let overflow = self.entries.len().saturating_sub(MAX_OBJECT_ALERTS);
    if overflow > 0 {
      self.entries.drain(..overflow);
    }
  }

  pub fn record_many(&mut self, tick: u64, messages: impl IntoIterator<Item = impl Into<String>>) {
    for message in messages {
      self.record(tick, message);
    }
  }

  pub fn latest(&self) -> Option<&AlertEntry> {
    self.entries.last()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn records_one_alert() {
    let mut alerts = AlertHistory::default();
    alerts.record(10, "needs repairs");
    assert_eq!(
      vec![AlertEntry {
        tick: 10,
        message: "needs repairs".into(),
      }],
      alerts.entries
    );
  }

  #[test]
  fn repeated_messages_refresh_the_tick_and_newest_position() {
    let mut alerts = AlertHistory::default();
    alerts.record(10, "needs repairs");
    alerts.record(15, "out of power");
    alerts.record(20, "needs repairs");
    assert_eq!(
      vec![
        AlertEntry {
          tick: 15,
          message: "out of power".into(),
        },
        AlertEntry {
          tick: 20,
          message: "needs repairs".into(),
        },
      ],
      alerts.entries
    );
  }

  #[test]
  fn different_messages_and_batches_retain_order() {
    let mut alerts = AlertHistory::default();
    alerts.record_many(10, ["needs repairs", "out of power"]);
    alerts.record(20, "cannot move");
    assert_eq!(
      ["needs repairs", "out of power", "cannot move"],
      alerts
        .entries
        .iter()
        .map(|alert| alert.message.as_str())
        .collect::<Vec<_>>()
        .as_slice()
    );
  }

  #[test]
  fn clips_to_the_ten_newest_messages() {
    let mut alerts = AlertHistory::default();
    for tick in 0..12 {
      alerts.record(tick, format!("alert-{tick}"));
    }
    assert_eq!(MAX_OBJECT_ALERTS, alerts.entries.len());
    assert_eq!("alert-2", alerts.entries[0].message);
    assert_eq!("alert-11", alerts.latest().unwrap().message);
  }
}
