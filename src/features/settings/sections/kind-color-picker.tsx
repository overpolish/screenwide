// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pencil } from "lucide-react";
import { useRef, useState } from "react";
import {
  Button,
  DialogTrigger,
  ToggleButtonGroup,
} from "react-aria-components";

import {
  useColorPanel,
  usesSystemColorPanel,
} from "../../../components/base/input-fields/use-color-panel";
import { Popover } from "../../../components/base/popover/popover";
import {
  ANNOTATION_SWATCHES,
  isAnnotationSwatch,
  sameAnnotationColor,
} from "../../../components/shared/annotation-style/palette";
import { BackgroundTile } from "../../../components/shared/background-picker/background-tile";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

/** The colour as a kind keeps it: `#rrggbb` in lower case, whatever the
 * Colours panel reports. */
const kindColor = (color: string) => color.slice(0, 7).toLowerCase();

/** Whether ink over `color` should be the dark of a light appearance. */
const isLight = (color: string) => {
  const channel = (offset: number) =>
    parseInt(color.slice(offset, offset + 2), 16) / 255;
  return 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5) > 0.5;
};

export type KindColorChange = { color: string; customColor?: string };

/**
 * A kind's colour as one swatch in its row, opening the palette, then one
 * square of the kind's own. That square is always there: empty until a
 * colour is chosen in it, and from then on showing the last one chosen,
 * which it keeps while a palette colour is in use. Pressing it takes that
 * colour back and opens the Colours panel on it.
 *
 * The Colours panel belongs to the swatch, not to the palette: opening it
 * closes the palette, and the panel's steps still have somewhere to go.
 */
export function KindColorPicker({
  color,
  customColor,
  isDisabled,
  name,
  onChange,
  onPreview,
}: {
  color: string;
  customColor: string | null;
  /** The kind's name, which the swatch is labelled with. */
  name: string;
  /** A colour to keep. */
  onChange: (change: KindColorChange) => void;
  /** A step of a drag in the Colours panel, shown until the drag settles. */
  onPreview: (change: KindColorChange) => void;
  isDisabled?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const systemPanel = usesSystemColorPanel();
  const colorPanel = useColorPanel({
    onChange: (next) => {
      onPreview({ color: kindColor(next), customColor: kindColor(next) });
    },
    onClose: (settled) => {
      if (settled)
        onChange({
          color: kindColor(settled),
          customColor: kindColor(settled),
        });
    },
  });
  // The input reports a blur however it was left; only one that picked a
  // colour has anything to keep.
  const pickedRef = useRef(false);
  const isCustom = !isAnnotationSwatch(color);
  const takeCustom = () => {
    if (customColor && !isCustom) onChange({ color: customColor });
  };
  const pick = (next: string) => {
    onChange({ color: next });
    setOpen(false);
  };

  return (
    <DialogTrigger isOpen={open} onOpenChange={setOpen}>
      <Button
        aria-label={`${name} colour`}
        className={cn(
          "size-control-height shrink-0 cursor-default rounded-control inset-ring-1 inset-ring-content-fg-quaternary",
          focusStyles,
          elementFocusVisible,
          isDisabled && "opacity-50",
        )}
        isDisabled={isDisabled}
        style={{ backgroundColor: color }}
      />
      {/* Hung from the swatch's leading edge, so the palette opens over its
          own row rather than out past the group. */}
      <Popover aria-label={`${name} colour`} placement="bottom start">
        <ToggleButtonGroup
          aria-label="Colour"
          className="flex gap-control"
          disallowEmptySelection
          selectedKeys={new Set([isCustom ? "custom" : color.toLowerCase()])}
          selectionMode="single"
        >
          {ANNOTATION_SWATCHES.map((swatch) => (
            <BackgroundTile
              ariaLabel={swatch.name}
              background={{ color: swatch.color, kind: "solid" }}
              id={swatch.color}
              isSelected={!isCustom && sameAnnotationColor(swatch.color, color)}
              key={swatch.color}
              onPress={() => {
                pick(swatch.color);
              }}
            />
          ))}
          <span className="relative inline-flex">
            <BackgroundTile
              ariaLabel="Custom colour"
              background={
                customColor ? { color: customColor, kind: "solid" } : undefined
              }
              id="custom"
              isSelected={isCustom}
              onPress={() => {
                takeCustom();
                if (!systemPanel) return;
                setOpen(false);
                colorPanel.open(customColor ?? color);
              }}
            >
              {/* The pen takes the appearance of what it sits on, so it reads
                  on a light colour and a dark one alike; on the empty square
                  it is the window's own. */}
              <span
                className={cn(
                  "flex text-content-fg",
                  customColor &&
                    (isLight(customColor)
                      ? "appearance-light"
                      : "appearance-dark"),
                )}
              >
                <Pencil aria-hidden="true" />
              </span>
            </BackgroundTile>
            {/* Off the Mac there is no panel to open, so the platform's colour
                input sits over the square at zero opacity and settles on
                blur. */}
            {systemPanel ? null : (
              <input
                aria-label="Custom colour"
                className="absolute inset-0 size-full cursor-default opacity-0 outline-none"
                onBlur={(event) => {
                  if (!pickedRef.current) return;
                  pickedRef.current = false;
                  const settled = kindColor(event.currentTarget.value);
                  onChange({ color: settled, customColor: settled });
                }}
                onChange={(event) => {
                  pickedRef.current = true;
                  const next = kindColor(event.currentTarget.value);
                  onPreview({ color: next, customColor: next });
                }}
                onClick={takeCustom}
                type="color"
                value={customColor ?? color}
              />
            )}
          </span>
        </ToggleButtonGroup>
      </Popover>
    </DialogTrigger>
  );
}
