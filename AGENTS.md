<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Working in Screenwide

Screenwide is a Tauri desktop app for macOS and Windows. The frontend uses React, TypeScript, Tailwind CSS, React Aria, and Motion. Native capture, preview, and output processing live in Rust and platform-specific code.

## Git

- Preserve unrelated changes. Do not commit, stage, or discard work unless asked.

## Structure and naming

- `src/components/base`: single-purpose primitives built on React Aria. They import nothing from `shared` or `features`. Story group: `Primitives`.
- `src/components/shared`: compositions of primitives reused by more than one feature. They import nothing from `features`. Story group: `Components`.
- `src/features`: UI and behaviour owned by one feature. May import from `base` and `shared`. Story group: `Features`, or `Legacy` for windows not yet built on the current primitives.
- `src/features/editor` and `src-tauri/src/editor`: the Editor feature.
- `src/index.css`: shared theme tokens and global styles.
- `src/lib/motion.ts`: shared motion timings and easing.
- `src/storybook/feature-story-stage.tsx`: standard feature-window story frame.
- **Editor** is the editing window and its workspace, in internal names, routes, events, and stories. **Export** means producing or saving finished files.
- **Annotation** is the only word for a drawn arrow or counter, in identifiers, comments, tests, and user-visible copy. Do not reintroduce "mark" as a synonym. **Annotate** stays the verb form for the live desktop overlay feature and its settings, routes, and commands. `mark` in this repository means the brand mark or a timeline tick marker, and nothing else.
- Native preview is owned by the native compositor. Browser stories cannot reproduce it; do not add a substitute rendering path just for stories.

## UI conventions

- Reuse primitives from `base` and components from `shared`. Inspect their APIs before adding custom controls, sizes, padding, typography, or interaction behaviour.
- Use the semantic tokens for colours, spacing, radii, and motion. A raw colour or spacing value must not replace an existing token. Use stories under `Features` as reference for current usage; `Legacy` stories show unfinished work.
- Use `WindowHeader` for window titles. Its display variant is the default; version text belongs in its actions area and uses the mono font.
- Window layout owns the inset and header/content gap. Use flex/grid gaps for sibling separation rather than adding spacing inside the header.
- Use `ScrollArea` for scrolling regions. With `edgeEffect="inset"` it owns its inner padding and provides the recessed content treatment. Settings sections are separated by spacing only: no borders, dividers, or blur.
- Use `Setting` for title/control rows. Descriptions are optional: omit them when the label is sufficient, otherwise use plain language and at most ten words in Settings. Avoid repeating the label.
- Use switches for settings that take effect immediately. Use checkboxes for selection or checklist semantics, such as release-note task lists.
- Use `Text` for body/help copy, `Keyboard` and `Shortcut` for shortcut display, `HotkeyField` for capture, and `PathField` for file/folder controls.
- Use Lucide icons. Size them through the shared control sizing, not through ancestor selectors that reach into nested primitives.
- Preserve keyboard navigation, accessible names, focus behaviour, and disabled states. Reuse the existing Escape-cancellation handling where applicable.
- Settings uses icon-only `SidebarNav` with tooltips.

## Stories and verification

- Feature windows built on the current primitives live under `Features` and use `FeatureStoryStage`. Unfinished or older windows stay under `Legacy`.
- Keep each story focused on one useful state. Avoid explanatory filler and artificial backgrounds unrelated to the feature. Use safe preview callbacks instead of triggering native saves, updates, or other live actions.
- Verify in proportion to the change. Use existing meaningful tests; do not add tests that merely repeat implementation details.
- For UI changes, inspect the relevant story. Native visual behaviour still needs native verification; lint and type checks do not prove appearance.
- Report unrelated pre-existing failures separately from failures introduced by the change. Do not quietly fix unrelated code to make checks pass.
- Respect the source-size checks and SPDX headers. Split responsibilities instead of raising a file's ceiling in `scripts/source-size-debt.json`.

Choose commands relevant to the change, run from the repository root; this is not a checklist. Reserve `pnpm check` for broad changes or required full verification:

- `pnpm exec eslint <changed files>`
- `pnpm exec prettier --check <changed files>`
- `pnpm exec tsc --noEmit`
- `pnpm exec vitest run <relevant tests>`
- `pnpm storybook` / `pnpm dev:storybook-native`
- `pnpm build-storybook`
- `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`
- `pnpm architecture:check` / `pnpm license:check`
- `pnpm check` for the full repository verification pipeline.
