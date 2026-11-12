// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { platformArg, t } from "../../../i18n/i18n";
import {
  antiFlickerToggle,
  autoZoomToggle,
  captureToggle,
  confidenceChecksToggle,
  controlsOnLaunchToggle,
  desktopIconsToggle,
  dockToggle,
  folders,
  hiddenAppsRow,
  launchToggle,
  menuBarToggle,
  recordingSizeToggle,
} from "../sections/general-rows";
import {
  glideToggles,
  modifierDescription,
  modifierKeys,
  modifierTitle,
} from "../sections/glide-rows";
import {
  keepAwakeShortcuts,
  recordingShortcuts,
  screenshotShortcuts,
} from "../sections/global-shortcuts";
import { ocrShortcuts } from "../sections/ocr-shortcuts";
import { rulerShortcuts } from "../sections/ruler-shortcuts";
import { sectionTitle, type SettingsSection } from "../settings-sections";
import { fixedShortcutGroups } from "../shortcuts/fixed-shortcut-entries";

import type { TranscriptionModel } from "../../transcription/types";

// One thing Settings search can find. `title` is shown and matched first;
// opening the result finds the row, or the group, of that title in its pane.
export type SettingsSearchEntry = {
  id: string;
  /** Where it lives, as "Tools › Glide". */
  location: string;
  section: SettingsSection;
  target: "group" | "pane" | "row";
  title: string;
  description?: string;
  /** Other words it is found by. */
  keywords?: string;
};

type Row = { title: string; description?: string };

const row = (title: string, description?: string): Row => ({
  description,
  title,
});

const shortcutRow = ({
  description,
  label,
}: {
  label: string;
  description?: string;
}) => row(label, description);

