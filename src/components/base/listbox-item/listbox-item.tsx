// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import {
  ListBoxItem as AriaListBoxItem,
  ListBoxItemProps as AriaListBoxItemProps,
} from "react-aria-components";

import { tv } from "../../../lib/variants";

const listBoxItemVariants = tv({
  base: [
    "inline-flex h-control-height shrink-0 cursor-default items-center gap-control-inset truncate rounded-control bg-transparent px-control-inset text-body text-content-fg outline-none transition-colors",
    // As in a native menu, the accent highlight is the focus indicator: it
    // follows the pointer and the keyboard alike, so there is no separate
    // focus ring.
    "data-[hovered]:bg-primary-surface data-[hovered]:text-primary-fg",
    "data-[focus-visible]:bg-primary-surface data-[focus-visible]:text-primary-fg",
    "data-[pressed]:bg-primary-surface data-[pressed]:text-primary-fg",
    "data-[disabled]:bg-transparent data-[disabled]:text-content-fg-tertiary",
  ],
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
    <AriaListBoxItem {...props} className={listBoxItemVariants({ className })}>
      {({ isSelected }) =>
        showsSelection ? (
          <>
            {/* The gutter is always present so labels line up whether or not
                the item carries a checkmark, as in a native menu. */}
            <span className="flex w-icon shrink-0 items-center justify-center">
              {isSelected && <Check className="size-icon-small" />}
            </span>
            <span className="truncate">{children}</span>
          </>
        ) : (
          <span className="truncate">{children}</span>
        )
      }
    </AriaListBoxItem>
  );
};
