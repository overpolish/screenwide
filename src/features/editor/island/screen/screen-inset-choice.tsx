// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";

// How far the picture's edges are extended, as four looks rather than a
// figure: a padding is judged by eye against the picture, and a share of
// the picture's shorter side looks the same at any size where pixels would
// not.

const INSETS = [
  {
    id: "none",
    label: () => t("editor-island-screen-inset-none"),
    share: 0,
    stroke: 0,
  },
  {
    id: "tight",
    label: () => t("editor-island-screen-inset-tight"),
    share: 0.03,
    stroke: 1.5,
  },
  {
    id: "medium",
    label: () => t("editor-island-screen-inset-medium"),
    share: 0.06,
    stroke: 2.5,
  },
  {
    id: "large",
    label: () => t("editor-island-screen-inset-large"),
    share: 0.1,
    stroke: 3.5,
  },
] as const;

/** The glyph's outer edge, in its own 24 units, and the room between the
 * padding and the picture, so the two read as separate parts. */
const FRAME = { at: 4, gap: 1.25, side: 16 };

/** The picture as a filled box, with the padding round it as a border as
 * heavy as the padding is wide. The border thickens inwards, so the four
 * glyphs share one outline and the picture shrinks inside it; with no
 * padding the picture fills the outline. Filled, it does not read as the
 * hollow corner glyphs beside it. */
function InsetGlyph({ stroke }: { stroke: number }) {
  const picture = stroke === 0 ? 0 : stroke + FRAME.gap;
  return (
    <svg aria-hidden fill="none" stroke="currentColor" viewBox="0 0 24 24">
      {stroke === 0 ? null : (
        <rect
          height={FRAME.side - stroke}
          rx={Math.max(0.5, 3 - stroke / 2)}
          strokeWidth={stroke}
          width={FRAME.side - stroke}
          x={FRAME.at + stroke / 2}
          y={FRAME.at + stroke / 2}
        />
      )}
      <rect
        fill="currentColor"
        height={FRAME.side - picture * 2}
        rx={1}
        stroke="none"
        width={FRAME.side - picture * 2}
        x={FRAME.at + picture}
        y={FRAME.at + picture}
      />
    </svg>
  );
}

/** The padding as one of four looks. A padding set to another value
 * elsewhere leaves none chosen until one is picked. */
export function ScreenInsetChoice({
  inset,
  insetMaximum,
  onChange,
}: {
  /** In output pixels. */
  inset: number;
  /** The picture's shorter side, in output pixels, which the looks are
   * shares of. */
  insetMaximum: number;
  onChange: (inset: number) => void;
}) {
  const pixels = (share: number) => Math.round(share * insetMaximum);
  return (
    <PillGroup
      aria-label={t("editor-island-screen-inset")}
      items={INSETS.map(({ id, label, stroke }) => ({
        ariaLabel: label(),
        icon: <InsetGlyph stroke={stroke} />,
        id,
      }))}
      onSelectionChange={(id) => {
        const chosen = INSETS.find((look) => look.id === id);
        if (chosen) onChange(pixels(chosen.share));
      }}
      selected={INSETS.find(({ share }) => pixels(share) === inset)?.id ?? ""}
    />
  );
}
