// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

pub(crate) fn input(scope: &str, message: impl std::fmt::Display) {
  if std::env::var_os("SCREENWIDE_GLIDE_TRACE").is_some() {
    eprintln!("[glide-{scope}] {message}");
  }
}
