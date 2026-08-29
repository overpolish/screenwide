// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { use } from "react";
import {
  ListBoxItem as AriaListBoxItem,
  ListBoxItemProps as AriaListBoxItemProps,
} from "react-aria-components";

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";
import { ListBoxSizeContext } from "../listbox/listbox-context";

const listBoxItemVariants = tv({
  base: [
    "inline-flex shrink-0 cursor-default items-center justify-between gap-2 bg-transparent text-content-fg transition-colors",
    "truncate",
    "data-[hovered]:bg-neutral",
    "data-[pressed]:bg-neutral-hover",
    "data-[selected]:bg-neutral",
    "data-[selected]:data-[hovered]:bg-neutral-hover",
    "data-[selected]:data-[pressed]:bg-neutral-pressed",
    "data-[disabled]:bg-neutral-subtle data-[disabled]:text-neutral-disabled-fg",
    focusStyles,
    elementFocusVisible,
  ],
  defaultVariants: {
    size: "default",
  },
  variants: {
    size: {
      compact: "rounded-lg px-2 py-1 text-xs",
      default: "rounded-xl px-3 py-2 text-sm",
    },
  },
});

type ListBoxItemProps = AriaListBoxItemProps & {
  children?: React.ReactNode;
  className?: string;
};

export const ListBoxItem = ({
  children,
  className,
  ...props
}: ListBoxItemProps) => {
  const size = use(ListBoxSizeContext);

  return (
    <AriaListBoxItem
      {...props}
      className={listBoxItemVariants({ className, size })}
    >
      {({ isSelected }) => (
        <>
          <div className="truncate">{children}</div>
          <AnimatePresence>
            {isSelected && (
              <motion.div
                animate={{ scale: 1 }}
                exit={{ scale: 0 }}
                initial={{ scale: 0 }}
              >
                <Check
                  className="text-content-fg transition-colors"
                  size={size === "compact" ? 14 : 16}
                  strokeWidth={3}
                />
              </motion.div>
            )}
          </AnimatePresence>
        </>
      )}
    </AriaListBoxItem>
  );
};
