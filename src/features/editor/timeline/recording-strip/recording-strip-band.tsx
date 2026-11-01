// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { ChromeSetBack } from "../../panel/chrome-set-back";
import {
  EditorTransport,
  type EditorTransportProps,
} from "../editor-transport";
import { TimelineStrip } from "../strip/timeline-strip";
import { type TimelineStripProps } from "../strip/timeline-strip-props";

// Under the picture, laid out as the row above it: the transport in the
// tool column's width, the strip under the picture. It keeps the window's
// inset from the bottom edge, as the tool column's glyphs keep it from the
// leading one. While the Export island stands over the editor the strip is
// set back and nothing on it is edited, but the recording still plays and
// its playhead still scrubs, so the export can be looked through.

export function RecordingStripBand({
  isLocked = false,
  strip,
  transport,
}: {
  strip: TimelineStripProps;
  transport: EditorTransportProps;
  /** Nothing on the strip is edited: only playing and scrubbing answer. */
  isLocked?: boolean;
}) {
  return (
    <section
      aria-label={t("editor-timeline-label")}
      className="flex shrink-0 gap-inline pb-inset"
    >
      <EditorTransport {...transport} isLocked={isLocked} />
      <ChromeSetBack
        className="flex min-w-0 grow"
        isSetBack={isLocked}
        keepsPointer
      >
        <TimelineStrip {...strip} isLocked={isLocked} />
      </ChromeSetBack>
    </section>
  );
}
