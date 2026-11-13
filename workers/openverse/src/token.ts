// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The app's token for Openverse, which lasts 12 hours: asked for with the
// app's registration and kept in the data centre's cache until a little
// before it lapses, so most searches ask for none.

const TOKEN_URL = "https://api.openverse.org/v1/auth_tokens/token/";

/** Where the token is kept in the cache: an address of its own, never one a
 * search could ask for. */
const KEPT = "https://openverse-token.screenwide.internal/";

/** How long before it lapses a token is let go, in s. */
const MARGIN_S = 600;

export type Registration = { clientId: string; clientSecret: string };

/** Asks Openverse for a new token with the app's registration. */
async function newToken(
  registration: Registration,
  fetcher: (input: string, init: RequestInit) => Promise<Response>,
) {
  const answer = await fetcher(TOKEN_URL, {
    body: new URLSearchParams({
      client_id: registration.clientId,
      client_secret: registration.clientSecret,
      grant_type: "client_credentials",
    }),
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    method: "POST",
  });
  if (!answer.ok) throw new Error(`No token: ${String(answer.status)}`);
  const body: unknown = await answer.json();
  const field = (name: string): unknown =>
    body && typeof body === "object" ? Reflect.get(body, name) : null;
  const token = field("access_token");
  const lasts = field("expires_in");
  if (typeof token !== "string" || typeof lasts !== "number")
    throw new Error("No token in Openverse's answer");
  return { lastsS: lasts, token };
}

/** The token kept in `cache`, or a new one asked for and kept there; a new
 * one whatever is kept where `fresh`. */
export async function keptToken(
  cache: Cache,
  registration: Registration,
  fresh: boolean,
  keep: (work: Promise<unknown>) => void,
) {
  if (!fresh) {
    const kept = await cache.match(KEPT);
    if (kept) return kept.text();
  }
  const { lastsS, token } = await newToken(registration, (input, init) =>
    fetch(input, init),
  );
  keep(
    cache.put(
      KEPT,
      new Response(token, {
        headers: {
          "Cache-Control": `max-age=${String(Math.max(lastsS - MARGIN_S, 60))}`,
        },
      }),
    ),
  );
  return token;
}
