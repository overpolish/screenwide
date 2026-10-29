// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { useReducedMotion } from "motion/react";
import { useEffect, useState } from "react";

import { cn } from "../../../../lib/styling";

import type { TemplatePreviewClip } from "../../../../bindings/TemplatePreviewClip";

/**
 * A caption style's preview, playing over and over: the clip the compositor
 * drew, so the tile is true to the export. The tile shows the clip's still
 * at once and moves on to the clip, which starts from that frame, once it
 * is decoded. Held on the still where motion is reduced, or where `still`
 * says the text it dresses never moves, without loading the clip at all.
 */
export function TemplatePreviewClipView({
  clip,
  still,
}: {
  clip: TemplatePreviewClip | undefined;
  still: boolean;
}) {
  const reducedMotion = useReducedMotion() ?? false;
  const clipUrl =
    clip && !reducedMotion && !still ? convertFileSrc(clip.path) : null;
  const [decoded, setDecoded] = useState<string | null>(null);
  useEffect(() => {
    if (!clipUrl) return;
    let current = true;
    const image = new Image();
    image.src = clipUrl;
    image.decode().then(
      () => {
        if (current) setDecoded(clipUrl);
      },
      // A clip that cannot be decoded leaves the tile on its still.
      () => undefined,
    );
    return () => {
      current = false;
    };
  }, [clipUrl]);
  const shown = clip
    ? clipUrl !== null && decoded === clipUrl
      ? clipUrl
      : convertFileSrc(clip.still)
    : null;
  return (
    <span
      aria-hidden
      // Until its still is drawn, the tile pulses where it will play.
      className={cn(
        "block aspect-video w-full rounded-control bg-fill bg-cover bg-no-repeat",
        !clip && "animate-pulse motion-reduce:animate-none",
      )}
      style={shown ? { backgroundImage: `url("${shown}")` } : undefined}
    />
  );
}
