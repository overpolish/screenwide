// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { useNumberFormatter } from "react-aria";
import {
  Group,
  Input,
  NumberField as AriaNumberField,
  NumberFieldProps as AriaNumberFieldProps,
  Label,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { motionDurations } from "../../lib/motion";
import { isMacHost } from "../../lib/platform";
import { tv } from "../../lib/variants";
import { fieldVariants } from "../field/field-variants";
import { focusFieldInput } from "../field/focus-field-input";
import { useSkin } from "../skin/skin";

import { useNumberFieldScrub } from "./use-number-field-scrub";

// The shared field with the value set as figures read: from the end, in
// digits of one width, so a changing value does not shift. It steps by the
// arrow keys, and in the app by dragging on the field; there are no stepper
// buttons.
const numberFieldVariants = tv({
  extend: fieldVariants,
  slots: {
    input: "text-end tabular-nums",
  },
});

export type NumberFieldProps = AriaNumberFieldProps &
  VariantProps<typeof numberFieldVariants> & {
    className?: string;
    label?: string;
    leftSection?: React.ReactNode;
    /** Called for each keystroke that reads as a number, instead of
     * `onChange`, which React Aria fires on blur or Enter. Give this to a field
     * whose value is read from outside the window it is typed in, where a value
     * waiting behind an unfocused field is a value the reader never sees. Plain
     * numeric entry only: a formatted field would need a parse of its own. */
    onTypedChange?: (value: number) => void;
    placeholder?: string;
    /** On the island, the characters the field keeps room for, so a value
     * typed or set within them leaves its width, and its neighbours, where
     * they are. Its figures sit at the end. */
    reservedLength?: number;
    rightSection?: React.ReactNode;
  };

export const NumberField = ({
  className,
  defaultValue,
  label,
  leftSection,
  material,
  maxValue,
  minValue,
  onChange,
  onTypedChange,
  placeholder,
  reservedLength = 1,
  rightSection,
  step,
  value,
  ...props
}: NumberFieldProps) => {
  const skin = useSkin();
  const isIsland = skin === "island";
  // A disabled field neither scrubs nor advertises it with the cursor.
  const nativeScrubbing = isTauri() && !props.isDisabled;
  const nativeSelectionRepaint = nativeScrubbing && isMacHost;
  const [uncontrolledValue, setUncontrolledValue] = useState(defaultValue ?? 0);
  const resolvedValue = value ?? uncontrolledValue;
  const valueRef = useRef(resolvedValue);
  const groupRef = useRef<HTMLDivElement>(null);
  // On the island a field hugs the value it shows, as text does, so it sits
  // as close to its neighbours as the island's other controls: the value as
  // React Aria formats it, or what is being typed until it is committed.
  // Elsewhere it fills the width it is given.
  const formatter = useNumberFormatter(props.formatOptions);
  const [typedLength, setTypedLength] = useState<number | null>(null);
  const shownLength = typedLength ?? formatter.format(resolvedValue).length;
  /** The typed commit as of this render, so the listener below subscribes
   * once rather than on every change of the handler's identity. */
  const commitTypedRef = useRef<(value: number) => void>(() => undefined);
  const typedCommits = onTypedChange !== undefined;

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

  // Each keystroke is the value, so nothing waits behind the field: the owner
  // of a setting read outside this window needs it at once, and a press on
  // another window blurs nothing here.
  useEffect(() => {
    if (!typedCommits) return;
    const input = groupRef.current?.querySelector("input");
    if (!input) return;

    const commitTyped = () => {
      const entry = input.value.trim();
      if (entry === "") return;
      const typed = Number(entry);
      if (Number.isFinite(typed)) commitTypedRef.current(typed);
    };

    input.addEventListener("input", commitTyped);
    return () => {
      input.removeEventListener("input", commitTyped);
    };
  }, [typedCommits]);

  valueRef.current = resolvedValue;

  /** A value the field settles on, held to the step and the bounds. `notify` is
   * what tells the owner, which differs between a typed keystroke and every
   * other way the value moves. */
  const applyValue = useCallback(
    (nextValue: number, notify?: (value: number) => void) => {
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
      notify?.(roundedValue);
    },
    [maxValue, minValue, step, value],
  );

  const changeValue = useCallback(
    (nextValue: number) => {
      applyValue(nextValue, onChange);
    },
    [applyValue, onChange],
  );
  commitTypedRef.current = (nextValue: number) => {
    applyValue(nextValue, onTypedChange);
  };

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
  } = numberFieldVariants({
    material: skin === "island" ? "island" : material,
  });

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
      {label && <Label className={_label()}>{label}</Label>}

      <Group
        className={field({
          className: nativeScrubbing && "cursor-ns-resize",
        })}
        // In the app a press on the field scrubs, and focuses the input when
        // it does not move; elsewhere a press focuses it at once.
        onMouseDown={nativeScrubbing ? undefined : focusFieldInput}
        onPointerDown={nativeScrubbing ? handlePointerDown : undefined}
        ref={groupRef}
      >
        {/* On the island the input hugs its figures, or keeps room for
            them (`reservedLength`), so they are set at the end, beside
            their unit. */}
        <div className={inputWrapper({ className: isIsland && "justify-end" })}>
          {leftSection != null && (
            <span className={section()}>{leftSection}</span>
          )}

          <Input
            className={input({
              className: nativeScrubbing && "cursor-ns-resize",
            })}
            onBlur={(event) => {
              setTypedLength(null);
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
            onInput={
              isIsland
                ? (event) => {
                    setTypedLength(event.currentTarget.value.length);
                  }
                : undefined
            }
            onKeyDown={(event) => {
              if (event.key !== "Enter" && event.key !== "Escape") return;
              event.preventDefault();
              event.stopPropagation();
              event.currentTarget.blur();
            }}
            placeholder={placeholder}
            // A pixel past the figures, which a text input keeps for its
            // caret. A value set from outside, such as a reset, or one
            // committed once typed, eases to its width as the island's black
            // glides round it (`island-section.tsx`), so the controls beside
            // it slide rather than jump; while typing it keeps up at once,
            // so the figures are never cut off.
            style={
              isIsland
                ? {
                    transition:
                      typedLength === null
                        ? `width ${String(motionDurations.travel)}s ease-out`
                        : undefined,
                    width: `calc(${String(Math.max(shownLength, reservedLength))}ch + 1px)`,
                  }
                : undefined
            }
          />

          {rightSection != null && (
            <span className={section()}>{rightSection}</span>
          )}
        </div>
      </Group>
    </AriaNumberField>
  );
};
