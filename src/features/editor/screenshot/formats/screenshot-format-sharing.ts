// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  BACKGROUND,
  placedFrom,
} from "../../recording/formats/format-look-carry";
import { leftBy, shapeHolding } from "../../recording/formats/format-sharing";
import {
  activeFormat,
  formatInHand,
  isMainCanvas,
  mainDocument,
  newShapeId,
} from "../../recording/formats/recording-formats";

import { shownWorkspace } from "./screenshot-format-resolve";
import {
  type InHand,
  removedScreenshotFormat,
  settledFormats,
} from "./screenshot-formats";
import { withScreenshotShownToggled } from "./screenshot-show-in";

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";
import type { ScreenshotShape } from "../../../../bindings/ScreenshotShape";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

// A screenshot's platforms share framing as a recording's do
// (`format-sharing.ts`): platforms of one canvas framed their own way can sit
// in one shape, which a change made with any of them in hand reaches; one
// taken out follows the main canvas again on its own; one still following
// it can take up another's framing; and one framed its own way can be
// given the main canvas's layout back. Each settles what was changed with
// it in hand first (`settledFormats`).

type Workspace = ScreenshotWorkspaceOutputSettings;
type Changed = { document: Workspace; formats: ScreenshotFormats };

/** The workspace the shape in hand shows as `shape`: laid out from the
 * main canvas for what its platforms cover, its layers where it set them
 * itself, and its background where it set that, as `inHand` shows them. */
const shownAs = (
  formats: ScreenshotFormats,
  { blank, workspace }: InHand,
  shape: ScreenshotShape,
) =>
  shownWorkspace(formats, {
    blank,
    main: mainDocument(formats, workspace),
    shape: { ...shape, look: workspace },
  });

/** `shape` as the one in hand keeps it, its workspace the window's. */
const withoutLook = ({ look: _look, ...shape }: ScreenshotShape) => shape;

/**
 * The platform `platform` moved into the shape in hand, from its own shape,
 * which goes where it held no other; or, `isShared` false, taken out of it
 * into a shape of its own that follows the main canvas. The platform in
 * hand stays. Null where nothing changes.
 */
export function withScreenshotPlatformShared(
  formats: ScreenshotFormats,
  inHand: InHand,
  { isShared, platform }: { isShared: boolean; platform: string },
): Changed | null {
  const settled = settledFormats(formats, inHand);
  const active = activeFormat(settled);
  if (isMainCanvas(settled) || platform === formatInHand(active)) return null;
  const platforms = active.platforms ?? [];
  if (isShared === platforms.includes(platform)) return null;
  if (isShared) {
    const from = shapeHolding(settled, platform);
    if (!from) return null;
    const left = (from.platforms ?? []).filter((each) => each !== platform);
    const shape = { ...active, platforms: [...platforms, platform] };
    return {
      document: shownAs(settled, inHand, shape),
      formats: {
        ...settled,
        list: settled.list.flatMap((each) => {
          if (each === active) return [shape];
          if (each !== from) return [each];
          return left.length > 0 ? [leftBy(each, platform, left)] : [];
        }),
      },
    };
  }
  const shape = leftBy(
    active,
    platform,
    platforms.filter((each) => each !== platform),
  );
  const own: ScreenshotShape = {
    id: newShapeId(),
    platform,
    platforms: [platform],
  };
  return {
    document: shownAs(settled, inHand, shape),
    formats: {
      ...settled,
      // Next to the shape it leaves, so the format bar keeps its order.
      list: settled.list.flatMap((each) =>
        each === active ? [shape, own] : [each],
      ),
    },
  };
}

/** The shape in hand, following the main canvas, made to share the framing
 * of the shape `id`: its platforms move there and it goes, and that shape
 * comes into hand with the same platform in hand. Null where nothing
 * changes. */
