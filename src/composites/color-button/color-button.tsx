// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock } from "lucide-react";
import { type ComponentProps, type Ref } from "react";

import { cn } from "../../lib/styling";
import { Button } from "../../primitives/button/button";

// A colour on a button, for choosing it somewhere else: in a pop-up of
// swatches, or the system's picker. With a label it is a neutral button with
// the colour beside its name; without one the button is the colour itself, a
// square at the control height. A hairline edge keeps white and translucent
// colours from losing their outline against the window.
const edge = "inset-ring-1 inset-ring-divider";

type ColorButtonProps = {
  onPress: ComponentProps<typeof Button>["onPress"];
  /** `#rrggbb` or `#rrggbbaa`. */
  value: string;
  /** Whether what the button opens is showing. */
  "aria-expanded"?: boolean;
  isDisabled?: boolean;
  isLocked?: boolean;
  ref?: Ref<HTMLButtonElement>;
} & (
  | {
      /** Shown beside the colour. Leave it out, and give `aria-label`, for
       * the swatch-only button. */
      label: string;
      "aria-label"?: never;
    }
  | { "aria-label": string; label?: never }
);

export function ColorButton({
  "aria-expanded": isExpanded,
  "aria-label": ariaLabel,
  isDisabled,
  isLocked,
  label,
  onPress,
  ref,
  value,
}: ColorButtonProps) {
  // The lock sits on the colour in both forms, white with a shadow so it
  // reads on any colour, sized to the swatch under it. Its size is forced
  // because Button sizes every icon inside it.
  const lock = (size: string) =>
    isLocked ? (
      <Lock
        aria-hidden="true"
        className={cn(
          "absolute text-white drop-shadow-[0_0_2px_rgb(0_0_0/0.8)]",
          size,
        )}
      />
    ) : null;

  if (label === undefined)
    return (
      <Button
        aria-expanded={isExpanded}
        aria-label={ariaLabel}
        className="w-control-height px-0"
        isDisabled={isDisabled}
        onPress={onPress}
        ref={ref}
        variant="ghost"
      >
        <span
          aria-hidden="true"
          className={cn(
            "size-full rounded-control transition-opacity in-data-[pressed]:opacity-80",
            edge,
          )}
          style={{ backgroundColor: value }}
        />
        {lock("size-icon-sm!")}
      </Button>
    );

  return (
    <Button
      aria-expanded={isExpanded}
      isDisabled={isDisabled}
      onPress={onPress}
      ref={ref}
    >
      <span className="relative flex shrink-0 items-center justify-center">
        <span
          aria-hidden="true"
          className={cn("size-icon rounded-sm", edge)}
          style={{ backgroundColor: value }}
        />
        {lock("size-icon-xs!")}
      </span>
      {label}
    </Button>
  );
}
