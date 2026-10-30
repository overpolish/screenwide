// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { splitAtPlaceable } from "../../../../i18n/split-at-placeable";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { Keyboard, Shortcut } from "../../../../primitives/keyboard/keyboard";
import { BEAT_GLYPHS } from "../../glyphs/beat-glyphs";

import type { BeatWhat } from "./beat-island";

// What happens at the beat, said as it happens there, with the glyph the
// strip draws for it.

export function BeatTitle({ beat }: { beat: BeatWhat }) {
  const Glyph = BEAT_GLYPHS[beat.kind];
  return <IslandRowTitle glyph={<Glyph />} title={beatTitle(beat)} />;
}

function beatTitle(beat: BeatWhat) {
  switch (beat.kind) {
    case "click":
      return t("editor-island-beat-click", { label: beat.label });
    case "type":
      return t("editor-island-beat-type", { label: beat.label });
    case "keys":
      return <PressedKeys keys={beat.keys} />;
    case "switch":
      return t("editor-island-beat-switch", { app: beat.app });
    case "screen":
      return t("editor-island-beat-screen");
    case "sound":
      return t("editor-island-beat-sound");
    case "speech":
      return t("editor-island-beat-speech", { line: beat.line });
  }
}

/** "Press" and the keys pressed together, as keycaps wherever the language
 * puts them. */
function PressedKeys({ keys }: { keys: string[] }) {
  const [before, after] = splitAtPlaceable((mark) =>
    t("editor-island-beat-press", { keys: mark }),
  );
  return (
    <>
      {before}
      <Shortcut className="align-middle">
        {keys.flatMap((key, index) => [
          ...(index > 0 ? (["+"] as const) : []),
          <Keyboard key={key}>{key}</Keyboard>,
        ])}
      </Shortcut>
      {after}
    </>
  );
}
