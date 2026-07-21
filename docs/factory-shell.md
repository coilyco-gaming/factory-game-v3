# Bevy/Wasm app shell

`crates/factory_shell` is the deployable web app shell (issue #18). It proves
the Rust/Bevy/Wasm packaging path and nothing more: a minimal Bevy scene
(title text, status text, an orbiting marker) that builds native and for
`wasm32-unknown-unknown`. It deliberately has **no dependency** on
`factory_sim`, `factory_content`, or `factory_cli` - sim integration belongs
to the future viewer specced in issue #16, and the factory/galaxy split in
[app-boundary.md](app-boundary.md) applies.

## Run

- `ward exec shell-run` - native window.
- `ward exec shell-serve` - browser with trunk hot reload.
- `ward exec shell-build-web` - release Wasm bundle in
  `crates/factory_shell/dist/`.

Local quirk: if a Homebrew Rust shadows rustup on PATH, the wasm build fails
with "can't find crate for `core`" - the wasm target lives in the rustup
toolchain, so put `~/.cargo/bin` first (`PATH="$HOME/.cargo/bin:$PATH"`).

## Deploy shape

The repo-root [`Dockerfile`](../Dockerfile) follows the coilyco-bridge/deploy
static-site precedent (atlas, galaxy-gen): a Rust builder stage runs the
trunk build, then an unprivileged nginx stage serves `dist/` on 8080 with the
wasm MIME, immutable caching for trunk's hashed bundles, a revalidated
`index.html`, and `/healthz` ([`nginx.conf`](../nginx.conf)). The deploy repo
builds this image at rollout from the repo's git context - there is no image
publish job here. The eventual public surface is `factory-game.coilysiren.me`.

Wasm-opt stays off (`data-wasm-opt="0"` in `index.html`) until bundle size
matters; when it turns on, use upstream binaryen, not Debian's - see the
galaxy-gen Dockerfile for the instantiation bug that pin avoids.
