// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ClipboardCheck,
  FolderCheck,
  ImageDown,
  PictureInPicture2,
  X,
} from "lucide-react";
import { type ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { Island } from "../../../primitives/island/island";
import { useAfterLeave } from "../../../primitives/island/island-after-leave";
import { IslandParts } from "../../../primitives/island/island-parts";
import { type IslandProps } from "../../../primitives/island/island-props";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";

import type { QuickScreenshotDestination } from "../../../bindings/QuickScreenshotDestination";

// Quick Screenshot on the island, from the overlay waiting for an area to
// the shot copied. While an area is drawn: what it becomes at the row's
// start, with the glyph the recording island's screenshot choices use for
// it, and a way to cancel at its end, as Text Recognition's island has. The
// other shortcut, pressed meanwhile, changes what it becomes, and the words
// give way to the new ones. Once copied, the same island reshapes: Copied at
// the start, Save and Edit before the close at the end, and the picture
// hanging beneath at the row's width. The drawing row fades out on the
// island's shape as it was, and the actions and picture take their room only
// once it has gone, so the black reshapes once, round what comes next,
// rather than while the old words are still on it. Save keeps it in
// Projects and Edit opens it in the editor; either way it stays on the
// clipboard. Saved, the row says so and its actions go, as there is nothing
// more to do.

/** A shot just copied, as the island shows it. */
export type CopiedShot = {
  /** The picture's shown height, in points. */
  height: number;
  isSaved: boolean;
  /** The picture, scaled down. */
  src: string;
  /** The picture's shown width, in points. */
  width: number;
};

export type QuickScreenshotIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onHoverChange" | "onShapeChange"
> & {
  destination: QuickScreenshotDestination;
  /** Ends the session without a screenshot, or puts the copied shot away. */
  onClose: () => void;
  /** The shot just copied; while none, an area is being drawn. */
  copied?: CopiedShot | null;
  /** A hint under the title while an area is drawn, such as which keys now
   * open it, at the row's width. */
  note?: ReactNode;
  onEdit?: () => void;
  onSave?: () => void;
};

const TITLES: Record<QuickScreenshotDestination, () => string> = {
  reference: () => t("screenshots-quick-reference"),
  screenshot: () => t("screenshots-quick-screenshot"),
};

const GLYPHS: Record<QuickScreenshotDestination, typeof ImageDown> = {
  // A picture standing over the screen, as a reference does.
  reference: PictureInPicture2,
  screenshot: ImageDown,
};

function title(
  copied: CopiedShot | null | undefined,
  destination: QuickScreenshotDestination,
) {
  if (!copied) {
    const Glyph = GLYPHS[destination];
    return {
      glyph: <Glyph aria-hidden />,
      key: destination,
      text: TITLES[destination](),
    };
  }
  return copied.isSaved
    ? {
        glyph: <FolderCheck aria-hidden />,
        key: "saved",
        text: t("screenshots-copied-saved"),
      }
    : {
        glyph: <ClipboardCheck aria-hidden />,
        key: "copied",
        text: t("screenshots-copied"),
      };
}

export function QuickScreenshotIsland({
  copied,
  destination,
  isShown = true,
  notch,
  note,
  onClose,
  onEdit,
  onHidden,
  onHoverChange,
  onSave,
  onShapeChange,
}: QuickScreenshotIslandProps) {
  const shown = title(copied, destination);
  // The shot is rebuilt with every render, so its arrival is timed by
  // whether there is one. Only the arrival waits: going back to drawing, the
  // actions and picture fade out at once, with the title.
  const hasArrived = useAfterLeave(Boolean(copied), { isShown });
  const picture = hasArrived ? copied : null;
  return (
    <Island
      // Hung from the end, the section holds what it shows to the row's
      // width, so the note wraps across the island from the text column.
      anchor={copied ? "center" : "end"}
      aria-label={t("screenshots-quick-island")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onHoverChange={onHoverChange}
      onShapeChange={onShapeChange}
      row={
        <IslandRow
          end={
            <IslandParts
              className="flex items-center gap-separate"
              direction="row"
            >
              {picture && !picture.isSaved ? (
                <span className="flex items-center gap-1" key="actions">
                  <Button onPress={onSave}>
                    {t("screenshots-copied-save")}
                  </Button>
                  <Button color="primary" onPress={onEdit}>
                    {t("screenshots-copied-edit")}
                  </Button>
                </span>
              ) : null}
              <IconButton
                aria-label={
                  copied
                    ? t("screenshots-copied-dismiss")
                    : t("screenshots-quick-cancel")
                }
                className="text-fg-secondary enabled:hover:text-fg"
                // Its own part in each state, so it fades with the row it
                // belongs to and comes back with the next, rather than
                // gliding across as the island reshapes.
                key={copied ? "close-copied" : "close-drawing"}
                onPress={onClose}
              >
                <X aria-hidden className="size-icon" />
              </IconButton>
            </IslandParts>
          }
          start={
            <IslandParts direction="row">
              <IslandRowTitle
                glyph={shown.glyph}
                key={shown.key}
                title={<span role="status">{shown.text}</span>}
              />
            </IslandParts>
          }
        />
      }
      section={
        picture || note ? (
          // Revealed as a part arriving is, once the black has mostly grown
          // round it, rather than as the section first opens. The note takes
          // the width the row sets rather than setting one; the picture sets
          // its own.
          <IslandParts
            className={cn("flex flex-col", !picture && "w-0 min-w-full")}
            initial
          >
            {picture ? (
              <div className="flex justify-center" key="picture">
                <img
                  alt=""
                  className="rounded-part"
                  draggable={false}
                  src={picture.src}
                  style={{ height: picture.height, width: picture.width }}
                />
              </div>
            ) : (
              <div key="note">{note}</div>
            )}
          </IslandParts>
        ) : undefined
      }
      sectionSpansRow
    />
  );
}
