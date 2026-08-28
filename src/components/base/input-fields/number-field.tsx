// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { Minus, Plus } from "lucide-react";
import { use, useCallback, useEffect, useRef, useState } from "react";
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
import { FieldGroupContext } from "../field-group/field-group-context";

import { useNumberFieldScrub } from "./use-number-field-scrub";

const numberFieldVariants = tv({
  defaultVariants: {
    size: "default",
  },
  slots: {
    base: "group flex shrink-0 flex-col gap-1",
    field: [
      "relative flex flex-row items-center bg-neutral text-content-fg outline-none transition-colors",
      "hover:bg-neutral-hover active:bg-neutral-pressed",
      "group-data-[disabled]:bg-neutral-subtle group-data-[disabled]:text-neutral-disabled-fg",
      focusStyles,
      "has-[input[data-focus-visible]]:ring-1 has-[input[data-focus-visible]]:ring-offset-1",
    ],
    input: [
      "w-full min-w-0 bg-transparent text-right text-content-fg tabular-nums outline-none selection:bg-transparent focus:selection:bg-accent",
      "placeholder:font-extralight placeholder:italic",
      "group-data-[disabled]:text-neutral-disabled-fg",
    ],
    inputWrapper:
      "flex min-w-0 flex-1 flex-row items-center justify-between gap-2 outline-none",
    label: "font-medium text-muted tabular-nums",
    section: "shrink-0 text-muted",
    stepper: [
      "self-stretch px-2 text-muted transition-colors outline-none",
      "data-[hovered]:text-content-fg",
      focusStyles,
      elementFocusVisible,
    ],
  },
  variants: {
    grouped: {
      true: {
        field: [
          "rounded-none bg-transparent",
          "hover:bg-transparent active:bg-transparent",
        ],
      },
    },
    size: {
      compact: {
        field: "h-6 rounded-lg",
        input: "text-xs placeholder:text-xs",
        inputWrapper: "px-2",
        label: "text-xs",
        section: "text-xs",
        stepper: "[&_svg]:size-icon-compact",
      },
      default: {
        field: "rounded-xl",
        input: "py-2 text-sm placeholder:text-xs",
        inputWrapper: "px-3",
        label: "text-sm",
        section: "text-sm",
        stepper: "[&_svg]:size-icon-default",
      },
    },
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
    size?: "compact" | "default";
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
  size,
  step,
  value,
  ...props
}: NumberFieldProps) => {
  const grouped = use(FieldGroupContext);
  const nativeScrubbing = isTauri();
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
  } = numberFieldVariants({ grouped, size });

  return (
    <AriaNumberField
      {...props}
      className={base({ className })}
      data-control-size={size ?? "default"}
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
        onPointerDown={handlePointerDown}
        ref={groupRef}
      >
        {showSteppers && (
          <Button aria-label="Decrement" className={stepper()} slot="decrement">
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
          <Button aria-label="Increment" className={stepper()} slot="increment">
            <Plus />
          </Button>
        )}
      </Group>
    </AriaNumberField>
  );
};
