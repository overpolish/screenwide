// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx } from "clsx";
import { ChevronDown } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { type ComponentProps, type ReactNode, type Ref, use } from "react";
import {
  Select as AriaSelect,
  SelectProps as AriaSelectProps,
  Button,
  Label,
  Popover,
  PopoverProps,
  SelectValue,
} from "react-aria-components";

import { focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";
import { FieldGroupContext } from "../field-group/field-group-context";
import { ListBox } from "../listbox/listbox";
import { OverflowShadow } from "../overflow-shadow/overflow-shadow";

import { ClearButton } from "./components/clear-button";

const selectVariants = tv({
  defaultVariants: {
    showFocus: true,
    size: "default",
  },
  slots: {
    base: "flex shrink-0 flex-col gap-1",
    controls: "text-muted",
    field: [
      "relative inline-flex shrink-0 items-stretch bg-neutral text-content-fg outline-none transition-colors",
      "has-[button[data-hovered]]:bg-neutral-hover has-[button[data-pressed]]:bg-neutral-pressed",
      "has-[button[data-disabled]]:bg-neutral-subtle has-[button[data-disabled]]:text-neutral-disabled-fg",
      "has-[[data-select-clear]]:[&>[data-select-trigger]]:pr-0",
      focusStyles,
    ],
    label: "text-muted font-medium",
    trigger: [
      "group gap-control relative inline-flex min-w-0 flex-1 shrink-0 flex-row items-center justify-between bg-transparent text-content-fg outline-none",
    ],
    value: "inline-flex min-w-0 flex-1 flex-row items-center",
  },
  variants: {
    grouped: {
      true: {
        field: [
          "rounded-none bg-transparent",
          "has-[button[data-hovered]]:bg-transparent has-[button[data-pressed]]:bg-transparent",
        ],
      },
    },
    showFocus: {
      true: {
        field:
          "has-[[data-select-trigger][data-focus-visible]:focus-visible]:ring-1 has-[[data-select-trigger][data-focus-visible]:focus-visible]:ring-offset-1",
      },
    },
    size: {
      compact: {
        controls: "[&_svg]:size-icon-compact",
        field: "h-6 rounded-lg",
        label: "text-xs",
        trigger: "px-2 text-xs",
        value: "gap-2",
      },
      default: {
        controls: "[&_svg]:size-icon-default",
        field: "rounded-xl",
        label: "text-sm",
        trigger: "px-3 py-2 text-sm",
        value: "gap-3",
      },
    },
  },
});

type SelectProps<T extends object> = Omit<AriaSelectProps<T>, "children"> & {
  children?: ReactNode | ((item: T) => ReactNode);
  className?: string;
  clearable?: boolean;
  items?: Iterable<T>;
  label?: string;
  leftSection?: ReactNode;
  listBoxClassName?: string;
  onClear?: () => void;
  onPress?: ComponentProps<typeof Button>["onPress"];
  popoverPlacement?: PopoverProps["placement"];
  popoverShouldFlip?: boolean;
  scrollShadow?: boolean;
  showFocus?: boolean;
  size?: "compact" | "default";
  /**
   * @default false
   * @type boolean
   * @description
   * You'll need to provide and control the visibility of your own
   * listbox
   */
  standalone?: boolean;
  triggerClassName?: string;
  triggerRef?: Ref<HTMLButtonElement>;
};

export const Select = <T extends object>({
  children,
  className,
  clearable = true,
  items,
  label,
  leftSection,
  listBoxClassName,
  onClear,
  onPress,
  placeholder,
  popoverPlacement,
  popoverShouldFlip,
  scrollShadow,
  showFocus,
  size,
  standalone,
  triggerClassName,
  triggerRef,
  ...props
}: SelectProps<T>) => {
  const grouped = use(FieldGroupContext);
  const {
    base,
    controls,
    field,
    label: _label,
    trigger,
    value,
  } = selectVariants({
    grouped,
    showFocus,
    size,
  });
  const listBox = (
    <ListBox
      className={
        scrollShadow
          ? "w-full overflow-visible rounded-none bg-transparent shadow-none"
          : listBoxClassName
      }
      items={items}
      size={size}
    >
      {children}
    </ListBox>
  );

  return (
    <AriaSelect
      {...props}
      className={base({ className })}
      data-control-size={size ?? "default"}
    >
      {({ isOpen }) => (
        <>
          {label && <Label className={_label()}>{label}</Label>}

          <div className={field()}>
            <Button
              className={trigger({ className: triggerClassName })}
              data-select-trigger
              onPress={onPress}
              ref={triggerRef}
            >
              <div className={value()}>
                {leftSection != null && <div>{leftSection}</div>}

                <SelectValue className="data-[placeholder]:text-muted truncate">
                  {({ defaultChildren, isPlaceholder }) =>
                    isPlaceholder ? placeholder : defaultChildren
                  }
                </SelectValue>
              </div>

              <motion.div
                animate={{
                  rotate: isOpen ? 180 : 0,
                  y: isOpen ? -0.5 : 0,
                }}
                aria-hidden="true"
                className={controls()}
                transition={{
                  duration: 0.2,
                }}
              >
                <ChevronDown />
              </motion.div>
            </Button>

            <AnimatePresence>
              {clearable && (
                <ClearButton
                  animate={{ opacity: 1 }}
                  exit={{
                    opacity: 0,
                    scale: 0,
                  }}
                  initial={{ opacity: 0 }}
                  onClear={onClear}
                  size={size}
                />
              )}
            </AnimatePresence>
          </div>

          <Popover
            className={({ placement }) =>
              clsx(
                isOpen ? "animate-in fade-in" : "animate-out fade-out",
                isOpen
                  ? placement === "bottom"
                    ? "slide-in-from-top-5"
                    : "slide-in-from-bottom-5"
                  : placement === "bottom"
                    ? "slide-out-to-top-5"
                    : "slide-out-to-bottom-5",
              )
            }
            // Standalone still needs listbox to be rendered to show a display value
            isOpen={standalone ? false : undefined}
            placement={popoverPlacement}
            shouldFlip={popoverShouldFlip}
          >
            {scrollShadow ? (
              <OverflowShadow
                constrainHeight
                rootClassName={clsx(
                  "rounded-xl bg-content shadow-md",
                  listBoxClassName,
                )}
              >
                {listBox}
              </OverflowShadow>
            ) : (
              listBox
            )}
          </Popover>
        </>
      )}
    </AriaSelect>
  );
};
