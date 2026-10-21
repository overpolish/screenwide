<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Working in Screenwide

Screenwide is a Tauri desktop app for macOS and Windows. The frontend uses React, TypeScript, Tailwind CSS, React Aria, and Motion. Native capture, preview, and output processing live in Rust and platform-specific code.

## Git

- Preserve unrelated changes. Do not commit, stage, or discard work unless asked.

## Structure and naming

- `src/primitives`: single-purpose primitives built on React Aria. They import nothing from `composites` or `features`. Story group: `Primitives`.
- `src/composites`: compositions of primitives reused by more than one feature. They import nothing from `features`. Story group: `Composites`.
- `src/features`: UI and behaviour owned by one feature. May import from `primitives` and `composites`. Story group: `Features`, or `Legacy` for windows not yet built on the current primitives.
- A layer's folder and its story group share one name, so a story is found where its source lives.
- `src/features/editor`, `src-tauri/src/editor`, and the `annotations`, `compositor`, `cursor_effects`, `export`, `images`, `keyboard_effects`, `placement`, `scenes`, `text`, `timeline_edit` and `workspace` crates in `src-tauri/crates`: the Editor feature. `text` is the text engine: type set on each platform, the style vocabulary and templates, and captions as one of its users. `workspace` is the editor workspace every platform shares: the scene its layers are drawn in and the editing engine (`workspace::editing`) that turns pointer and typing input into gestures, annotations and chrome; a platform surface only provides its `Host`.
- `src-tauri/crates`: the Rust workspace's crates, each a layer the app crate (`src-tauri/src`) and the crates above it depend on, never the reverse. New Rust subsystems start as their own crate there, not as a module of the app crate, which wires the crates together: Tauri setup, commands and windows. Test a crate on its own with `cargo test -p <crate>`.
- `src/styles`: the design tokens. `theme.css` holds the five type roles (title, heading, body, label, caption), two heights (`control-height` for controls, `part-height` for small parts such as badges and keys), the size of a small selection mark (`mark`, for checkboxes and radios), icon sizes (`icon-sm`, `icon`, `icon-lg`; `icon-xs` is a utility in `utilities.css`, since it sets a thinner stroke with its size), semantic spacing (`inline`, `separate`, `inset`, `inset-sm`, `traffic-lights`) and four radii (`sm` for small marks such as swatches and checkboxes, `part` for small parts, `control`, `panel`); `colors.css` the colour roles and materials per appearance; `utilities.css` the utilities over them. `src/index.css` imports them and holds the fonts and global styles.
- `src/lib/motion.ts`: shared motion timings and easing.
- `src/storybook/feature-story-stage.tsx`: `featureStage`, the decorator that frames a feature story at its production size.
- **Editor** is the editing window and its workspace, in internal names, routes, events, and stories. **Export** means producing or saving finished files.
- **Annotation** is the only word for a drawn arrow or counter, in identifiers, comments, tests, and user-visible copy. Do not reintroduce "mark" as a synonym. **Annotate** stays the verb form for the live desktop overlay feature and its settings, routes, and commands. `mark` in this repository means the brand mark or a timeline tick marker, and nothing else.
- Native preview is owned by the native compositor. Browser stories cannot reproduce it; do not add a substitute rendering path just for stories.

## UI conventions

