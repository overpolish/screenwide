// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import {
  EditorTransport,
  type EditorTransportProps,
} from "../editor-transport";
import { TimelineStrip } from "../strip/timeline-strip";
import { type TimelineStripProps } from "../strip/timeline-strip-props";

// Under the picture, laid out as the row above it: the transport in the
// tool column's width, the strip under the picture. It keeps the window's
// inset from the bottom edge, as the tool column's glyphs keep it from the
// leading one.

export function RecordingStripBand({
  strip,
  transport,
}: {
  strip: TimelineStripProps;
  transport: EditorTransportProps;
}) {
  return (
    <section
      aria-label={t("editor-timeline-label")}
      className="flex shrink-0 gap-inline pb-inset"
    >
      <EditorTransport {...transport} />
      <TimelineStrip {...strip} />
    </section>
  );
}
