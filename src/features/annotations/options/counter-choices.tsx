// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { AngleDial } from "../../../primitives/angle-dial/angle-dial";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { dialDegrees, radiansFromDial } from "../annotation-angle";
import { annotationSizeLabel } from "../annotation-text";
import { annotationSizes } from "../widths";

import { CounterPinIcon } from "./counter-pin-icon";
import { SizeDot } from "./size-dot";

/** A counter's disc sizes as dots, a quarter of each disc's diameter. */
export function CounterSizeChoice({
  onChange,
  value,
}: {
  onChange: (points: number) => void;
  /** The disc's size in points. */
  value: number;
}) {
  return (
    <PillGroup
      aria-label={annotationSizeLabel("size")}
      items={annotationSizes("counter").map((points) => ({
        ariaLabel: t("annotate-island-points", { points }),
        icon: <SizeDot diameter={points / 4} />,
        id: String(points),
      }))}
      onSelectionChange={(id) => {
        onChange(Number(id));
      }}
      selected={String(value)}
    />
  );
}

/** Where a counter's tail points, as a dial stepped in eighths, the pin
 * turning with it. */
export function CounterAngleDial({
  onChange,
  value,
}: {
  onChange: (radians: number) => void;
  /** The tail's direction in radians. */
  value: number;
}) {
  return (
    <AngleDial
      aria-label={t("annotation-angle")}
      isStepped
      onChange={(degrees) => {
        onChange(radiansFromDial(degrees));
      }}
      value={dialDegrees(value)}
    >
      <CounterPinIcon className="size-icon" />
    </AngleDial>
  );
}
