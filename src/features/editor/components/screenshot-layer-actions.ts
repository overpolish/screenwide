// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { arranged, Arrangement } from "../annotation-order";
import { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

/** The workspace with one layer moved through the stacking. Every layer is
 * drawn with every other, so each step passes the next one along. */
export const moveScreenshotLayer = ({
  arrangement,
  itemId,
  settings,
}: {
  arrangement: Arrangement;
  itemId: number;
  settings: ScreenshotWorkspaceOutputSettings;
}) => {
  const index = settings.items.findIndex((item) => item.id === itemId);
  if (index === -1) return settings;
  const items = arranged(settings.items, index, {
    arrangement,
    meets: () => true,
  });
  return items === settings.items ? settings : { ...settings, items };
};

export const deleteScreenshotLayer = ({
  itemId,
  settings,
}: {
  itemId: number;
  settings: ScreenshotWorkspaceOutputSettings;
}) => {
  if (settings.items.length <= 1) return null;
  const index = settings.items.findIndex((item) => item.id === itemId);
  if (index === -1) return null;
  const items = settings.items.filter((item) => item.id !== itemId);
  return {
    nextSelectedItemId: items[Math.min(index, items.length - 1)]?.id ?? null,
    settings: { ...settings, items },
  };
};
