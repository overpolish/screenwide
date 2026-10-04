// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowRightToLine, Repeat } from "lucide-react";

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";

import type { StickerPlay } from "../../annotations/annotations";
import type { EditorKind } from "../../types";
import type { ToolPanelPatch } from "../tool-panel-patch";

/**
 * Which frame of a moving sticker shows: the one a screenshot keeps, and in
 * a recording the one playback starts on. Counted from one, as a person
 * counts frames; the document counts from zero.
 */
export function StickerFrameRow({
  change,
  isLocked,
  play,
  workspace,
}: {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  play: StickerPlay;
  workspace: EditorKind;
}) {
  const title = workspace === "recording" ? "Start frame" : "Frame";
  return (
    <ControlRow title={title}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <SliderNumberField
            aria-label={title}
            className="w-48"
            isDisabled={isLocked}
            maxValue={play.frames}
            minValue={1}
            onChange={(next) => {
              change({
                annotationStickerPlay: { frame: Math.round(next) - 1 },
              });
            }}
            value={play.frame + 1}
          />
        </div>
      )}
    </ControlRow>
  );
}

const PLAYBACK_ITEMS = [
  { icon: <Repeat aria-hidden="true" />, id: "loop", label: "Loop" },
  { icon: <ArrowRightToLine aria-hidden="true" />, id: "once", label: "Once" },
];

/**
 * Whether a recording loops a moving sticker for as long as its clip lasts,
 * or plays it through once, the clip then lasting exactly one run so it
 * leaves as the run ends.
 */
export function StickerPlaybackRow({
  change,
  isLocked,
  play,
}: {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  play: StickerPlay;
}) {
  return (
    <ControlRow title="Playback">
      {(controlProps) => (
        <div {...controlProps} role="group">
          <PillGroup
            aria-label="Playback"
            display="icon-label"
            isDisabled={isLocked}
            items={PLAYBACK_ITEMS}
            onSelectionChange={(id) => {
              change({ annotationStickerPlay: { once: id === "once" } });
            }}
            selected={play.once ? "once" : "loop"}
          />
        </div>
      )}
    </ControlRow>
  );
}
