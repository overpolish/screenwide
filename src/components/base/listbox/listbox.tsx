// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchSlash } from "lucide-react";
import { Ref } from "react";
import {
  ListBox as AriaListBox,
  ListBoxProps as AriaListBoxProps,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { tv } from "../../../lib/variants";

import { type ListBoxSize, ListBoxSizeContext } from "./listbox-context";

const listBoxVariants = tv({
  base: [
    "gap-control p-control flex w-(--trigger-width) flex-col overflow-auto rounded-xl text-content-fg outline-none",
    "scroll-py-5",
    "data-[empty]:py-section data-[empty]:text-xs data-[empty]:text-muted data-[empty]:flex data-[empty]:flex-row data-[empty]:items-center data-[empty]:justify-center",
  ],
  defaultVariants: {
    variant: "filled",
  },
  variants: {
    variant: {
      filled: "bg-content shadow-md",
      transparent: "bg-transparent shadow-none",
    },
  },
});

type ListBoxProps<T extends object> = AriaListBoxProps<T> &
  VariantProps<typeof listBoxVariants> & {
    className?: string;
    ref?: Ref<HTMLDivElement>;
    size?: ListBoxSize;
  };

export const ListBox = <T extends object>({
  children,
  className,
  ref,
  size = "default",
  variant,
  ...props
}: ListBoxProps<T>) => {
  return (
    <ListBoxSizeContext value={size}>
      <AriaListBox
        ref={ref}
        renderEmptyState={() => (
          <>
            <SearchSlash className="size-icon-compact" />
            No items found.
          </>
        )}
        {...props}
        className={listBoxVariants({ className, variant })}
      >
        {children}
      </AriaListBox>
    </ListBoxSizeContext>
  );
};
