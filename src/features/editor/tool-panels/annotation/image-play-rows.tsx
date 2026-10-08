// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowRightToLine, Repeat } from "lucide-react";

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";

import type { ImagePlay } from "../../annotations/annotations";
import type { EditorKind } from "../../types";
import type { ToolPanelPatch } from "../tool-panel-patch";

/**
 * Which frame of a moving image shows: the one a screenshot keeps, and in
 * a recording the one playback starts on. Counted from one, as a person
 * counts frames; the document counts from zero.
 */
export function ImageFrameRow({
  change,
  isLocked,
  play,
  workspace,
}: {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  play: ImagePlay;
  workspace: EditorKind;
}) {
  const title =
    workspace === "recording"
      ? t("editor-panels-start-frame")
      : t("editor-panels-frame");
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
                annotationImagePlay: { frame: Math.round(next) - 1 },
              });
            }}
            value={play.frame + 1}
          />
        </div>
      )}
    </ControlRow>
  );
}

const playbackItems = () => [
  {
    icon: <Repeat aria-hidden="true" />,
    id: "loop",
    label: t("editor-panels-playback-loop"),
  },
  {
    icon: <ArrowRightToLine aria-hidden="true" />,
    id: "once",
    label: t("editor-panels-playback-once"),
  },
];

/**
 * Whether a recording loops a moving image for as long as its clip lasts,
 * or plays it through once, the clip then lasting exactly one run so it
 * leaves as the run ends.
 */
export function ImagePlaybackRow({
  change,
  isLocked,
  play,
}: {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  play: ImagePlay;
}) {
  return (
    <ControlRow title={t("editor-panels-playback")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <PillGroup
            aria-label={t("editor-panels-playback")}
            display="icon-label"
            isDisabled={isLocked}
            items={playbackItems()}
            onSelectionChange={(id) => {
              change({ annotationImagePlay: { once: id === "once" } });
            }}
            selected={play.once ? "once" : "loop"}
          />
        </div>
      )}
    </ControlRow>
  );
}
