// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { X } from "lucide-react";
import { AnimatePresence } from "motion/react";

import { cn } from "../../lib/styling";
import { IconButton } from "../button/icon-button";

// The clear button at the end of a field once something is typed: it sits
// in the field's end corner at the field's height, a quiet glyph that comes
// up under the pointer, and takes over the end padding while it shows. In a
// search field React Aria names it and clears the field itself; elsewhere
// the field gives it both.
export function FieldClearButton({
  isDisabled,
  isShown,
  ...props
}: {
  isDisabled: boolean;
  isShown: boolean;
  "aria-label"?: string;
  onPress?: () => void;
}) {
  return (
    <AnimatePresence>
      {isShown ? (
        <IconButton
          {...props}
          animate={{ opacity: 1 }}
          className={cn(
            "shrink-0 text-fg-tertiary data-[hovered]:text-fg",
            "[&_svg.lucide]:size-icon-sm",
          )}
          exit={{ opacity: 0, scale: 0 }}
          initial={{ opacity: 0 }}
          isDisabled={isDisabled}
          variant="ghost"
        >
          <X />
        </IconButton>
      ) : null}
    </AnimatePresence>
  );
}
