// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { useGlyphMotion } from "../../../annotations/options/glyph-motion";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { IslandFigure } from "../island-figure";
import { IslandModelLine } from "../island-model-line";
import { IslandWorkLine } from "../island-work-line";

import type { ToolPanelModel } from "../../tools/tool-panel-selection";

/** How loud the clip plays, in dB from the level its kind plays at by
 * default, with its reset while it is off that level. */
export function AudioClipVolumeRow({
  onChange,
  volumeDb,
}: {
  onChange: (volumeDb: number) => void;
  volumeDb: number;
}) {
  return (
    <ControlRow title={t("editor-island-audio-volume")}>
      {(controlProps) => (
        <IslandFigure
          controlProps={controlProps}
          isDefault={volumeDb === 0}
          onReset={() => {
            onChange(0);
          }}
          resetLabel={t("editor-island-audio-reset-volume")}
        >
          <NumberField
            aria-label={t("editor-island-audio-volume")}
            maxValue={12}
            minValue={-60}
            onChange={onChange}
            reservedLength={3}
            rightSection="dB"
            step={1}
            value={volumeDb}
          />
        </IslandFigure>
      )}
    </ControlRow>
  );
}

// Lucide's Mic, with its capsule as a path, and MicOff, which shares its
// stand: the microphone's outline gives way to the broken one and the
// strike drawn across it, last.
const STAND = "M12 19v3";
const SINGING = [
  "M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z",
  "M19 10v2a7 7 0 0 1-14 0v-2",
];
const WITHOUT = [
  "M15 9.34V5a3 3 0 0 0-5.68-1.33",
  "M9 9v3a3 3 0 0 0 5.12 2.12",
  "M16.95 16.95A7 7 0 0 1 5 12v-2",
  "M18.89 13.23A7 7 0 0 0 19 12v-2",
  "m2 2 20 20",
];

/** A microphone, struck out while the song plays without its singing.
 * Changing undraws the one and draws the other in, a part at a time. */
function InstrumentalGlyph({ isInstrumental }: { isInstrumental: boolean }) {
  const motionOf = useGlyphMotion();
  const lines = (paths: string[], shown: boolean) =>
    paths.map((d, index) => (
      <motion.path
        animate={
          shown ? { opacity: 1, pathLength: 1 } : { opacity: 0, pathLength: 0 }
        }
        d={d}
        initial={false}
        key={d}
        transition={{
          ...motionOf.draw,
          delay:
            shown && motionOf.draw.duration !== 0 ? 0.08 + index * 0.04 : 0,
        }}
      />
    ));
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <path d={STAND} />
      {lines(SINGING, !isInstrumental)}
      {lines(WITHOUT, isInstrumental)}
    </svg>
  );
}

/** Whether a song plays without its singing. */
export function InstrumentalToggle({
  isInstrumental,
  onChange,
}: {
  isInstrumental: boolean;
  onChange: (isInstrumental: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <InstrumentalGlyph isInstrumental={isOn} />}
      isOn={isInstrumental}
      label={t("editor-island-audio-instrumental")}
      onChange={onChange}
    />
  );
}

/** What an instrumental song still needs: Studio ducking's model, or its
 * singing separated. */
export function InstrumentalNote({
  isInstrumental,
  isSong,
  model,
  onDownloadModel,
  separating,
}: {
  isInstrumental: boolean;
  isSong: boolean;
  model: ToolPanelModel | null;
  onDownloadModel: () => void;
  /** How far its singing is separated, or null while none is under way. */
  separating: number | null;
}) {
  if (!isSong || !isInstrumental) return null;
  if (model && model.status !== "downloaded")
    return (
      <IslandModelLine
        model={model}
        onDownload={onDownloadModel}
        title={t("editor-island-audio-instrumental")}
      />
    );
  if (separating === null) return null;
  return (
    <IslandWorkLine
      label={t("editor-island-audio-removing-singing")}
      value={separating}
    />
  );
}
