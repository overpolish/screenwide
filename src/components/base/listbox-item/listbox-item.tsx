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
    "relative flex h-control-height shrink-0 cursor-default items-center gap-control-inset truncate rounded-control bg-transparent px-control-inset text-body text-content-fg outline-none transition-colors",
    // As in a native menu, the accent highlight is the focus indicator: it
    // follows the pointer and the keyboard alike, so there is no separate
    // focus ring.
    "data-[hovered]:bg-primary-surface data-[hovered]:text-primary-fg",
    "data-[focus-visible]:bg-primary-surface data-[focus-visible]:text-primary-fg",
    "data-[pressed]:bg-primary-surface data-[pressed]:text-primary-fg",
    "data-[disabled]:bg-transparent data-[disabled]:text-control-fg-disabled",
    // A Fluent flyout item keeps its label colour and takes the subtle fill
    // on hover, focus and press; the selection is an accent pill over the
    // leading padding rather than a check in a gutter, so the label sits at
    // the item's own inset (Fluent's 11px, the section inset here).
    "windows:px-section",
    "windows:data-[hovered]:bg-control-subtle-hover windows:data-[hovered]:text-content-fg",
    "windows:data-[focus-visible]:bg-control-subtle-hover windows:data-[focus-visible]:text-content-fg",
    "windows:data-[pressed]:bg-control-subtle-pressed windows:data-[pressed]:text-control-fg-pressed",
    "windows:data-[selected]:bg-control-subtle-hover windows:data-[selected]:data-[hovered]:bg-control-subtle-pressed",
  ],
});

// Fluent's selection indicator: a 3 by 16 accent pill on the item's leading
// edge. Only the Windows skin shows it; macOS shows the check.
const selectionPillClassName =
  "hidden windows:block absolute top-1/2 left-0 h-4 w-[3px] -translate-y-1/2 rounded-full bg-primary-surface";

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
            <span className="flex w-icon shrink-0 items-center justify-center windows:hidden">
              {isSelected && <Check className="size-icon-small" />}
            </span>
            {isSelected && <span className={selectionPillClassName} />}
            <span className="min-w-0 flex-1 truncate">{children}</span>
          </>
        ) : (
          <span className="min-w-0 flex-1 truncate">{children}</span>
        )
      }
    </AriaListBoxItem>
  );
};
