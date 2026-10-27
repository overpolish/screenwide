// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import {
  ListBoxItem as AriaListBoxItem,
  ListBoxItemProps as AriaListBoxItemProps,
} from "react-aria-components";

import { tv } from "../../lib/variants";
import { useSkin } from "../skin/skin";

// An item draws only its highlight, a translucent fill over the panel's
// material with the text unchanged. As in a native menu the highlight is the
// focus indicator: it follows the pointer and the keyboard alike, so there
// is no separate ring. Its corners are the part radius, so they run parallel
// to the panel's own corner, a tight inset out. On the island the last item
// sits in the section's bottom corners, so its own follow them, concentric.
const listBoxItemVariants = tv({
  base: [
    "relative flex h-control-height shrink-0 cursor-default items-center gap-inline truncate rounded-part px-inset-sm text-body text-fg outline-none",
    "data-[focus-visible]:bg-fill data-[hovered]:bg-fill data-[pressed]:bg-fill-strong",
    "transition-opacity data-[disabled]:opacity-40",
  ],
  defaultVariants: { skin: "window" },
  variants: {
    skin: {
      island: "last:rounded-b-(--island-list-radius)",
      window: "",
    },
  },
});

type ListBoxItemProps = AriaListBoxItemProps & {
  children?: React.ReactNode;
  className?: string;
  /**
   * @default true
   * @description
   * Whether the item reserves a leading check gutter. A menu of actions keeps
   * no selection, so it turns this off and the content starts at the inset.
   */
  showsSelection?: boolean;
};

export const ListBoxItem = ({
  children,
  className,
  showsSelection = true,
  ...props
}: ListBoxItemProps) => {
  return (
    <AriaListBoxItem
      {...props}
      className={listBoxItemVariants({ className, skin: useSkin() })}
    >
      {({ isSelected }) =>
        showsSelection ? (
          <>
            {/* The gutter is always present so labels line up whether or not
                the item carries a check, as in a native menu. */}
            <span className="flex w-icon shrink-0 items-center justify-center">
              {isSelected && <Check className="size-icon-sm" />}
            </span>
            <span className="min-w-0 flex-1 truncate">{children}</span>
          </>
        ) : (
          <span className="min-w-0 flex-1 truncate">{children}</span>
        )
      }
    </AriaListBoxItem>
  );
};
