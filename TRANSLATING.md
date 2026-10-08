<!--
SPDX-FileCopyrightText: 2026 overpolish
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Translating Screenwide

Screenwide's text lives in [Fluent](https://projectfluent.org) files under `locales/`, one folder per language. `locales/en-US` is the source: every other language translates it, and anything a translation leaves out shows in English. The app, its native overlays, the menu bar or notification area menu, and the macOS permission prompts all read the same files.

Screenwide follows the language order in the system settings: the first language it has a translation for wins. There is no language setting in the app. Dates, numbers and sizes follow the system's region, as long as it uses the same language as the translation, so a British English system keeps day-first dates.

## What gets translated

Everything a person reads in the interface: labels, buttons, menus, tooltips, accessible names, alert titles and the main sentence of each alert. Some text stays in English on purpose:

- The technical detail under an alert, such as an error from the system or from FFmpeg. It is there for bug reports.
- Log output and developer errors.
- Names of keys baked into exported videos by the keyboard overlay, and audio track names written into exported files.

Right-to-left languages are not supported yet. The text can be translated, but the layout does not mirror.

## Add a language

1. Create a folder in `locales/` named for the language's [BCP 47 tag](https://www.w3.org/International/articles/language-tags/), such as `de`, `pt-BR` or `zh-Hans`. Use a region only when the language differs by region.
2. Copy files from `locales/en-US` into it as you translate them. A file can be left out until it is ready: its messages show in English.
3. Run `pnpm i18n:generate`. This writes the language's macOS bundle strings in `src-tauri/macos-localizations`, which tell macOS the app speaks the language.
4. Run `pnpm i18n:check` and fix what it reports.

The build picks up a new folder by itself; there is no list of languages to edit.

## Translate a file

Each line is a message: an id, then its text. Translate the text and leave the id alone. The comments above a message (`#`) and a section (`##`, `###`) say where the text appears.

```ftl
project-browser-search-label = Search projects
```

Placeholders such as `{ $name }` are filled in by the app. Keep them, moving them where the sentence needs them. A message must not use a placeholder its English source lacks.

A message that depends on a count chooses a form with `$count`. Use the plural categories your language has (`zero`, `one`, `two`, `few`, `many`, `other`), whatever English uses:

```ftl
editor-panels-annotations-selected =
    { $count ->
        [one] { $count } annotation selected
       *[other] { $count } annotations selected
    }
```

A message that differs between macOS and Windows chooses with `$platform`, which is `windows` or `macos`:

```ftl
project-browser-open-in-file-manager =
    { $platform ->
        [windows] Open in Explorer
       *[other] Open in Finder
    }
```

The variant marked `*` is the default and must be present. Translate the variants; keep the selector and the variant names.

Terms start with `-`, like `-app-name` in `app.ftl`. Keep product names such as Screenwide, Glide and FFmpeg as they are unless your language has an established form.

Text in `overlay.ftl` and `tray.ftl` is drawn by the system in tight spaces, so keep it short.

## Check a translation

`pnpm i18n:check` fails on a syntax error, a message the source does not have, a placeholder the source does not use, or out-of-date generated files. It lists messages your language has not translated yet; those pass.

To see the translation in the app, start it with `SCREENWIDE_LOCALE` set to your folder name:

```sh
SCREENWIDE_LOCALE=de pnpm tauri dev
```

To look through windows without running the app, start Storybook with `pnpm storybook` and pick your language from the Language menu in the toolbar.

The `en-XA` pseudo-locale shows English with every letter accented and every vowel doubled, so Settings reads `Şḗḗŧŧīīƞɠş`. It shows text that misses translation, because that text stays plain, and layouts that break when text runs about a third longer. Use it with `SCREENWIDE_LOCALE=en-XA` or the Storybook Language menu.

## For developers

Code rules for text are in the Translation section of `AGENTS.md`. In short: the webviews call `t("id")` from `src/i18n/i18n.ts`, Rust calls `t!("id")` from `crate::i18n`, and the Objective-C overlays call `screenwide_osc_localized(@"id")`. After changing an `en-US` message, run `pnpm i18n:generate` to update `src/i18n/messages.ts`, which type-checks ids and placeholders.
