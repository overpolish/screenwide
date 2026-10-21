// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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

import { cn } from "../../lib/styling";
import { tv } from "../../lib/variants";
import { fieldVariants } from "../field/field-variants";
import { ListBox } from "../listbox/listbox";
import { ScrollArea } from "../scroll-area/scroll-area";

import { ClearButton } from "./clear-button";

// In-page fallback when the list is not hosted in its own panel window. Opaque:
// a backdrop blur is not reliable over a transparent material window. The
// stroke is Fluent's flyout hairline, transparent on macOS.
const popoverSurface =
  "rounded-panel bg-popover shadow-md inset-ring inset-ring-popover-stroke";

// The shared field in its raised material: a select is a trigger pressed to
// choose, not a field typed into. The trigger button carries the interaction
// state, so the frame around it takes its hover and press, and shows the
// focus ring only for keyboard focus, as a button does.
const selectVariants = tv({
  defaultVariants: {
    material: "raised",
    showFocus: true,
  },
  extend: fieldVariants,
  slots: {
    base: "shrink-0",
    controls:
      "text-content-fg-secondary [&_svg]:size-icon-mini [&_svg]:shrink-0 [&_svg]:transform-gpu",
    field: [
      "inline-flex shrink-0 items-stretch",
      "has-[[data-select-trigger][data-hovered]]:control-raised-hover",
      "has-[[data-select-trigger][data-pressed]]:control-raised-pressed",
      "has-[[data-select-clear]]:[&>[data-select-trigger]]:pe-0",
    ],
    trigger:
      "group relative inline-flex min-w-0 flex-1 shrink-0 cursor-default flex-row items-center justify-between gap-control-inset bg-transparent px-section text-body text-content-fg outline-none",
    value:
      "inline-flex min-w-0 flex-1 flex-row items-center gap-control-inset [&_svg]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu",
  },
  variants: {
    showFocus: {
      true: {
        field: "has-[[data-select-trigger][data-focus-visible]]:ring-3",
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
                  className={cn(
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
              cn(
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
                rootClassName={cn(popoverSurface, listBoxClassName)}
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
