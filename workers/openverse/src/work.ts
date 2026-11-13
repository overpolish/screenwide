// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// One work, looked up again as the app downloads it: a licence can change,
// or a work be taken down, between being found and being added, so the app
// adds it only where it is still offered under a licence it may use, and
// credits it as it stands now.

import {
  askOpenverse,
  endpointOf,
  type Found,
  foundOf,
  isKind,
  type Kind,
  type Tokens,
  UpstreamError,
} from "./search";

/** Openverse's identifiers for works. */
const ID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

export type Work = { id: string; kind: Kind };

/** The work `url` asks for, or why it is not one. */
export function parseWork(url: URL): Work | string {
  const kind = url.searchParams.get("kind");
  const id = (url.searchParams.get("id") ?? "").toLowerCase();
  if (!isKind(kind)) return "unknown_kind";
  if (!ID.test(id)) return "bad_id";
  return { id, kind };
}

/** The work `work` as it stands now, or none where it is gone or no longer
 * under a licence the app may use. */
export async function lookUp(
  { id, kind }: Work,
  tokens: Tokens,
  fetcher: (input: URL, init: RequestInit) => Promise<Response>,
): Promise<Found | null> {
  const answer = await askOpenverse(
    new URL(`${endpointOf(kind)}${id}/`),
    tokens,
    fetcher,
  );
  if (answer.status === 404) return null;
  if (!answer.ok) throw new UpstreamError(answer.status);
  return foundOf(await answer.json());
}
