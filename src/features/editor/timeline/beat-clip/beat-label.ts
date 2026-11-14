// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { BeatWhat } from "../../island/beat/beat-island";

/** A beat on the strip: what happens at it, as its island says, a moment
 * flagged while recording, named by its kind, which the user chooses, and
 * drawn in the kind's colour, `#rrggbb`, or a gap, time inserted with
 * nothing recorded behind it. */
export type TimelineBeat =
  BeatWhat | { color: string; kind: "moment"; name: string } | { kind: "gap" };

/** What happens at a beat, said in a line: its clip's name and title. */
export function beatLabel(beat: TimelineBeat) {
  switch (beat.kind) {
    case "click":
      return beat.label === undefined
        ? t("editor-island-beat-click-unnamed")
        : t("editor-island-beat-click", { label: beat.label });
    case "type":
      return beat.label === undefined
        ? t("editor-island-beat-type-unnamed")
        : t("editor-island-beat-type", { label: beat.label });
    case "keys":
      return t("editor-island-beat-press", { keys: beat.keys.join("+") });
    case "switch":
      return t("editor-island-beat-switch", { app: beat.app });
    case "screen":
      return t("editor-island-beat-screen");
    case "sound":
      return t("editor-island-beat-sound");
    case "speech":
      return beat.line;
    case "moment":
      return t("editor-timeline-moment", { moment: beat.name });
    case "gap":
      return t("editor-timeline-gap");
  }
}
