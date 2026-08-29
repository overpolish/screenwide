// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// Shared latest-frame synchronization. Native code retains the corresponding
/// GPU surfaces; this state chooses which source revisions form each monotonic
/// composite frame.
#[derive(Clone, Debug)]
pub struct FrameSynchronizer {
  latest: Vec<Option<i64>>,
  last_output_ns: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionTick {
  pub output_ns: i64,
  pub source_ns: Vec<i64>,
}

impl FrameSynchronizer {
  pub fn new(source_count: usize) -> Result<Self, String> {
    if source_count == 0 {
      return Err("A desktop recording needs at least one capture source".to_owned());
    }
    Ok(Self {
      latest: vec![None; source_count],
      last_output_ns: None,
    })
  }

  pub fn update(
    &mut self,
    source_index: usize,
    timestamp_ns: i64,
  ) -> Result<Option<CompositionTick>, String> {
    let slot = self
      .latest
      .get_mut(source_index)
      .ok_or_else(|| "A frame arrived from an unknown desktop source".to_owned())?;
    if timestamp_ns < 0 {
      return Err("A desktop frame has an invalid timestamp".to_owned());
    }
    if slot.is_some_and(|latest| timestamp_ns <= latest) {
      return Ok(None);
    }
    *slot = Some(timestamp_ns);
    let Some(source_ns) = self.latest.iter().copied().collect::<Option<Vec<_>>>() else {
      return Ok(None);
    };
    let output_ns = source_ns.iter().copied().max().unwrap_or(timestamp_ns);
    if self
      .last_output_ns
      .is_some_and(|previous| output_ns <= previous)
    {
      return Ok(None);
    }
    self.last_output_ns = Some(output_ns);
    Ok(Some(CompositionTick {
      output_ns,
      source_ns,
    }))
  }
}
