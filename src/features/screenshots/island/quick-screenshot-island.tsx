// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clipboard, ImageDown, X } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { IconButton } from "../../../primitives/button/icon-button";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandParts } from "../../../primitives/island/island-parts";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";

import type { QuickScreenshotDestination } from "../../../bindings/QuickScreenshotDestination";

// Quick Screenshot on the island, out while its overlay waits for an area:
// where the area drawn will go at the row's start, with the glyph the
// recording island's screenshot choices use for it, and a way to cancel at
// its end, as Text Recognition's island has while an area is drawn. The
// other shortcut, pressed meanwhile, changes where it goes, and the words
// give way to the new ones.

export type QuickScreenshotIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
> & {
  destination: QuickScreenshotDestination;
  /** Ends the session without a screenshot, as Escape does. */
  onClose: () => void;
};

const TITLES: Record<QuickScreenshotDestination, () => string> = {
  clipboard: () => t("screenshots-quick-clipboard"),
  editor: () => t("screenshots-quick-editor"),
};

const GLYPHS: Record<QuickScreenshotDestination, typeof ImageDown> = {
  clipboard: Clipboard,
  editor: ImageDown,
};

export function QuickScreenshotIsland({
  destination,
  isShown = true,
  notch,
  onClose,
  onHidden,
  onShapeChange,
}: QuickScreenshotIslandProps) {
  const Glyph = GLYPHS[destination];
  return (
    <Island
      aria-label={t("screenshots-quick-island")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={
        <IslandRow
          end={
            <IconButton
              aria-label={t("screenshots-quick-cancel")}
              className="text-fg-secondary enabled:hover:text-fg"
              onPress={onClose}
            >
              <X aria-hidden className="size-icon" />
            </IconButton>
          }
          start={
            <IslandParts direction="row">
              <IslandRowTitle
                glyph={<Glyph aria-hidden />}
                key={destination}
                title={<span role="status">{TITLES[destination]()}</span>}
              />
            </IslandParts>
          }
        />
      }
    />
  );
}
