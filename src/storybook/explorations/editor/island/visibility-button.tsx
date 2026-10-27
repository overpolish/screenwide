// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Eye, EyeOff } from "lucide-react";

import { IconToggleButton } from "../../../../primitives/button/icon-button";

/** Whether an overlay over the whole recording shows, as an eye: open while
 * it shows, struck through while it is hidden. */
export function VisibilityButton({
  isShown,
  label,
  onToggle,
}: {
  isShown: boolean;
  label: string;
  onToggle: () => void;
}) {
  return (
    <IconToggleButton
      aria-label={label}
      isSelected={isShown}
      off={<EyeOff aria-hidden className="size-icon" />}
      onChange={onToggle}
    >
      <Eye aria-hidden className="size-icon" />
    </IconToggleButton>
  );
}
