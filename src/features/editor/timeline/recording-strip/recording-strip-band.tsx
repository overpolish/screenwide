// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import {
  type PopupMenuAnchor,
  pointerAnchor,
} from "../../../popup-panel/use-popup-menu";
import { ChromeSetBack } from "../../panel/chrome-set-back";
import {
  EditorTransport,
  type EditorTransportProps,
} from "../editor-transport";
import { TimelineStrip } from "../strip/timeline-strip";
import { type TimelineStripProps } from "../strip/timeline-strip-props";

// Under the picture and the tool column, across the window: the strip, and
// under its minimap the transport in a row, set apart from both the minimap
// and the window's bottom edge. The row keeps the window's inset from the
// start edge, lining up with the strip and minimap, which keep it inside
// their own padding. It is only as tall as the strip and that row. While
// the Export island stands over the editor the strip is set back and nothing
// on it is edited, but the recording still plays and its playhead still
// scrubs, so the export can be looked through. Making a GIF, the strip stays
// in focus, as its range is what the GIF is made of.

export function RecordingStripBand({
  isLocked = false,
  onMenu,
  strip,
  transport,
}: {
  strip: TimelineStripProps;
  transport: EditorTransportProps;
  /** Nothing on the strip is edited: only playing and scrubbing answer. */
  isLocked?: boolean;
  /** Opens the strip's menu under the point pressed. */
  onMenu?: (anchor: PopupMenuAnchor) => void;
}) {
  return (
    <section
      aria-label={t("editor-timeline-label")}
      // The room above it, which the editor leaves under the picture, is
      // taken into the strip's top, so a press there scrubs
      // (`strip-scrub-band.tsx`).
      className="-mt-separate flex shrink-0 flex-col gap-separate pb-inset"
      // Sounds dragged in are dropped here (`use-sound-drag.ts`).
      data-sound-drop-area
    >
      <ChromeSetBack
        className="flex min-w-0 grow"
        isSetBack={isLocked && !transport.isGifRange}
        keepsPointer
      >
        {/* Making a GIF, a press here draws its range rather than putting
          the Export island away (`use-export-light-dismiss.ts`). */}
        <div
          className="contents"
          data-gif-range
          // A right press anywhere on the strip offers to add the frame
          // under the playhead to a script (`use-editor-cue.ts`).
          onContextMenu={
            onMenu &&
            ((event) => {
              event.preventDefault();
              onMenu(pointerAnchor(event.clientX, event.clientY));
            })
          }
        >
          <TimelineStrip {...strip} isLocked={isLocked} />
        </div>
      </ChromeSetBack>
      <div className="ps-inset">
        <EditorTransport {...transport} isLocked={isLocked} />
      </div>
    </section>
  );
}
