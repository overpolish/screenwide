// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { Check, ChevronLeft } from "lucide-react";
import { useLayoutEffect, useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { IconButton } from "../../../../primitives/button/icon-button";
import { Text } from "../../../../primitives/text/text";
import { StripFrame } from "../../../project-browser/card/strip-frame";

import { MomentScrubber } from "./moment-scrubber";

import type { DrawnMoment } from "./use-cue-moment";
import type { TeleprompterCueTiming } from "../../../../bindings/TeleprompterCueTiming";
import type { MinimapProps } from "../../../editor/timeline/minimap/minimap";
import type { TimeMap } from "../../../editor/timeline/timeline-scale";
import type { ScrubStrip } from "../../../project-browser/types";

// A recording's moments, to scrub along what plays to the one a cue shows
// and confirm it: a line with the way back, the recording's name and the
// confirming tick, the moment as the edit shows it, and the recording's
// overview to scrub along. The latest drawing stays up while the next is
// drawn; the scrub strip stands in only until the first arrives.
export function CueMoment({
  beats,
  drawn,
  hasFailed,
  isMaking,
  map,
  onBack,
  onConfirm,
  onPlayedChange,
  played,
  strip,
  title,
}: {
  beats: MinimapProps["beats"];
  drawn: DrawnMoment | null;
  hasFailed: boolean;
  isMaking: boolean;
  /** How the recording plays now. */
  map: TimeMap;
  onBack: () => void;
  onConfirm: (timing: TeleprompterCueTiming) => void;
  onPlayedChange: (played: number) => void;
  /** The moment scrubbed to, in played seconds. */
  played: number;
  strip: ScrubStrip | null;
  /** The recording's name. */
  title: string;
}) {
  const share = map.duration > 0 ? played / map.duration : 0;
  // The picture takes the recording's shape, from its drawing or else its
  // strip, so the panel fits it with no bands beside it.
  const aspect = drawn
    ? drawn.picture.width / drawn.picture.height
    : strip
      ? strip.frameWidth / strip.frameHeight
      : 16 / 9;
  return (
    <div className="flex flex-col gap-separate p-inset-sm">
      <div className="flex items-center gap-inline">
        <IconButton aria-label={t("teleprompter-cue-back")} onPress={onBack}>
          <ChevronLeft aria-hidden className="size-icon rtl:-scale-x-100" />
        </IconButton>
        <Text
          as="h2"
          className="min-w-0 flex-1 truncate"
          dir="auto"
          variant="heading"
        >
          {title}
        </Text>
        <IconButton
          aria-label={t("teleprompter-cue-add")}
          color="primary"
          isDisabled={isMaking}
          onPress={() => {
            onConfirm({
              playedMs: played * 1000,
              sourceMs: map.toRecorded(played) * 1000,
            });
          }}
        >
          <Check aria-hidden className="size-icon" />
        </IconButton>
      </div>
      <div
        className="relative w-full overflow-hidden rounded-control"
        style={{ aspectRatio: String(aspect) }}
      >
        {drawn ? (
          <MomentPicture picture={drawn.picture} />
        ) : strip ? (
          <StripFrame
            frame={Math.min(strip.frames - 1, Math.floor(share * strip.frames))}
            strip={strip}
          />
        ) : null}
      </div>
      <MomentScrubber
        beats={beats}
        map={map}
        onPlayedChange={onPlayedChange}
        played={played}
      />
      {hasFailed ? (
        <Alert color="error">{t("teleprompter-cue-failed")}</Alert>
      ) : null}
    </div>
  );
}

/** A drawn moment, filling its box. */
function MomentPicture({ picture }: { picture: ImageBitmap }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  useLayoutEffect(() => {
    canvasRef.current?.getContext("2d")?.drawImage(picture, 0, 0);
  }, [picture]);
  return (
    <canvas
      className="absolute inset-0 size-full"
      height={picture.height}
      ref={canvasRef}
      width={picture.width}
    />
  );
}
