// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A random ID made once for this installation and sent to the Worker with
//! each question, so it can share what Openverse allows the app among
//! people at one address, as in an office, rather than counting them as one
//! (`workers/openverse/src/allowance.ts`). It is made for this alone, not
//! the licence's installation ID, so searches cannot be tied to a licence,
//! and is sent nowhere else.

use std::fmt::Write as _;
use std::path::Path;

/// The header the Worker reads the ID from.
pub const INSTALL_HEADER: &str = "X-Screenwide-Install";

/// The file in the app's data folder the ID is kept in.
const FILE: &str = "openverse-install-id";

/// What an ID looks like: 128 random bits in hex.
fn is_id(text: &str) -> bool {
  text.len() == 32
    && text
      .bytes()
      .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// This installation's ID, kept in `data`, made the first time it is
/// asked for; none where it can be neither read nor made, as the Worker
/// then counts the asker with everyone else at its address who sends none.
pub fn install_id(data: &Path) -> Option<String> {
  let path = data.join(FILE);
  if let Ok(kept) = std::fs::read_to_string(&path) {
    if is_id(kept.trim()) {
      return Some(kept.trim().to_owned());
    }
  }
  let mut bytes = [0_u8; 16];
  getrandom::fill(&mut bytes).ok()?;
  let id = bytes
    .iter()
    .fold(String::with_capacity(32), |mut id, byte| {
      let _ = write!(id, "{byte:02x}");
      id
    });
  std::fs::create_dir_all(data).ok()?;
  std::fs::write(&path, &id).ok()?;
  Some(id)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn an_installation_keeps_the_id_made_for_it() {
    let data = std::env::temp_dir().join(format!("screenwide-install-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    let made = install_id(&data).unwrap();
    assert!(is_id(&made));
    assert_eq!(install_id(&data).as_deref(), Some(made.as_str()));
    let _ = std::fs::remove_dir_all(&data);
  }
}
