//! Task A3: popup geometry clamping. Pure-function tests — no Tauri runtime.
//! P1-2 + rev-6-1: PopupAnchor unifies units. cursor_logical + work_area are
//! both CSS px (logical); scale_factor converts to physical so the Tauri window
//! API gets the right numbers on Retina. `work_area` is filled from the REAL
//! `Monitor::work_area()` (PhysicalRect<i32, u32>) at the call site.
use linguaray_lib::popup::{
    compute_popup_geometry_logical, LogicalWorkArea, PopupAnchor, PopupMode,
};

const MARGIN: i32 = 8;

fn anchor_at(cx: f64, cy: f64, w: f64, h: f64, sf: f64) -> PopupAnchor {
    PopupAnchor {
        cursor_logical: (cx, cy),
        work_area: LogicalWorkArea { left: 0.0, top: 0.0, right: w, bottom: h },
        scale_factor: sf,
    }
}

#[test]
fn loading_mode_logical_is_200x40() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 1.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Loading, &a);
    assert_eq!((w, h), (200, 40));
}

#[test]
fn single_mode_logical_is_400x300() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 1.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert_eq!((w, h), (400, 300));
}

#[test]
fn multi_mode_logical_is_600x400() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 1.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Multi, &a);
    assert_eq!((w, h), (600, 400));
}

#[test]
fn error_mode_logical_is_400x300() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 1.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Error, &a);
    assert_eq!((w, h), (400, 300));
}

#[test]
fn retina_doubles_physical_size() {
    let a = anchor_at(100.0, 100.0, 1920.0, 1080.0, 2.0);
    let (_, _, w, h) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert_eq!((w, h), (800, 600));
}

#[test]
fn clamps_right_edge_to_work_area_minus_margin() {
    let a = anchor_at(990.0, 100.0, 1000.0, 800.0, 1.0);
    let (x, _, w, _) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert!(x + w as i32 <= 1000 - MARGIN, "x={x} w={w} overflowed right edge");
}

#[test]
fn clamps_bottom_edge_to_work_area_minus_margin() {
    let a = anchor_at(100.0, 790.0, 1000.0, 800.0, 1.0);
    let (_, y, _, h) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert!(y + h as i32 <= 800 - MARGIN, "y={y} h={h} overflowed bottom edge");
}

#[test]
fn clamps_left_edge_to_work_area_plus_margin() {
    let a = anchor_at(0.0, 100.0, 1000.0, 800.0, 1.0);
    let (x, _, _, _) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert!(x >= MARGIN, "x={x} underflowed left edge");
}

#[test]
fn clamps_top_edge_to_work_area_plus_margin() {
    let a = anchor_at(100.0, 0.0, 1000.0, 800.0, 1.0);
    let (_, y, _, _) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert!(y >= MARGIN, "y={y} underflowed top edge");
}

#[test]
fn retina_clamps_in_logical_then_scales_position() {
    // 2x display, cursor at logical (990,100) inside a 1000x800 logical work area.
    // Clamp in logical, then physical position is 2x. Physical width is 800.
    let a = anchor_at(990.0, 100.0, 1000.0, 800.0, 2.0);
    let (x, _, w, _) = compute_popup_geometry_logical(PopupMode::Single, &a);
    assert_eq!(w, 800);
    assert!(x <= 1184, "x={x} overflowed physical right edge after logical clamp");
}
