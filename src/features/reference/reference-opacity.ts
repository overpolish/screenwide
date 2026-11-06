// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

/** How see-through a reference can be made, in percent: a quarter at a time,
 * so it can be laid over what it shows and compared. */
export const REFERENCE_OPACITIES = [25, 50, 75, 100] as const;

export type ReferenceOpacity = (typeof REFERENCE_OPACITIES)[number];
