# AGENTS.md — mandatory project conventions

These rules are non-negotiable. Follow them in every change. When in doubt, any
of these rules wins over "matching surrounding code".

## Naming: no abbreviations, no single letters

- Every function, variable, constant, parameter and field MUST have a full,
  descriptive name.
- NEVER use single-letter identifiers (`x`, `n`, `a`, `b`, `i`, `t`, `p`, ...).
- NEVER use cryptic or ad-hoc abbreviations (`tmp`, `val`, `ret`, `buf`, `cfg`,
  `idx`, `cnt`, `getTmp`). Use the whole word instead (`temporary`,
  `value`, `result`, `buffer`, `configuration`, `index`, `count`).
- Only well-established SI/unit and domain prefixes/suffixes are allowed where
  they are part of the meaning, e.g. `_ms`, `_s`, `_mv`, `pct` (percent), and
  fixed-width integer suffixes such as `u8`/`u16`/`u32` when naming a NixN
  helper (`persist_u16`, `persist_u32`). Everything else must be spelled out.
- Prefer unambiguous full words over clever short forms even if they are longer.

## One unit per file

- Each file defines exactly ONE logical unit: one function when feasible, or one
  item. Maintain the existing layout where a module directory contains one file
  named after the unit it provides (see `src/config_storage/`, `src/gate/`,
  `src/homeassistant/`, `src/http/`, `src/pure/`, `src/state/`).
- The file name matches the item it provides (e.g. `valid_grace_ms.rs` contains
  `fn valid_grace_ms`).
- New top-level logic goes into its own file inside the right module; re-export
  it from that module's `mod.rs`.
- Keep tests in the module's `tests.rs` (e.g. `src/pure/tests.rs`), not spread
  across feature files.

## TypeScript: arrow functions and `type` only

- NEVER use the `function` keyword. Always use arrow functions: `const name = (args) => { ... }`.
- NEVER use `interface`. Always use `type`: `export type Foo = { ... }`.
- Every function takes at most ONE parameter (enforced by `max-params`).
  Prefer a single object argument over positional arguments.

## JavaScript/TypeScript dependencies

- NEVER install anything into `devDependencies`. Every dependency goes into
  `dependencies` (including tooling such as `eslint`, `prettier`,
  `typescript`).

## Layout sanity

- Run `cargo fmt --check` after finishing; every source file must end with a
  trailing newline.
- If a `rust-toolchain.toml` selects the `esp` toolchain, use
  `RUSTUP_TOOLCHAIN=stable` for host-side lint/format/test/deny so the toolchain
  does not need to be installed.
- Format the TypeScript scripts with Prettier (`pnpm format`), NOT `deno fmt`.

## Verification before done

- `RUSTUP_TOOLCHAIN=stable cargo fmt --check`
- `RUSTUP_TOOLCHAIN=stable cargo test --lib --target x86_64-unknown-linux-gnu`
- `RUSTUP_TOOLCHAIN=stable cargo clippy --lib --target x86_64-unknown-linux-gnu -- -D warnings`
- `RUSTUP_TOOLCHAIN=stable cargo deny check licenses` (verify toolchain, not
  necessarily run locally)
- `pnpm lint`
- `pnpm format:check`