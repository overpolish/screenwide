// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowUpRight, MapPinPlusInside, Type } from "lucide-react";

/**
 * The glyphs the annotation tools are drawn with.
 *
 * The same tools are picked up in three places - the live overlay's toolbar,
 * the screenshot workspace and the recording workspace - and a tool that
 * looked different depending on where it was taken up would read as a
 * different tool. They are named here so a change reaches all three.
 */
export const ArrowToolIcon = ArrowUpRight;
export const CounterToolIcon = MapPinPlusInside;
export const TextToolIcon = Type;
