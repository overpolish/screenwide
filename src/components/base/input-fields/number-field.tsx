// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { Minus, Plus } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  Button,
  Group,
  Input,
  NumberField as AriaNumberField,
  NumberFieldProps as AriaNumberFieldProps,
  Label,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

import { fieldVariants } from "./input-field";
import { useNumberFieldScrub } from "./use-number-field-scrub";

const numberFieldVariants = tv({
  extend: fieldVariants,
  slots: {
    input: "text-right tabular-nums",
    // Units and glyphs read as part of the value, so they share its size.
    section:
      "shrink-0 text-content-fg-secondary [&_svg]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu",
    stepper: [
      "flex h-full shrink-0 cursor-default items-center px-control text-content-fg-secondary outline-none transition-colors",
      "data-[hovered]:bg-control-subtle-hover data-[hovered]:text-content-fg",
      "data-[pressed]:bg-control-subtle-pressed",
      "data-[disabled]:text-control-fg-disabled",
      "[&_svg]:size-icon-mini [&_svg]:transform-gpu",
      focusStyles,
      elementFocusVisible,
    ],
  },
});

export type NumberFieldProps = AriaNumberFieldProps &
  VariantProps<typeof numberFieldVariants> & {
    className?: string;
    fieldClassName?: string;
    label?: string;
    labelClassName?: string;
    leftSection?: React.ReactNode;
    rightSection?: React.ReactNode;
    showSteppers?: boolean;
  };

export const NumberField = ({
  className,
  defaultValue,
  fieldClassName,
  label,
  labelClassName,
  leftSection,
  maxValue,
  minValue,
  onChange,
  rightSection,
  showSteppers = true,
  step,
  value,
  ...props
}: NumberFieldProps) => {
  // A disabled field neither scrubs nor advertises it with the cursor.
  const nativeScrubbing = isTauri() && !props.isDisabled;
  const nativeSelectionRepaint =
    nativeScrubbing && navigator.userAgent.includes("Mac");
  const [uncontrolledValue, setUncontrolledValue] = useState(defaultValue ?? 0);
  const resolvedValue = value ?? uncontrolledValue;
  const valueRef = useRef(resolvedValue);
  const groupRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const blurOnOutsidePointer = (event: PointerEvent) => {
      const group = groupRef.current;
      const input = group?.querySelector("input");
      if (
        !group ||
        !input ||
        document.activeElement !== input ||
        group.contains(event.target as Node)
      ) {
        return;
      }
      input.blur();
    };

    document.addEventListener("pointerdown", blurOnOutsidePointer, true);
    return () => {
      document.removeEventListener("pointerdown", blurOnOutsidePointer, true);
    };
  }, []);

  valueRef.current = resolvedValue;

  const changeValue = useCallback(
    (nextValue: number) => {
      const clampedValue = Math.min(
        maxValue ?? Number.POSITIVE_INFINITY,
        Math.max(minValue ?? Number.NEGATIVE_INFINITY, nextValue),
      );
      const precision = Math.max(
        0,
        String(step ?? 1).split(".")[1]?.length ?? 0,
      );
      const roundedValue = Number(clampedValue.toFixed(precision));

      valueRef.current = roundedValue;
      if (value === undefined) {
        setUncontrolledValue(roundedValue);
      }
      onChange?.(roundedValue);
    },
    [maxValue, minValue, onChange, step, value],
  );

  const handlePointerDown = useNumberFieldScrub({
    changeValue,
    groupRef,
    step,
    valueRef,
  });

  const {
    base,
    field,
    input,
    inputWrapper,
    label: _label,
    section,
    stepper,
  } = numberFieldVariants();

  return (
    <AriaNumberField
      {...props}
      className={base({ className })}
      maxValue={maxValue}
      minValue={minValue}
      onChange={changeValue}
      step={step}
      value={resolvedValue}
    >
      {label && (
        <Label className={_label({ className: labelClassName })}>{label}</Label>
      )}

      <Group
        className={field({
          className: [nativeScrubbing && "cursor-ns-resize", fieldClassName],
        })}
        onPointerDown={nativeScrubbing ? handlePointerDown : undefined}
        ref={groupRef}
      >
        {showSteppers && (
          <Button
            aria-label="Decrement"
            className={stepper({ className: "rounded-l-[inherit]" })}
            slot="decrement"
          >
            <Minus />
          </Button>
        )}

        <div className={inputWrapper()}>
          {leftSection != null && (
            <span className={section()}>{leftSection}</span>
          )}

          <Input
            className={input({
              className: nativeScrubbing && "cursor-ns-resize",
            })}
            onBlur={(event) => {
              const input = event.currentTarget;
              window.requestAnimationFrame(() => {
                if (document.activeElement === input) return;

                const caret = input.value.length;
                input.setSelectionRange(caret, caret);

                if (nativeSelectionRepaint) {
                  // WKWebView can retain a one-pixel selection bitmap after a
                  // selected input blurs. A synchronous visibility flush
                  // discards it before the browser presents another frame.
                  const previousVisibility = input.style.visibility;
                  input.style.visibility = "hidden";
                  input.getBoundingClientRect();
                  input.style.visibility = previousVisibility;
                }
              });
            }}
            onKeyDown={(event) => {
              if (event.key !== "Enter" && event.key !== "Escape") return;
              event.preventDefault();
              event.stopPropagation();
              event.currentTarget.blur();
            }}
          />

          {rightSection != null && (
            <span className={section()}>{rightSection}</span>
          )}
        </div>

        {showSteppers && (
          <Button
            aria-label="Increment"
            className={stepper({ className: "rounded-r-[inherit]" })}
            slot="increment"
          >
            <Plus />
          </Button>
        )}
      </Group>
    </AriaNumberField>
  );
};
