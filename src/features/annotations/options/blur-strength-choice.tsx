// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useId } from "react";

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { ANNOTATION_BLUR_STRENGTHS } from "../widths";

/** A blur's strength as a dot in the text's colour softened by it, the
 * lightest barely, the strongest to a haze. The filter reaches well past
 * the dot so a strong blur is not cut square at the dot's box. */
function BlurDot({ strength }: { strength: number }) {
  const filterId = useId();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="currentColor"
      viewBox="0 0 24 24"
    >
      <filter height="300%" id={filterId} width="300%" x="-100%" y="-100%">
        <feGaussianBlur stdDeviation={strength * 0.6} />
      </filter>
      <circle cx={12} cy={12} filter={`url(#${filterId})`} r={6} />
    </svg>
  );
}

/** How wide a blurred redaction's blur spreads, in its five steps. */
export function BlurStrengthChoice({
  onChange,
  value,
}: {
  onChange: (strength: number) => void;
  /** The step, 1 the lightest. */
  value: number;
}) {
  return (
    <PillGroup
      aria-label={t("annotation-blur-strength")}
      items={ANNOTATION_BLUR_STRENGTHS.map((strength) => ({
        ariaLabel: t("annotation-blur-strength-step", {
          count: ANNOTATION_BLUR_STRENGTHS.length,
          strength,
        }),
        icon: <BlurDot strength={strength} />,
        id: String(strength),
      }))}
      onSelectionChange={(id) => {
        onChange(Number(id));
      }}
      selected={String(value)}
    />
  );
}
