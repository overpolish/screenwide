// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { renderCaptionPreviews } from "../../recording/captions/captions-api";

import type { CaptionPreviewStrip } from "../../../../bindings/CaptionPreviewStrip";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";

/** How long the styles have to stay as they are before they are drawn, so a
 * slider's run of changes draws one preview rather than one per step. */
const SETTLE_MS = 250;

/**
 * Each of `styles` previewed by the compositor, in order, or none until
 * drawn. A strip already drawn for the same style is kept on disk, so only
 * a style never seen before costs a draw; the last strips stay up while the
 * next are drawn.
 */
export function useCaptionPreviews(styles: CaptionStyle[] | null) {
  const [strips, setStrips] = useState<CaptionPreviewStrip[] | null>(null);
  const sample = t("editor-panels-captions-sample");
  const key = styles ? JSON.stringify(styles) : null;
  useEffect(() => {
    if (!key) return;
    let current = true;
    const timer = window.setTimeout(() => {
      void renderCaptionPreviews(JSON.parse(key) as CaptionStyle[], sample)
        .then((drawn) => {
          if (current) setStrips(drawn);
        })
        // A preview is a picture of a setting, not the setting: one that
        // cannot be drawn leaves the tile showing its name alone.
        .catch(() => undefined);
    }, SETTLE_MS);
    return () => {
      current = false;
      window.clearTimeout(timer);
    };
  }, [key, sample]);
  return strips;
}
