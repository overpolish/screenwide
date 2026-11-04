// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Ref } from "react";

import { JumpMarks } from "./jump-marks";
import { SplitMarks } from "./split-marks";
import { StripBeatRow, type StripBeatRowProps } from "./strip-beat-row";
import { type TakenOut } from "./taken-out";
import { TakenOutBand } from "./taken-out-band";
import { type StripReshape } from "./use-strip-reshape";

// The strip's row of beats and lulls with what is marked on it: where play
// jumps, where it is split, and past its end what was taken out. While
// the strip is reshaped, what is new fades in and what went fades out.

/** How a reshape's new beats and lulls fade in where they land, gliding as
 * the rest do. */
const ARRIVE =
  "*:transition-[left,right,width,transform,translate,opacity]! *:starting:opacity-0";
/** How a reshape's beats and lulls that went fade out where they were. */
const LEAVE =
  "pointer-events-none absolute inset-y-0 start-0 opacity-0 transition-opacity! starting:opacity-100";

const nothing = () => undefined;

export function StripBeats({
  jumps,
  onPutBack,
  onRemoveSplit,
  pxPerSecond,
  reshape,
  rowRef,
  splits,
  taken,
  ...row
}: Omit<StripBeatRowProps, "className"> & {
  /** The recorded moments where play jumps. */
  jumps: number[];
  onPutBack: (at: number) => void;
  onRemoveSplit: (at: number) => void;
  /** How wide a second is drawn, in px. */
  pxPerSecond: number;
  reshape: StripReshape;
  /** The row what is lifted is drawn in, which carries its offset. */
  rowRef: Ref<HTMLDivElement>;
  /** The recorded moments the recording is split at. */
  splits: number[];
  /** What is taken out of the recording, drawn past the end. */
  taken: TakenOut;
}) {
  const { leaving } = reshape;
  return (
    // A spring's badge, on the top edge of its box, reaches a few px above
    // the beats.
    <div className="relative mt-1" ref={rowRef}>
      <StripBeatRow
        {...row}
        className={reshape.isReshaping ? ARRIVE : undefined}
      />
      {leaving ? (
        <div
          aria-hidden
          className={LEAVE}
          inert
          style={{ width: leaving.width }}
        >
          <StripBeatRow
            beats={leaving.beats}
            captions={null}
            cuts={[]}
            isAltHeld={false}
            lifted={null}
            lulls={leaving.lulls}
            map={row.map}
            onBeatTrim={nothing}
            onGrab={nothing}
            onLullPlayChange={nothing}
            onLullTrim={nothing}
            onRestore={nothing}
            onSelect={nothing}
            pictureAspect={row.pictureAspect}
            scale={leaving.scale}
            selection={null}
            width={leaving.width}
          />
        </div>
      ) : null}
      <JumpMarks jumps={jumps} onPutBack={onPutBack} scale={row.scale} />
      <SplitMarks onRemove={onRemoveSplit} scale={row.scale} splits={splits} />
      <TakenOutBand from={row.width} pxPerSecond={pxPerSecond} taken={taken} />
    </div>
  );
}
