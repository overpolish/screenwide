// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_measurements(
  context: *mut c_void,
  output: *mut MeasurementPacket,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let measurements = state.measurements();
    if output.is_null() || capacity == 0 {
      return measurements.len();
    }
    for (index, measurement) in measurements.iter().take(capacity).enumerate() {
      output.add(index).write(measurement.into());
    }
    measurements.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_viewports(
  context: *mut c_void,
  output: *mut ViewportPacket,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let viewports = state.viewports();
    if output.is_null() || capacity == 0 {
      return viewports.len();
    }
    for (index, visual) in viewports.iter().take(capacity).enumerate() {
      output.add(index).write(visual.into());
    }
    viewports.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_probes(
  context: *mut c_void,
  output: *mut ProbePacket,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let probes = state.probes();
    if output.is_null() || capacity == 0 {
      return probes.len();
    }
    for (index, probe) in probes.iter().take(capacity).enumerate() {
      output.add(index).write(probe.into());
    }
    probes.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_guides(
  context: *mut c_void,
  output: *mut GuidePacket,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let guides = state.guides();
    if output.is_null() || capacity == 0 {
      return guides.len();
    }
    for (index, guide) in guides.iter().take(capacity).enumerate() {
      output.add(index).write(guide.into());
    }
    guides.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_guide_gaps(
  context: *mut c_void,
  output: *mut GuideGapPacket,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let gaps = state.guide_gaps();
    if output.is_null() || capacity == 0 {
      return gaps.len();
    }
    for (index, gap) in gaps.iter().take(capacity).enumerate() {
      output.add(index).write(gap.into());
    }
    gaps.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_radii(
  context: *mut c_void,
  output: *mut RadiusPacket,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let radii = state.radii();
    if output.is_null() || capacity == 0 {
      return radii.len();
    }
    for (index, radius) in radii.iter().take(capacity).enumerate() {
      output.add(index).write(radius.into());
    }
    radii.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_centerlines(
  context: *mut c_void,
  output: *mut CenterlinePacket,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let (centerlines, _) = state.center_aids();
    if output.is_null() || capacity == 0 {
      return centerlines.len();
    }
    for (index, line) in centerlines.iter().take(capacity).enumerate() {
      output.add(index).write(line.into());
    }
    centerlines.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_inner_objects(
  context: *mut c_void,
  output: *mut InnerObjectPacket,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let (_, objects) = state.center_aids();
    if output.is_null() || capacity == 0 {
      return objects.len();
    }
    for (index, object) in objects.iter().take(capacity).enumerate() {
      output.add(index).write(object.into());
    }
    objects.len()
  })
  .unwrap_or(0)
}
