// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Minus, Plus } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import {
  Button,
  Group,
  Input,
  NumberField as AriaNumberField,
  NumberFieldProps as AriaNumberFieldProps,
  Label,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { tv } from "../../../lib/variants";
import { inputFieldVariants } from "../input-fields/input-field";

import { useNumberFieldScrub } from "./use-number-field-scrub";

const ICON_SIZES = {
  md: 16,
  sm: 14,
};

const numberFieldVariants = tv({
  extend: inputFieldVariants,
  slots: {
    stepper: [
      "text-muted/75 px-3 self-stretch transition-colors",
      "data-[hovered]:text-content-fg",
    ],
  },
});

export type NumberFieldProps = AriaNumberFieldProps &
  VariantProps<typeof numberFieldVariants> & {
    centered?: boolean;
    className?: string;
    fieldClassName?: string;
    label?: string;
    labelClassName?: string;
    leftSection?: React.ReactNode;
    rightAligned?: boolean;
    rightSection?: React.ReactNode;
    scrubStep?: number;
    scrubbable?: boolean;
    showSteppers?: boolean;
    size?: "sm" | "md";
    variant?: "ghost" | "line" | "solid";
  };

export const NumberField = ({
  centered,
  className,
  defaultValue,
  fieldClassName,
  label,
  labelClassName,
  leftSection,
  maxValue,
  minValue,
  onChange,
  rightAligned,
  rightSection,
  scrubStep,
  scrubbable = false,
  showSteppers = true,
  size,
  step,
  value,
  variant,
  ...props
}: NumberFieldProps) => {
  const [uncontrolledValue, setUncontrolledValue] = useState(defaultValue ?? 0);
  const resolvedValue = value ?? uncontrolledValue;
  const valueRef = useRef(resolvedValue);
  const groupRef = useRef<HTMLDivElement>(null);

  valueRef.current = resolvedValue;

  const changeValue = useCallback(
    (nextValue: number) => {
      const clampedValue = Math.min(
        maxValue ?? Number.POSITIVE_INFINITY,
        Math.max(minValue ?? Number.NEGATIVE_INFINITY, nextValue),
      );
      const precision = Math.max(
        0,
        String(scrubStep ?? step ?? 1).split(".")[1]?.length ?? 0,
      );
      const roundedValue = Number(clampedValue.toFixed(precision));

      valueRef.current = roundedValue;
      if (value === undefined) {
        setUncontrolledValue(roundedValue);
      }
      onChange?.(roundedValue);
    },
    [maxValue, minValue, onChange, scrubStep, step, value],
  );

  const handlePointerDown = useNumberFieldScrub({
    changeValue,
    groupRef,
    scrubStep,
    scrubbable,
    step,
    valueRef,
  });

  const {
    base,
    field,
    input,
    inputWrapper,
    label: _label,
    line,
    stepper,
  } = numberFieldVariants({ centered, rightAligned, size, variant });

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
          className: [
            scrubbable ? "cursor-ew-resize" : undefined,
            fieldClassName,
          ],
        })}
        onPointerDown={handlePointerDown}
        ref={groupRef}
      >
        {showSteppers && (
          <Button aria-label="Decrement" className={stepper()} slot="decrement">
            <Minus size={size ? ICON_SIZES[size] : 16} />
          </Button>
        )}

        <div className={inputWrapper()}>
          {leftSection}

          <Input
            className={input()}
            onKeyDown={(event) => {
              if (
                !scrubbable ||
                (event.key !== "Enter" && event.key !== "Escape")
              ) {
                return;
              }
              event.preventDefault();
              event.stopPropagation();
              event.currentTarget.blur();
            }}
          />

          {rightSection}
        </div>

        {showSteppers && (
          <Button aria-label="Increment" className={stepper()} slot="increment">
            <Plus size={size ? ICON_SIZES[size] : 16} />
          </Button>
        )}

        {variant === "line" && <div className={line()} />}
      </Group>
    </AriaNumberField>
  );
};
