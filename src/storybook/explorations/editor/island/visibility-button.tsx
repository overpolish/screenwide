// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Eye, EyeOff } from "lucide-react";

import { IslandButton } from "../../../../primitives/island/island-controls";

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
  const Icon = isShown ? Eye : EyeOff;
  return (
    <IslandButton aria-label={label} aria-pressed={isShown} onPress={onToggle}>
      <Icon aria-hidden className="size-icon" />
    </IslandButton>
  );
}