// Every setting and shortcut, in the panes' own order. The lists come from
// the same modules the panes draw from, so a row added to one is found here;
// the rows a pane writes out one by one are listed beside it below.
export function settingsSearchEntries({
  isMac,
  models,
  momentNames,
}: {
  isMac: boolean;
  models: readonly TranscriptionModel[];
  /** The names of the person's moments. */
  momentNames: readonly string[];
}): SettingsSearchEntry[] {
  const entries: SettingsSearchEntry[] = [];
  // The rows of one group: `at(section, group)(rows, keywords)`.
  const at =
    (section: SettingsSection, group?: string) =>
    (
      rows: readonly Row[],
      keywords?: string,
      target: SettingsSearchEntry["target"] = "row",
    ) => {
      const location = [sectionTitle(section), group]
        .filter(Boolean)
        .join(" › ");
      for (const { description, title } of rows)
        entries.push({
          description,
          id: `${location}:${title}`,
          keywords,
          location,
          section,
          target,
          title,
        });
    };

  // General
  at("general", t("settings-saving"))(
    folders(),
    t("settings-search-keywords-folders"),
  );
  const capture = at("general", t("settings-capture"));
  capture([captureToggle(), desktopIconsToggle(), hiddenAppsRow()]);
  capture(
    [row(t("settings-screenshot-delay"))],
    t("settings-search-keywords-countdown"),
  );
  at("general", t("settings-startup"))(
    [launchToggle(), controlsOnLaunchToggle()],
    t("settings-search-keywords-startup"),
  );
  at("general")(
    [row(t("settings-about"))],
    t("settings-search-keywords-updates"),
    "group",
  );

  // Recording
  const controls = at("recording", t("settings-recording-controls"));
  controls([confidenceChecksToggle(), recordingSizeToggle()]);
  controls(
    [row(t("settings-countdown"))],
    t("settings-search-keywords-countdown"),
  );
  const video = at("recording", t("settings-recording-video"));
  video(
    [row(t("settings-frame-rate"), t("settings-frame-rate-description"))],
    t("settings-search-keywords-frame-rate"),
  );
  video(
    [menuBarToggle(), ...(isMac ? [dockToggle()] : [])],
    t("settings-search-keywords-bars"),
  );
  video([autoZoomToggle()], t("settings-search-keywords-auto-zoom"));
  video([antiFlickerToggle()], t("settings-search-keywords-anti-flicker"));

  // Tools
  const glide = t("settings-section-glide");
  at("tools", glide)(
    [row(t("settings-glide-enabled")), ...glideToggles()],
    t("settings-search-keywords-glide"),
  );
  at("tools", glide)(
    [row(t("settings-glide-window-gap"))],
    t("settings-search-keywords-window-gap"),
  );
  at("tools", t("settings-section-ruler"))(
    [row(t("settings-ruler-enabled"))],
    t("settings-search-keywords-ruler"),
  );
  at("tools", t("settings-section-annotate"))(
    [
      row(
        t("settings-annotate-enabled"),
        t("settings-annotate-enabled-description"),
      ),
      row(t("settings-annotate-keep"), t("settings-annotate-keep-description")),
    ],
    t("settings-search-keywords-annotate"),
  );
  at("tools", t("settings-section-ocr"))(
    [row(t("settings-ocr-enabled"))],
    t("settings-search-keywords-ocr"),
  );

  // Moments
  at("moments")(
    [row(sectionTitle("moments"))],
    t("settings-search-keywords-moments"),
    "pane",
  );
  const voiceNotes = at("moments", t("settings-voice-notes"));
  voiceNotes([
    row(
      t("settings-voice-notes-enabled"),
      t("settings-voice-notes-enabled-description"),
    ),
    row(t("settings-voice-notes-transcription")),
  ]);
  voiceNotes(
    [
      row(
        t("settings-voice-notes-microphone"),
        t("settings-voice-notes-microphone-description"),
      ),
    ],
    t("settings-search-keywords-microphone"),
  );

  // Models
  at("transcription")(
    models.map(({ description, name }) => row(name, description)),
    t("settings-search-keywords-models"),
  );
  at("transcription")([
    row(
      t("settings-transcription-language"),
      t("settings-transcription-language-description"),
    ),
  ]);

  // Shortcuts
  const shortcut = t("settings-search-keywords-shortcut");
  at("hotkeys", t("settings-shortcuts-system-keys"))(
    [
      row(
        t("settings-shortcuts-system-keys-replace", {
          platform: platformArg(),
        }),
        t("settings-shortcuts-system-keys-replace-description"),
      ),
    ],
    shortcut,
  );
  at("hotkeys", t("settings-recording"))(
    recordingShortcuts().map(shortcutRow),
    shortcut,
  );
  at("hotkeys", t("settings-screenshots"))(
    screenshotShortcuts().map(shortcutRow),
    shortcut,
  );
  at("hotkeys", t("settings-section-moments"))(
    momentNames.map((moment) =>
      row(t("settings-shortcut-place-moment", { moment })),
    ),
    shortcut,
  );
  at("hotkeys", glide)(
    modifierKeys.map((key) =>
      row(modifierTitle(key), modifierDescription(key)),
    ),
    shortcut,
  );
  at("hotkeys", t("settings-section-ruler"))(
    [row(t("settings-ruler-show")), ...rulerShortcuts().map(shortcutRow)],
    shortcut,
  );
  at("hotkeys", t("settings-section-annotate"))(
    [row(t("settings-annotate-draw")), row(t("settings-annotate-clear"))],
    shortcut,
  );
  at("hotkeys", t("settings-section-ocr"))(
    [row(t("settings-ocr-activate")), ...ocrShortcuts().map(shortcutRow)],
    shortcut,
  );
  at("hotkeys", t("settings-keep-awake"))(
    keepAwakeShortcuts().map(shortcutRow),
    shortcut,
  );
  for (const [group, labels] of fixedShortcutGroups())
    at("hotkeys", group)(
      labels.map((label) => row(label)),
      shortcut,
    );

  // Licences
  at("licences", t("settings-licences-keys"))(
    [row(t("settings-licences-add-key"))],
    t("settings-search-keywords-licences"),
  );

  return entries;
}
