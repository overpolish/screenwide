// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioLines, Music, Repeat } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { StackBar, type StackBarPlacement } from "../bar-stack/stack-bar";
import { type Span } from "../timeline-scale";

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";

// A song or sound effect across the time it plays (`stack-bar.tsx`): its
// glyph and title in the middle of the line, with no fill of their own, the
// line fading out round them as a lull's wire fades round its glyphs. A
// title longer than the bar fades out where it is cut rather than ending
// in an ellipsis, and a bar too short for any of it keeps its glyph.

/** How far, in px, the line fades out before the title. */
const TITLE_FADE_PX = 8;
/** How much of the bar's width, in px, the glyphs and the room round them
 * take, so what is left is the title's. */
const GLYPHS_PX = 24;
/** The least room, in px, worth showing a title in. */
const LEAST_TITLE_PX = 16;

export type TimelineAudioClip = Span & {
  id: string;
  isLooped: boolean;
  kind: AudioClipKind;
  /** Its lane in the stack, 0 at the top. */
  lane: number;
  /** Its sound's title, or the name of the file it came from. */
  title: string;
};

export function AudioBar({
  clip,
  ...placement
}: StackBarPlacement & { clip: TimelineAudioClip }) {
  const Glyph = clip.kind === "music" ? Music : AudioLines;
  return (
    <StackBar
      {...placement}
      badge={(px) => (
        <span
          className={cn(
            "flex items-center gap-1 text-caption whitespace-nowrap transition-colors [&_svg]:size-icon-sm [&_svg]:shrink-0",
            placement.isSelected
              ? "text-fg"
              : "text-fg-secondary group-hover:text-fg",
          )}
        >
          <Glyph />
          {clip.isLooped ? <Repeat /> : null}
          {px - GLYPHS_PX >= LEAST_TITLE_PX ? (
            <FadingTitle maxWidth={px - GLYPHS_PX} title={clip.title} />
          ) : null}
        </span>
      )}
      badgeFade={TITLE_FADE_PX}
      label={
        clip.kind === "music"
          ? t("editor-timeline-audio-song", { title: clip.title })
          : t("editor-timeline-audio-effect", { title: clip.title })
      }
    />
  );
}

/** The title, at most `maxWidth` px wide, fading out at its end only where
 * it is cut. */
function FadingTitle({ maxWidth, title }: { maxWidth: number; title: string }) {
  const ref = useRef<HTMLSpanElement>(null);
  const [isCut, setIsCut] = useState(false);
  useLayoutEffect(() => {
    const element = ref.current;
    if (element) setIsCut(element.scrollWidth > element.clientWidth);
  }, [maxWidth, title]);
  return (
    <span
      className="overflow-hidden"
      dir="auto"
      ref={ref}
      style={{
        maskImage: isCut
          ? `linear-gradient(to right, #000 calc(100% - ${String(TITLE_FADE_PX * 2)}px), transparent)`
          : undefined,
        maxWidth,
      }}
    >
      {title}
    </span>
  );
}
