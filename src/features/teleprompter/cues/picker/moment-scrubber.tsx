// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import {
  Slider as AriaSlider,
  SliderThumb,
  SliderTrack,
} from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import {
  Minimap,
  type MinimapProps,
} from "../../../editor/timeline/minimap/minimap";

import type { TimeMap } from "../../../editor/timeline/timeline-scale";

/** Spoken as seconds into what plays. */
const SECONDS = {
  maximumFractionDigits: 1,
  style: "unit",
  unit: "second",
} as const;

// The recording's overview, the one its card and the timeline show, pressed
// or dragged along to scrub, or stepped with the arrow keys. The overview
// draws the playhead, so the thumb is only where focus lands, ringed while
// the keyboard moves it.
export function MomentScrubber({
  beats,
  map,
  onPlayedChange,
  played,
}: {
  beats: MinimapProps["beats"];
  map: TimeMap;
  onPlayedChange: (played: number) => void;
  /** In played seconds. */
  played: number;
}) {
  return (
    <AriaSlider
      aria-label={t("teleprompter-cue-moment")}
      className="w-full"
      formatOptions={SECONDS}
      maxValue={Math.max(map.duration, 0.1)}
      minValue={0}
      onChange={onPlayedChange}
      step={0.1}
      value={played}
    >
      <SliderTrack className="relative cursor-ew-resize py-inline" dir="ltr">
        {({ state }) => (
          <>
            <Minimap
              at={state.getThumbPercent(0)}
              beats={beats}
              isPicture
              map={map}
            />
            <SliderThumb
              className={cn(
                "top-1/2 h-4 w-1 rounded-full",
                focusStyles,
                elementFocusVisible,
              )}
            />
          </>
        )}
      </SliderTrack>
    </AriaSlider>
  );
}
