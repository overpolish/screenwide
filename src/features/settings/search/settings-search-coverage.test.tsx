// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import { isMacHost } from "../../../lib/platform";
import { GeneralPane } from "../panes/general-pane";
import { RecordingPane } from "../panes/recording-pane";
import { ToolsPane } from "../panes/tools-pane";
import { SettingsApiContext } from "../settings-api-context";
import { createPreviewApi } from "../settings-window-preview";
import { ShortcutsPane } from "../shortcuts/shortcuts-pane";

import { settingsSearchEntries } from "./settings-search-index";

// The panes as the window shows them, with the story's settings: every row
// they draw, by the title its control is named by, must be one search can
// open. Moments and Models are lists of the person's own kinds and models,
// found by their pane; a moment's shortcut is found by its name.

const api = createPreviewApi();

// The pictures paint their sample desktop while rendering; outside a page
// there is nothing to paint on, and the rows are what is read here.
vi.stubGlobal("document", {
  createElement: () => ({ getContext: () => null, toDataURL: () => "data:," }),
  documentElement: { dataset: { platform: "macos" } },
});

describe("settings search", () => {
  it("finds every row the panes show", async () => {
    const [general, glide, ruler, ocr, annotate, shortcuts, defaults, moments] =
      await Promise.all([
        api.getGeneralSettings(),
        api.getGlideSettings(),
        api.getRulerSettings(),
        api.getOcrSettings(),
        api.getAnnotateSettings(),
        api.getShortcutSettings(),
        api.getShortcutDefaults(),
        api.getMomentSettings(),
      ]);
    const html = renderToStaticMarkup(
      <SettingsApiContext value={api}>
        <GeneralPane
          isSaving={false}
          onChange={() => undefined}
          onError={() => undefined}
          settings={general}
          updateSetting={null}
        />
        <RecordingPane
          isSaving={false}
          onChange={() => undefined}
          settings={general}
        />
        <ToolsPane
          annotate={annotate}
          glide={glide}
          ocr={ocr}
          onChangeAnnotate={() => undefined}
          onChangeGlide={() => undefined}
          onChangeOcr={() => undefined}
          onChangeRuler={() => undefined}
          ruler={ruler}
          savingAnnotate={false}
          savingGlide={false}
          savingOcr={false}
          savingRuler={false}
        />
        <ShortcutsPane
          annotate={annotate}
          defaults={defaults}
          glide={glide}
          moments={{
            addKind: () => undefined,
            isSaving: false,
            preview: () => undefined,
            save: () => Promise.resolve(),
            settings: moments,
          }}
          ocr={ocr}
          onCaptureChange={() => Promise.resolve()}
          onChangeBinding={() => undefined}
          onChangeGlide={() => undefined}
          onChangeOcr={() => undefined}
          onChangeRuler={() => undefined}
          ruler={ruler}
          savingAnnotate={false}
          savingGlide={false}
          savingOcr={false}
          savingRuler={false}
          savingShortcut={null}
          shortcuts={shortcuts}
        />
      </SettingsApiContext>,
    );
    const found = new Set(
      settingsSearchEntries({
        isMac: isMacHost,
        models: [],
        momentNames: moments.kinds.map((kind) => kind.name),
      }).map(({ title }) => title),
    );
    const rows = [...html.matchAll(/id="[^"]+-title"[^>]*>([^<]+)</g)].map(
      ([, title]) => title.replace(/&#x27;/g, "'").replace(/&amp;/g, "&"),
    );
    // The Shortcuts pane lists one scope at a time, Anywhere's first.
    expect(rows.length).toBeGreaterThan(30);
    expect(rows.filter((title) => !found.has(title))).toEqual([]);
  });
});
