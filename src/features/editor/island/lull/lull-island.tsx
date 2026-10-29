// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FastForward, Pin, Scissors, Snail } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IslandKeyHint } from "../../../../primitives/island/island-key-hint";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { SpringIcon } from "../../glyphs/spring-icon";

import { LullChoices } from "./lull-parts";

import type { EditorIslandContent } from "../editor-island";

// A lull on the strip, in hand: a stretch with nothing in it worth watching
// at full speed, whether the speaker stopped, a page was loading or the
// pointer sat idle. It squeezes with the rest unless it is kept as
// recorded; dragging its end gives it a speed of its own, which then shows
// as a third choice.

/** How a lull plays. A retimed lull's `speed` is how many times as fast as
 * recorded, and never 1: a drag that ends at the recorded length keeps it
 * as recorded instead. */
type LullPlay =
  { kind: "keep" } | { kind: "retimed"; speed: number } | { kind: "squeeze" };

export type LullIslandProps = {
  id: string;
  onCut: () => void;
  onPlayChange: (play: "keep" | "squeeze") => void;
  onTrimToBreath: () => void;
  play: LullPlay;
};

export function lullIsland(props: LullIslandProps): EditorIslandContent {
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-lull-confirm-cut")}
        icon={<Scissors />}
        label={t("editor-island-lull-cut")}
        onConfirm={props.onCut}
        variant="ghost"
      />
    ),
    // Its note keeps to the start, on the island's text column.
    anchor: "start",
    below: <LullChoices {...props} />,
    key: `lull-${props.id}`,
    // The key that turns dragging the lull's end from a retime into a trim.
    section: (
      <IslandKeyHint
        binding="Alt"
        message={(key) => t("editor-island-lull-hint", { key })}
      />
    ),
    title: (
      <IslandRowTitle
        glyph={playGlyph(props.play)}
        glyphKey={playGlyphKey(props.play)}
        title={t("editor-island-lull")}
      />
    ),
  };
}

/** How the lull plays, at a glance: a spring while it squeezes with the
 * rest, a pin when kept as recorded, and with a speed of its own, which way
 * that goes. */
function playGlyph(play: LullPlay) {
  switch (playGlyphKey(play)) {
    case "keep":
      return <Pin />;
    case "squeeze":
      return <SpringIcon />;
    case "faster":
      return <FastForward />;
    case "slower":
      return <Snail />;
  }
}

function playGlyphKey(play: LullPlay) {
  if (play.kind !== "retimed") return play.kind;
  return play.speed > 1 ? "faster" : "slower";
}
