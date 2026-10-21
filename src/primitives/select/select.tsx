// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronsUpDown } from "lucide-react";
import { type ComponentProps, type ReactNode, type Ref } from "react";
import {
  Select as AriaSelect,
  SelectProps as AriaSelectProps,
  Button,
  Label,
  Popover,
  SelectValue,
} from "react-aria-components";

import { cn } from "../../lib/styling";
import { tv } from "../../lib/variants";
import { fieldVariants } from "../field/field-variants";
import { ListBox } from "../listbox/listbox";

// The shared field in its raised material: a select is a trigger pressed to
// choose, not a field typed into. The trigger button carries the interaction
// state, so the frame around it takes its hover and press, and shows the
// focus ring only for keyboard focus, as a button does. A leading glyph sits
// back as a field's does; the chevrons say a list opens here.
const selectVariants = tv({
  defaultVariants: { material: "raised" },
  extend: fieldVariants,
  slots: {
    base: "shrink-0",
    chevron:
      "shrink-0 text-fg-secondary [&_svg]:size-icon-sm [&_svg]:transform-gpu",
    field: [
      "inline-flex shrink-0 items-stretch",
      "has-[[data-select-trigger][data-hovered]]:control-raised-hover",
      "has-[[data-select-trigger][data-pressed]]:control-raised-pressed",
      "has-[[data-select-trigger][data-focus-visible]]:ring-3",
    ],
    trigger:
      "relative inline-flex min-w-0 flex-1 cursor-default flex-row items-center justify-between gap-inline bg-transparent px-inset text-body text-fg outline-none",
    value: "inline-flex min-w-0 flex-1 flex-row items-center gap-inline",
  },
});

type SelectProps<T extends object> = Omit<AriaSelectProps<T>, "children"> & {
  children?: ReactNode | ((item: T) => ReactNode);
  className?: string;
  items?: Iterable<T>;
  label?: string;
  leftSection?: ReactNode;
  /** Opens the list, which the caller draws in the native popup panel. */
  onPress?: ComponentProps<typeof Button>["onPress"];
  renderValue?: (item: T | null) => ReactNode;
  triggerRef?: Ref<HTMLButtonElement>;
};

// Only the trigger: its list opens in the native popup panel, which the
// caller shows under it from `onPress` (`PopupSelect` does this for a list of
// choices). React Aria builds the chosen value from the options it is given,
// so they are mounted in a list that never opens; nothing is drawn in the
// page. Open is whatever the caller says, the panel's own state, and shut
// otherwise, so a press alone never announces a list that is not there.
export const Select = <T extends object>({
  children,
  className,
  items,
  label,
  leftSection,
  onPress,
  placeholder,
  renderValue,
  triggerRef,
  ...props
}: SelectProps<T>) => {
  const {
    base,
    chevron,
    field,
    label: labelSlot,
    section,
    trigger,
    value,
  } = selectVariants();

  return (
    <AriaSelect
      {...props}
      className={base({ className })}
      isOpen={props.isOpen ?? false}
    >
      {label && <Label className={labelSlot()}>{label}</Label>}

      <div className={field()}>
        <Button
          className={trigger()}
          data-select-trigger
          onPress={onPress}
          ref={triggerRef}
        >
          <span className={value()}>
            {leftSection != null && (
              <span className={section()}>{leftSection}</span>
            )}
            <SelectValue<T>
              className={cn(
                "truncate data-[placeholder]:text-fg-tertiary",
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
          </span>
          <span aria-hidden className={chevron()}>
            <ChevronsUpDown />
          </span>
        </Button>
      </div>

      <Popover isOpen={false}>
        <ListBox items={items}>{children}</ListBox>
      </Popover>
    </AriaSelect>
  );
};
