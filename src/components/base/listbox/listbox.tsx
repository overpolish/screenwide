// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchSlash } from "lucide-react";
import { Ref } from "react";
import {
  ListBox as AriaListBox,
  ListBoxProps as AriaListBoxProps,
} from "react-aria-components";

import { tv } from "../../../lib/variants";

const listBoxVariants = tv({
  base: [
    // Native menus pad the panel and let the items touch, so the highlight of
    // adjacent items forms one continuous run. The list draws no surface of
    // its own: the panel window it lives in supplies the material.
    "flex w-(--trigger-width) flex-col overflow-auto p-list-inset text-content-fg outline-none",
    // A Fluent flyout keeps a 4px gap between its items, so each highlight
    // stands alone rather than forming a run.
    "windows:gap-control",
    "scroll-py-5",
    "data-[empty]:flex data-[empty]:flex-row data-[empty]:items-center data-[empty]:justify-center data-[empty]:gap-control data-[empty]:py-section data-[empty]:text-subheadline data-[empty]:text-content-fg-secondary",
  ],
});

type ListBoxProps<T extends object> = AriaListBoxProps<T> & {
  className?: string;
  ref?: Ref<HTMLDivElement>;
};

export const ListBox = <T extends object>({
  children,
  className,
  ref,
  ...props
}: ListBoxProps<T>) => {
  return (
    <AriaListBox
      ref={ref}
      renderEmptyState={() => (
        <>
          <SearchSlash className="size-icon-small" />
          No items found.
        </>
      )}
      {...props}
      className={listBoxVariants({ className })}
    >
      {children}
    </AriaListBox>
  );
};