- Reuse `primitives` and `composites`. Inspect their APIs before adding custom controls, sizes, padding, typography, or interaction behaviour.
- Join class names with `cn` from `src/lib/styling.ts`, never `clsx` directly: `cn` runs `clsx` and then resolves conflicting Tailwind classes, so a caller's `className` overrides the component's own. ESLint rejects `clsx` imports anywhere else.
- Use the tokens for type, colour, heights, icon sizes, spacing and radii. Spacing tokens are named by what they separate: `gap-inline` between things read as one (an icon and its label), `gap-separate` between neighbours that stand on their own (a message and its action, buttons side by side), `p-inset` between a box's edge and its content, and `p-inset-sm` for the same inside a small part (a badge, a key). A role with no token yet gets one when it appears, rather than a raw step from Tailwind's scale. A raw colour must not replace a token. Text and fills are translucent so the native window material shows through; keep them so. A value one feature needs, such as a column's width, stays in that feature rather than becoming a token. Use stories under `Features` as reference for current usage; `Legacy` stories show unfinished work.
- One look on every platform. What the OS dictates (traffic lights versus caption buttons, ⌘ versus Ctrl) keys off `data-platform` where it is used; the `windows:` variants left in components go as each is redone.
- Colour is monochrome, and each role is chosen by meaning. `primary` fills the primary action, the one control a window exists for, anything checked or on (a switch, a chosen option, the selected clip) and progress, with `on-primary` on it: a warm black, a warm near-white in dark. `focus` is that ink at a third of its strength, ringing the focused control. `selected` is one of a set chosen in place: a grey surface with the text unchanged. `emphasis` is the brand orange, the logo's colour, kept for where you are (the playhead, the selection on the picture): a mark, never text. Neutrals and the status colours are the app's own, drawn in OKLCH beside the orange.
- Name a component's colour prop `color`. Use status names (`error`, `success`) only when the colour means that status. Colours that only tell things apart, such as rotation axes, take hue names (`red`, `green`, `blue`) drawn with the status colours, so the app keeps one palette.
- Use `WindowHeader` for window titles. Its display variant is the default; version text belongs in its actions area and uses the mono font.
- Window layout owns the inset and header/content gap. Use flex/grid gaps for sibling separation rather than adding spacing inside the header.
- Use `ScrollArea` for scrolling regions. With `edgeEffect="inset"` it owns its inner padding and provides the recessed content treatment. Settings sections are separated by spacing only: no borders, dividers, or blur.
- Use `ControlRow` for title/control rows, in Settings and editor tool panels alike. Descriptions are optional: omit them when the label is sufficient, otherwise use plain language and at most ten words in Settings. Avoid repeating the label.
- Editor tool panels order their rows in fixed groups, skipping any a tool lacks: (1) mode: the tool's on/off switch, a choice that changes which rows appear (Show shortcuts, Redaction and its Style, Fit), or an image's picture rows (Image, Frame, Playback); (2) size: Dimensions, Size or Width, Block size or Strength, then Position; (3) geometry: Radius, Angle, Inset; (4) options: Head, Alignment, Hand-drawn, Drop shadow, other switches; (5) animation: Animate, Animation, Click animation; (6) colour: the colour grid, Background, Appearance; (7) footnotes; (8) one-shot actions (Reverse, Recenter, Reset, Restore) in a right-aligned button row. An image's Flip is the exception: it sits beside Replace in the Image row, since a button acts on the row it sits in. Recording-only and conditional rows sit late so their absence never reorders the rows above.
- The camera panel orders by what each row changes: rows that transform the camera's content (Enhance, then Remove background and its rows) come before layout (Size, Radius), with Drop shadow after.
- Use switches for settings that take effect immediately. Use checkboxes for selection or checklist semantics, such as release-note task lists.
- Set text with a type-role token and a text-colour token on the element itself (`<p className="text-caption text-fg-secondary">`); there is no text component. Use `Keyboard` and `Shortcut` for shortcut display, `HotkeyField` for capture, and `PathField` for file/folder controls.
- Use Lucide icons at a size token: `size-icon-xs` for an icon laid over a small part such as a swatch (it brings a thinner stroke with it), `size-icon-sm` beside caption text, `size-icon` beside body text and control labels, `size-icon-lg` for an icon standing alone that needs more presence. A primitive sizes the icons it contains; do not reach into nested primitives with ancestor selectors to resize them.
- Text buttons carry no icon, primary or neutral: dialogs, sheets, Settings, forms, footers, and tool panel actions are text only, as on macOS and Windows. A window toolbar (the Editor titlebar, the recording bar) is the exception: there every text command takes a leading icon, and an action always uses the same icon. Icon-only buttons are unaffected. Exempt are icons that belong to a control or its content rather than decorating a label: `PathField`, `HotkeyField`, `Select`, `NumberField` steppers, source thumbnails, and `ConfirmActionButton`, whose icon signals its armed state.
- No ellipsis ("…" or "...") in labels, buttons, menu items, or status and loading text. Truncating content that does not fit, such as a shortened file path, may still use one.
- Preserve keyboard navigation, accessible names, focus behaviour, and disabled states. Reuse the existing Escape-cancellation handling where applicable.
- Every primitive tells assistive technology what it is and what value it holds: build it on React Aria's hook or component for that role (`useMeter`, `useSlider`, `useSwitch`). A control with no visible label takes a required `aria-label`, and its value text is a translated message. Parts that only decorate (ticks, glyphs, a readout beside an announced value) are `aria-hidden`.
- Settings uses icon-only `SidebarNav` with tooltips.

## Translation

