// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import {
  launchTemplatePreviews,
  renderTemplatePreviews,
} from "../../recording/captions/captions-api";

import type { TemplatePreviewClip } from "../../../../bindings/TemplatePreviewClip";
import type { TextStyle } from "../../../../bindings/TextStyle";

/** How long the styles have to stay as they are before they are drawn, so a
 * slider's run of changes draws one preview rather than one per step. */
const SETTLE_MS = 250;

/** The clips drawn this session, by sample and style, so a picker opened
 * again shows the ones it showed before at once. */
const drawn = new Map<string, TemplatePreviewClip>();
const drawnKey = (sample: string, style: TextStyle) =>
  `${sample}\n${JSON.stringify(style)}`;

/**
 * Each of `styles` previewed by the compositor, in order, or none until
 * drawn. A clip already drawn for the same style is kept on disk, so only
 * a style never seen before costs a draw. Clips seen this session show at
 * once, and the last clips stay up while the next are drawn. The first ask
 * goes at once; later ones wait for the styles to settle.
 */
export function useTemplatePreviews(styles: TextStyle[]) {
  const [last, setLast] = useState<readonly TemplatePreviewClip[]>([]);
  // Whether this picker has shown clips, after which a change waits.
  const hasShownRef = useRef(false);
  const sample = t("editor-panels-text-sample");
  const key = JSON.stringify(styles);
  useEffect(() => {
    let current = true;
    const timer = window.setTimeout(
      () => {
        const asked = JSON.parse(key) as TextStyle[];
        void renderTemplatePreviews(asked, sample)
          .then((clips) => {
            clips.forEach((clip, index) => {
              drawn.set(drawnKey(sample, asked[index]), clip);
            });
            hasShownRef.current = true;
            if (current) setLast(clips);
          })
          // A preview is a picture of a setting, not the setting: one that
          // cannot be drawn leaves the tile showing its name alone.
          .catch(() => undefined);
      },
      hasShownRef.current ? SETTLE_MS : 0,
    );
    return () => {
      current = false;
      window.clearTimeout(timer);
    };
  }, [key, sample]);
  return styles.map(
    (style, index): TemplatePreviewClip | undefined =>
      drawn.get(drawnKey(sample, style)) ?? last[index],
  );
}

/**
 * Loads every template's preview, as new captions would take it up, into
 * this window as it starts, where `isWanted`, so the template panel first
 * opens with them ready rather than loading them then. Each still and clip
 * is held for as long as the window runs, so the webview keeps it; a clip
 * decodes a frame at a time, so that costs little.
 */
export function usePreloadedTemplatePreviews(isWanted: boolean) {
  const heldRef = useRef<HTMLImageElement[]>([]);
  const sample = t("editor-panels-text-sample");
  useEffect(() => {
    if (!isWanted) return;
    let current = true;
    void launchTemplatePreviews(sample)
      .then((clips) => {
        if (!current) return;
        heldRef.current = clips
          .flatMap((clip) => [clip.still, clip.path])
          .map((path) => {
            const image = new Image();
            image.src = convertFileSrc(path);
            void image.decode().catch(() => undefined);
            return image;
          });
      })
      // Loading ahead is only a head start: the panel loads what is missing.
      .catch(() => undefined);
    return () => {
      current = false;
      heldRef.current = [];
    };
  }, [isWanted, sample]);
}
