# Bevy/Wasm app surface

`crates/factory_shell` is the single deployable native and web application.
Issue #18 established the Rust/Bevy/Wasm packaging path. The viewer milestone
then replaced the placeholder scene with the snapshot projection described in
[factory-viewer.md](factory-viewer.md).

The crate depends on `factory_sim` and `factory_content`, but Bevy remains a
client of the simulation. `CompactGame` owns every player-loop rule and
deterministic tick. The application renders immutable `CompactSnapshot` values
and sends typed planning commands back to the simulation.

## Run

- `ward exec shell-run` - native simulation viewer with optimized graphics
  dependencies.
- `ward exec shell-serve` - browser viewer with trunk hot reload.
- `ward exec shell-build-web` - Wasm viewer bundle in
  `crates/factory_shell/dist/`.

Local quirk: if a Homebrew Rust shadows rustup on PATH, the wasm build fails
with "can't find crate for `core`" - the wasm target lives in the rustup
toolchain, so put `~/.cargo/bin` first (`PATH="$HOME/.cargo/bin:$PATH"`).

## Deploy shape

The repo-root [`Dockerfile`](../Dockerfile) follows the established static-site
deployment pattern. A Rust builder stage runs the trunk build, then an
unprivileged nginx stage serves `dist/` on 8080 with the wasm MIME, immutable
caching for trunk's hashed bundles, a revalidated `index.html`, and `/healthz`
([`nginx.conf`](../nginx.conf)). Forgejo Actions publishes the git-sha image to
the in-cluster registry. The deploy repo owns rollout and the public
`factory.coilysiren.me` surface.

Wasm-opt stays off (`data-wasm-opt="0"` in `index.html`) until bundle size
matters. When it turns on, use upstream binaryen because Debian's build has an
instantiation bug documented by the static-site precedent.
