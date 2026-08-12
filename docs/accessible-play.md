# Accessible play

Bevy draws the entire viewer into one canvas. A canvas exposes no structure, so
a screen reader finds nothing to read and a keyboard finds nothing to focus.
`bevy_a11y` does not close this on the web: AccessKit ships macOS and Windows
adapters and no web adapter, so the browser build reaches a player through
nothing but pixels.

The shell therefore builds a real DOM surface beside the canvas. It is the
readable half of the game, not a test hook.

## What a player gets

The panel is a landmark region with headings, focusable buttons sized for
imprecise pointing, and visible focus outlines. Four regions carry the state:

* Simulation status - a live region naming tick, speed, sales, revenue, market demand, and the building allowance in one sentence.
* Focused cell - what occupies the targeted cell and whether it has road frontage, because a player who cannot see the grid still has to choose where to build.
* Last result - an assertive region carrying the same feedback the canvas shows, including refused edits.
* Recent events and World inventory - a rolling event log and the deposit, factory, and warehouse inventory a sighted player reads off the map at a glance.

Controls cover the whole loop. Play, pause, step, speed, and reset. A column
and row pair with place road, remove road, place factory, and describe. A
factory number with select and the two recipes.

## Perception is the feature

The shell already had complete keyboard control before any of this. Input was
never the gap. A player who cannot see the canvas could press every key and
learn nothing about what happened, so the work is in the text, and the buttons
are the easy half.

Two consequences shape the code. The event log keeps a short window rather than
the current tick, because most ticks emit nothing and a region that blanks a
moment after it speaks reads as a fault. Each region is written only when its
text actually changes, because publishing runs every frame and a live region
rewritten sixty times a second is announced sixty times a second.

## The simulation still decides

Every button lands in the same host paths the pointer and keyboard use, so
`factory_sim` keeps authority and the three surfaces cannot disagree. A refused
edit returns the sim's own message to the assertive region.

## Why the browser never sleeps

The native shell drops to reactive frames while paused, which takes an idle
window from most of a core to nothing. The browser build is deliberately
excluded.

A DOM click is not a winit event. A sleeping web build does not wake for one,
and dispatching a synthetic canvas event does not wake it either, so a paused
web build would ignore the panel until a mouse moved over the canvas. That is
exactly the input its users may not have, so the browser keeps redrawing and
pays the CPU rather than dropping clicks. Waking winit from the DOM would let
the browser sleep too, and is the fix worth making if that cost ever matters.

## Driving it from a script

The panel is ordinary DOM, so Playwright drives the game with no pixel
coordinates and no computer use. Every control has a stable id: `#fg-pause`,
`#fg-step`, `#fg-speed`, `#fg-reset`, `#fg-x`, `#fg-y`, `#fg-road`,
`#fg-erase`, `#fg-factory`, `#fg-inspect`, `#fg-building`, `#fg-select`,
`#fg-iron`, `#fg-copper`. State reads from `#fg-status`, `#fg-focus`,
`#fg-feedback`, `#fg-world`, and `#fg-events`.

A road column up x=7, a row west along y=2, a factory at 6,3 set to iron bars,
then play, banks revenue inside a few hundred ticks. That is the same route
[headless-play.md](headless-play.md) documents for the CLI, which remains the
better surface for driving a program: no browser, deterministic, and able to
branch from a save string.

See [factory-viewer.md](factory-viewer.md) for the canvas surface.
