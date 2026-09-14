// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx } from "clsx";
import { ChevronDown, ChevronsUpDown } from "lucide-react";
import { AnimatePresence } from "motion/react";
import { type ComponentProps, type ReactNode, type Ref } from "react";
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
import { ListBox } from "../listbox/listbox";
import { ScrollArea } from "../scroll-area/scroll-area";

import { ClearButton } from "./components/clear-button";

// In-page fallback when the list is not hosted in its own panel window. Opaque:
// a backdrop blur is not reliable over a transparent material window. The
// stroke is Fluent's flyout hairline, transparent on macOS.
const popoverSurface =
  "rounded-panel bg-popover shadow-md inset-ring inset-ring-popover-stroke";

const selectVariants = tv({
  defaultVariants: {
    showFocus: true,
  },
  slots: {
    base: "gap-control flex shrink-0 flex-col",
    controls:
      "text-content-fg-secondary [&_svg]:size-icon-mini [&_svg]:shrink-0 [&_svg]:transform-gpu",
    field: [
      "relative inline-flex h-control-height shrink-0 items-stretch rounded-control bg-control-fill text-content-fg outline-none transition-[background-color,box-shadow,color] control-stroke",
      // The bezel's states are the platform tokens, as on a button: a
      // pop-up button on macOS darkens its bezel on press and ignores hover,
      // a Fluent combo box reacts to both. The trigger button carries the
      // interaction state; the field wrapping it and the clear button paints.
      "has-[[data-select-trigger][data-hovered]]:bg-control-fill-hover",
      "has-[[data-select-trigger][data-pressed]]:bg-control-fill-pressed has-[[data-select-trigger][data-pressed]]:text-control-fg-pressed",
      "has-[[data-select-trigger][data-disabled]]:bg-control-fill-disabled has-[[data-select-trigger][data-disabled]]:text-control-fg-disabled",
      "has-[[data-select-clear]]:[&>[data-select-trigger]]:pr-0",
      focusStyles,
    ],
    label: "text-body text-content-fg",
    trigger:
      "group relative inline-flex min-w-0 flex-1 shrink-0 cursor-default flex-row items-center justify-between gap-control-inset bg-transparent px-section text-body text-content-fg outline-none",
    value:
      "inline-flex min-w-0 flex-1 flex-row items-center gap-control-inset [&_svg]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu",
  },
  variants: {
    showFocus: {
      true: {
        // The trigger is a button, so the ring is keyboard-only. The Windows
        // form mirrors `elementFocusVisible` for a wrapper.
        field:
          "has-[[data-select-trigger][data-focus-visible]]:ring-3 windows:has-[[data-select-trigger][data-focus-visible]]:ring-2 windows:has-[[data-select-trigger][data-focus-visible]]:ring-offset-1 windows:has-[[data-select-trigger][data-focus-visible]]:ring-offset-focus-ring-inner",
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
  renderValue?: (item: T | null) => ReactNode;
  scrollShadow?: boolean;
  showFocus?: boolean;
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
  renderValue,
  scrollShadow,
  showFocus,
  standalone,
  triggerClassName,
  triggerRef,
  ...props
}: SelectProps<T>) => {
  const {
    base,
    controls,
    field,
    label: _label,
    trigger,
    value,
  } = selectVariants({ showFocus });
  const listBox = (
    <ListBox
      className={scrollShadow ? "w-full overflow-visible" : listBoxClassName}
      items={items}
    >
      {children}
    </ListBox>
  );

  return (
    <AriaSelect {...props} className={base({ className })}>
      {({ isDisabled, isOpen }) => (
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

                <SelectValue<T>
                  className={clsx(
                    "truncate data-[placeholder]:text-content-fg-secondary",
                    renderValue && "min-w-0 flex-1",
                  )}
                >
                  {({ defaultChildren, isPlaceholder, selectedItems }) =>
                    isPlaceholder
                      ? placeholder
                      : renderValue
                        ? renderValue(selectedItems[0] ?? null)
                        : defaultChildren
                  }
                </SelectValue>
              </div>

              {/* A macOS pop-up button shows a static up/down chevron pair;
                  a Fluent combo box a single down chevron. */}
              <span aria-hidden className={controls()}>
                <ChevronsUpDown className="windows:hidden" />
                <ChevronDown className="hidden windows:block" />
              </span>
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
                  isDisabled={isDisabled}
                  onClear={onClear}
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
              <ScrollArea
                constrainHeight
                rootClassName={clsx(popoverSurface, listBoxClassName)}
              >
                {listBox}
              </ScrollArea>
            ) : (
              <div className={popoverSurface}>{listBox}</div>
            )}
          </Popover>
        </>
      )}
    </AriaSelect>
  );
};
