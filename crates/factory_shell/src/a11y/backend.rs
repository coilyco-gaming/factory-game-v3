//! Browser half of the accessible surface. Builds the control DOM once, then
//! projects each snapshot into it as text. See docs/accessible-play.md.

use super::{describe_cell, summary_line, world_lines, Command, Report};
use crate::{ControlAction, ToolMode};
use factory_sim::{CompactRecipe, GridPosition};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlInputElement};

thread_local! {
  static QUEUE: RefCell<Vec<Command>> = const { RefCell::new(Vec::new()) };
  static NODES: RefCell<Option<Nodes>> = const { RefCell::new(None) };
}

struct Nodes {
  status: Element,
  focus: Element,
  world: Element,
  events: Element,
  feedback: Element,
  x: HtmlInputElement,
  y: HtmlInputElement,
  building: HtmlInputElement,
  published: Published,
}

/// The last text written to each region. Publishing runs every frame, and a
/// region rewritten that often is announced that often. See the docs page.
#[derive(Default)]
struct Published {
  status: String,
  focus: String,
  world: String,
  events: String,
  feedback: String,
}

fn write(node: &Element, last: &mut String, text: String) {
  if *last == text {
    return;
  }
  node.set_text_content(Some(&text));
  *last = text;
}

fn push(command: Command) {
  QUEUE.with(|queue| queue.borrow_mut().push(command));
}

pub fn drain() -> Vec<Command> {
  QUEUE.with(|queue| std::mem::take(&mut *queue.borrow_mut()))
}

/// The cell the build buttons act on. Out-of-range text falls back to 0 rather
/// than dropping the click, so a mistyped field still does something visible.
fn selected_cell() -> GridPosition {
  NODES.with(|nodes| {
    let nodes = nodes.borrow();
    let Some(nodes) = nodes.as_ref() else {
      return GridPosition { x: 0, y: 0 };
    };
    GridPosition {
      x: nodes.x.value().parse().unwrap_or(0),
      y: nodes.y.value().parse().unwrap_or(0),
    }
  })
}

fn selected_building() -> u16 {
  NODES.with(|nodes| {
    nodes
      .borrow()
      .as_ref()
      .and_then(|nodes| nodes.building.value().parse().ok())
      .unwrap_or(0)
  })
}

fn button(document: &Document, parent: &Element, id: &str, label: &str, command: fn() -> Command) {
  let Ok(element) = document.create_element("button") else {
    return;
  };
  let _ = element.set_attribute("id", id);
  let _ = element.set_attribute("type", "button");
  element.set_text_content(Some(label));
  let handler = Closure::<dyn FnMut()>::new(move || push(command()));
  let _ = element
    .add_event_listener_with_callback("click", handler.as_ref().unchecked_ref())
    .map_err(|_| ());
  // The listener outlives this call by design: the panel lives as long as the
  // page, so the closure is deliberately leaked rather than dropped.
  handler.forget();
  let _ = parent.append_child(&element);
}

fn labelled_number(
  document: &Document,
  parent: &Element,
  id: &str,
  label: &str,
  max: i32,
) -> Option<HtmlInputElement> {
  let wrapper = document.create_element("label").ok()?;
  let _ = wrapper.set_attribute("for", id);
  wrapper.set_text_content(Some(label));
  let input = document
    .create_element("input")
    .ok()?
    .dyn_into::<HtmlInputElement>()
    .ok()?;
  let _ = input.set_attribute("id", id);
  let _ = input.set_attribute("type", "number");
  let _ = input.set_attribute("min", "0");
  let _ = input.set_attribute("max", &max.to_string());
  input.set_value("0");
  let refresh = Closure::<dyn FnMut()>::new(move || push(Command::Focus(selected_cell())));
  let _ = input
    .add_event_listener_with_callback("change", refresh.as_ref().unchecked_ref())
    .map_err(|_| ());
  refresh.forget();
  let _ = wrapper.append_child(&input);
  let _ = parent.append_child(&wrapper);
  Some(input)
}

fn region(document: &Document, parent: &Element, id: &str, label: &str, live: &str) -> Element {
  let element = document
    .create_element("div")
    .unwrap_or_else(|_| parent.clone());
  let _ = element.set_attribute("id", id);
  let _ = element.set_attribute("aria-label", label);
  if !live.is_empty() {
    let _ = element.set_attribute("aria-live", live);
  }
  let _ = parent.append_child(&element);
  element
}

