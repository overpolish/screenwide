// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx } from "clsx";
import { ChevronsUpDown } from "lucide-react";
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
// a backdrop blur is not reliable over a transparent material window.
const popoverSurface = "rounded-panel bg-content shadow-md";

const selectVariants = tv({
  defaultVariants: {
    showFocus: true,
  },
  slots: {
    base: "gap-control flex shrink-0 flex-col",
    controls:
      "text-content-fg-secondary [&_svg]:size-icon-mini [&_svg]:shrink-0 [&_svg]:transform-gpu",
    field: [
      "relative isolate inline-flex h-control-height shrink-0 items-stretch rounded-control bg-fill text-content-fg outline-none transition-colors",
      // Bezeled controls do not react to hover; a press stacks the secondary
      // fill behind the label rather than replacing the bezel, as a native
      // pop-up button darkens its bezel.
      "after:pointer-events-none after:absolute after:inset-0 after:-z-10",
      "after:rounded-[inherit] after:transition-colors after:content-['']",
      "has-[button[data-pressed]]:after:bg-fill-secondary",
      "has-[button[data-disabled]]:bg-fill-quaternary has-[button[data-disabled]]:text-content-fg-tertiary",
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
        // The trigger is a button, so the ring is keyboard-only.
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

              {/* Native pop-up buttons show a static up/down chevron pair. */}
              <span aria-hidden className={controls()}>
                <ChevronsUpDown />
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
