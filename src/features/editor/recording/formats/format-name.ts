// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import {
  frameSizePreset,
  frameSizePresetName,
} from "../../tools/frame/frame-sizes";

import type { ToolPanelFormats } from "../../tools/tool-panel-frame";

type Size = { height: number; width: number };

/** The most either side of a proportion is written as before a shape is
 * named by its size instead: 21:9 reads, 1597:901 does not. */
const MOST_READABLE = 32;

const divisor = (one: number, other: number): number =>
  other === 0 ? one : divisor(other, one % other);

/** A shape's name: its proportions, as 9:16, where they read as such,
 * else its size. */
export const shapeName = ({ height, width }: Size) => {
  const common = divisor(Math.round(width), Math.round(height)) || 1;
  const across = Math.round(width) / common;
  const down = Math.round(height) / common;
  return across <= MOST_READABLE && down <= MOST_READABLE
    ? t("editor-frame-format-ratio", { height: down, width: across })
    : t("editor-frame-format-size", { height, width });
};

/** A format's name: its platform's template, or for the main canvas,
 * made for none, its size, the one thing that sets it apart. */
export const formatName = (format: Size & { preset: string | null }) => {
  const preset = frameSizePreset(format.preset);
  return preset
    ? frameSizePresetName(preset.id)
    : t("editor-frame-format-size", {
        height: format.height,
        width: format.width,
      });
};

/** The size a format exports at: its platform's template size, or for a
 * shape made for none, the shape's canvas. */
export const exportSizeOf = (format: Size & { preset: string | null }) => {
  const preset = frameSizePreset(format.preset);
  return preset
    ? { height: preset.height, width: preset.width }
    : { height: format.height, width: format.width };
};

/** Every format the snapshot lists, shape by shape, with its shape's
 * canvas and its name. */
export const listedFormats = (formats: ToolPanelFormats) =>
  formats.list.flatMap((shape) =>
    shape.formats.map((format) => {
      const sized = { ...format, height: shape.height, width: shape.width };
      return { ...sized, name: formatName(sized), shape: shape.id };
    }),
  );

/** Whether the main canvas is in hand, the one canvas sized freely; it
 * is while no platform is added. */
export const isMainInHand = (formats: ToolPanelFormats | undefined) =>
  !formats?.list.length || formats.list[0]?.id === formats.activeId;