fn heading(document: &Document, parent: &Element, text: &str) {
  if let Ok(element) = document.create_element("h2") {
    element.set_text_content(Some(text));
    let _ = parent.append_child(&element);
  }
}

pub fn install() {
  let Some(document) = web_sys::window().and_then(|window| window.document()) else {
    return;
  };
  let Some(root) = document.get_element_by_id("fg-a11y") else {
    return;
  };
  root.set_inner_html("");

  let status = region(&document, &root, "fg-status", "Simulation status", "polite");
  let focus = region(&document, &root, "fg-focus", "Focused cell", "polite");
  let feedback = region(&document, &root, "fg-feedback", "Last result", "assertive");

  heading(&document, &root, "Simulation");
  let controls = region(&document, &root, "fg-controls", "Simulation controls", "");
  button(&document, &controls, "fg-pause", "Play or pause", || {
    Command::Control(ControlAction::TogglePause)
  });
  button(&document, &controls, "fg-step", "Step one tick", || {
    Command::Control(ControlAction::Step)
  });
  button(&document, &controls, "fg-speed", "Toggle speed", || {
    Command::Control(ControlAction::ToggleSpeed)
  });
  button(&document, &controls, "fg-reset", "Reset the game", || {
    Command::Control(ControlAction::Reset)
  });

  heading(&document, &root, "Build");
  let build = region(&document, &root, "fg-build", "Build controls", "");
  let x = labelled_number(&document, &build, "fg-x", "Column", 15);
  let y = labelled_number(&document, &build, "fg-y", "Row", 15);
  button(&document, &build, "fg-road", "Place road", || {
    Command::EditAt(ToolMode::Road, selected_cell())
  });
  button(&document, &build, "fg-erase", "Remove road", || {
    Command::EditAt(ToolMode::Erase, selected_cell())
  });
  button(&document, &build, "fg-factory", "Place factory", || {
    Command::EditAt(ToolMode::Building, selected_cell())
  });
  button(
    &document,
    &build,
    "fg-inspect",
    "Describe this cell",
    || Command::Focus(selected_cell()),
  );

  heading(&document, &root, "Recipe");
  let recipe = region(&document, &root, "fg-recipe", "Recipe controls", "");
  let building = labelled_number(&document, &recipe, "fg-building", "Factory number", 64);
  button(&document, &recipe, "fg-select", "Select factory", || {
    Command::SelectBuilding(selected_building())
  });
  button(&document, &recipe, "fg-iron", "Make iron bars", || {
    Command::Control(ControlAction::Configure(CompactRecipe::IronBars))
  });
  button(&document, &recipe, "fg-copper", "Make copper bars", || {
    Command::Control(ControlAction::Configure(CompactRecipe::CopperBars))
  });

  heading(&document, &root, "World");
  let world = region(&document, &root, "fg-world", "World inventory", "");
  heading(&document, &root, "Recent events");
  let events = region(&document, &root, "fg-events", "Recent events", "polite");

  let (Some(x), Some(y), Some(building)) = (x, y, building) else {
    return;
  };
  NODES.with(|nodes| {
    *nodes.borrow_mut() = Some(Nodes {
      status,
      focus,
      world,
      events,
      feedback,
      x,
      y,
      building,
      published: Published::default(),
    });
  });
}

pub fn publish(report: &Report<'_>) {
  NODES.with(|nodes| {
    let mut nodes = nodes.borrow_mut();
    let Some(nodes) = nodes.as_mut() else {
      return;
    };
    let published = &mut nodes.published;
    write(&nodes.status, &mut published.status, summary_line(report));
    write(
      &nodes.focus,
      &mut published.focus,
      describe_cell(report.snapshot, report.focus),
    );
    write(
      &nodes.feedback,
      &mut published.feedback,
      report.feedback.to_string(),
    );
    write(
      &nodes.world,
      &mut published.world,
      world_lines(report.snapshot).join(" "),
    );
    write(
      &nodes.events,
      &mut published.events,
      report.events.join(". "),
    );
  });
}