- User-visible text lives in `locales/en-US/*.ftl` (Fluent). The webviews read it with `t` from `src/i18n/i18n.ts` and Rust with `screenwide_i18n::t!`. Do not write copy into components or native code. Rust error details, logs and text written into exported files stay English; titles and messages a person reads are translated. `TRANSLATING.md` is the contributor guide.
- A message id starts with its file's name (`project-browser-…`) and holds one string: no attributes. Pass ids to `t` and `t!` as literals so `pnpm i18n:check` can find them. The Annotation wording rule applies to ids as well as text.
- Write whole sentences in the message, with placeholders and variants for counts (`$count`) and platforms (`$platform`, from `platformArg()`). Never join fragments or choose plurals in code.
- Call `t` while rendering, not at module scope. Format dates, numbers and lists with React Aria's `useDateFormatter`, `useNumberFormatter` and `useListFormatter`, or with `appLocale().formatLocale` outside components; never `Intl.*(undefined)`.
- After adding a language, or adding, renaming or removing an `en-US` message, or changing its placeholders, run `pnpm i18n:generate`. It writes `src/i18n/messages.ts` and the macOS bundle strings in `src-tauri/macos-localizations`. Tests compare text in `en-US` with Fluent's direction isolates (U+2068, U+2069) removed.
- `screenwide/no-literal-copy` flags literal copy anywhere in `src`; stories, tests, fixtures and dev previews are exempt in `eslint.config.js`.
- Try a language with `SCREENWIDE_LOCALE=<tag>` for the app, or Storybook's Language toolbar. `en-XA` is the pseudo-locale: accented, longer English that shows text skipping translation and layouts that break. `en-XB` is its right-to-left twin, and Storybook's Direction toolbar mirrors any language.
- Layouts mirror in right-to-left languages. Name sides by start and end: `ms-`/`me-`, `ps-`/`pe-`, `start-`/`end-`, `text-start`, `rounded-s`, `border-e`, and `insetInlineStart`-style keys in `style`. `screenwide/logical-direction` flags left and right. Where something must sit on a physical side, say so with an `rtl:` or `ltr:` variant; a sideways `translate-x` needs its `rtl:` twin by hand. Directional icons (arrows, chevrons) flip with `rtl:-scale-x-100`. The timeline, the picture and screen geometry stay left to right: their roots carry `dir="ltr"`, and their files are exempt from the rule in `eslint.config.js`.
- Names that come from the system or the user (devices, windows, projects, files, moments) take `dir="auto"`, so an English name in a right-to-left window keeps its own order and is cut short at its own end.

## Models

- A model is usable when the licence on its weights permits it. Go by the licence the authors attach to the weights; do not trace the licences of the datasets they were trained on.
- Weights with no licence of their own take the licence of the repository that publishes them. Weights published with no licence anywhere are not usable.

## Stories and verification

- Feature windows built on the current primitives live under `Features`. Unfinished or older windows stay under `Legacy`.
- Frame every feature story with the `featureStage` decorator, so the canvas scrolls instead of squeezing it. A window in `src-tauri/tauri.conf.json` takes `featureStage({ window: "<label>" })`, never copied numbers: a resizable window then fills the canvas down to its minimum size. A window sized at runtime, a Rust-built window or a part of one takes `featureStage({ width, height })`; omit `height` when the content sets it. Pass a function to pick the size from args. Other decorators go before it in the array, so the stage wraps them. A window's root fills its parent, as `WindowShell` does by default, never the viewport (`h-screen`, `w-screen`, `dvh`): in a story the viewport is the canvas, not the window.
- Keep each story focused on one useful state. Avoid explanatory filler and artificial backgrounds unrelated to the feature. Use safe preview callbacks instead of triggering native saves, updates, or other live actions.
- Give every event prop (`onPress`, `onChange`, …) an `fn()` from `storybook/test` in `meta.args`, so the Actions panel logs each event, and leave it out of Controls, where it only offers to replace the function. A story that renders its own grid passes the arg on to every instance, and a render that handles an event itself, such as one keeping Controls in step, still calls the arg.
- Storybook Docs shows a component's JSDoc. A primitive whose name says what it does gets no component JSDoc; notes for whoever edits it go in `//` comments. Prop JSDoc gives the unit or meaning in a few words ("Angle in degrees."), or is omitted when the name says it. Stories take no JSDoc: the story's name says what it shows.
- Stories import `Meta` and `StoryObj` as types from `@storybook/react-vite` and declare `meta` with `satisfies Meta<typeof Component>`.
- A story not framed by `featureStage` uses `layout: "centered"`, set once in `meta`; no story switches to `padded`.
- Give a `color` or variant prop an `inline-radio` control with its default in `table.defaultValue`, and one story that shows every value side by side, with Controls disabled. Lay it out with `StoryGrid` and a `StoryGridRow` per value, from `src/storybook/story-grid.tsx`; when a second prop changes how the first looks, pass its values as `columns`. Set the shown value after spreading `args` (`<Button {...args} color={color} />`), or the story's args override it. Pass elements to a story's components as JSX children, never inside object props: Storybook's source printer recurses through such objects until the stack overflows.
- Do not add a story that only changes a value Controls already sets, such as a number or text. Interaction states such as disabled keep their own story, as does an optional part that changes the layout when present, such as a description under a label. A control with a few distinct states (unchecked, checked, indeterminate) shows them side by side in a `States` story, and its `Disabled` story repeats that grid disabled.
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