export function withScreenshotShapeFollowed(
  formats: ScreenshotFormats,
  inHand: InHand,
  id: string,
): Changed | null {
  const settled = settledFormats(formats, inHand);
  const active = activeFormat(settled);
  const target = settled.list.find((shape) => shape.id === id);
  if (isMainCanvas(settled) || !target || target === active) return null;
  const shape: ScreenshotShape = {
    ...target,
    platform: formatInHand(active),
    platforms: [...(target.platforms ?? []), ...(active.platforms ?? [])],
  };
  return {
    document: shownWorkspace(settled, {
      blank: inHand.blank,
      main: mainDocument(settled, inHand.workspace),
      shape,
    }),
    formats: {
      ...settled,
      active: target.id,
      list: settled.list.flatMap((each) => {
        if (each === active) return [];
        return [each === target ? withoutLook(shape) : each];
      }),
    },
  };
}

/** The shape in hand's layers laid out as the main canvas lays them out
 * again, no longer its own; with nothing of its own left to share, each
 * other platform it held follows the main canvas on its own again. Null on
 * the main canvas. */
export function withScreenshotMainPlacement(
  formats: ScreenshotFormats,
  inHand: InHand,
): Changed | null {
  const settled = settledFormats(formats, inHand);
  const active = activeFormat(settled);
  if (isMainCanvas(settled)) return null;
  const { placed: _placed, ...following } = active;
  let changed: Changed = {
    // Its background stays as the window shows it, where it is its own.
    document: shownWorkspace(settled, {
      blank: inHand.blank,
      main: mainDocument(settled, inHand.workspace),
      shape: { ...following, look: inHand.workspace },
    }),
    formats: {
      ...settled,
      list: settled.list.map((each) => (each === active ? following : each)),
    },
  };
  const kept = formatInHand(following);
  for (const platform of following.platforms ?? []) {
    if (platform === kept) continue;
    changed =
      withScreenshotPlatformShared(
        changed.formats,
        { blank: inHand.blank, workspace: changed.document },
        { isShared: false, platform },
      ) ?? changed;
  }
  return changed;
}

/** The shape in hand's background put back to the main canvas's, which it
 * follows again. Null on the main canvas. */
export function withScreenshotMainBackground(
  formats: ScreenshotFormats,
  inHand: InHand,
): Changed | null {
  const settled = settledFormats(formats, inHand);
  if (isMainCanvas(settled)) return null;
  const active = activeFormat(settled);
  const { ownBackground: _own, ...following } = active;
  return {
    document: placedFrom(
      BACKGROUND,
      mainDocument(settled, inHand.workspace),
      inHand.workspace,
    ),
    formats: {
      ...settled,
      list: settled.list.map((each) => (each === active ? following : each)),
    },
  };
}

/** The layer or annotation `syncId` shown in `format`, or left out of it,
 * the shape in hand laid out again for what it shows now. Null where
 * nothing changes. */
export function withScreenshotShownIn(
  formats: ScreenshotFormats,
  inHand: InHand,
  change: { format: string; syncId: string },
): Changed | null {
  const settled = settledFormats(formats, inHand);
  const next = withScreenshotShownToggled(settled, change);
  if (next === settled) return null;
  return {
    document: isMainCanvas(next)
      ? inHand.workspace
      : shownAs(next, inHand, activeFormat(next)),
    formats: next,
  };
}

/** The platform `id` taken away: out of a shape it shares with others,
 * which is kept clear of what those left cover; else with its shape. */
export function removedScreenshotPlatform(
  formats: ScreenshotFormats,
  inHand: InHand,
  id: string,
) {
  const holder = shapeHolding(formats, id);
  const platforms = holder?.platforms ?? [];
  if (!holder || platforms.length < 2)
    return removedScreenshotFormat(formats, inHand, id);
  const settled = settledFormats(formats, inHand);
  const left = platforms.filter((each) => each !== id);
  const isActive = holder.id === settled.active;
  const shape = leftBy(
    settled.list.find((each) => each.id === holder.id) ?? holder,
    id,
    left,
  );
  return {
    document: isActive ? shownAs(settled, inHand, shape) : inHand.workspace,
    formats: {
      ...settled,
      list: settled.list.map((each) => (each.id === holder.id ? shape : each)),
    },
  };
}
