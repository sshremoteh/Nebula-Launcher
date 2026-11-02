//! The design system's widgets drawn in the game (`crates/horizon-theme`:
//! palette, type scale, radii, control sizes, motion and the bundled fonts;
//! `docs/design-system.md`). Every menu is vanilla's (`ui`); what is left
//! here draws Horizon's own overlays that vanilla has no screen for: the
//! on-screen keyboard a controller types with (`osk`), plus the
//! `horizon.txt` preferences.
//!
//! Build a [`Ui`] over the frame's `HudBatch` with [`Ui::new`], lay out in dp
//! (the launcher's logical pixels) and give every interactive widget a stable
//! `id`. Clicks run the same layout with [`Ui::dry`] and read [`Ui::hit`]: one
//! function draws and hit-tests, so they never disagree.
use crate::hud::{HudAtlas, HudBatch, Rect};
use ab_glyph::{Font as _, FontRef, PxScale, ScaleFont as _};
use std::{cell::RefCell, collections::HashMap, path::Path};

/// A Horizon-only preference from `horizon.txt` beside `options.txt`
/// (`key:value` lines), so vanilla's options file stays exactly as vanilla
/// writes it.
pub fn preference(dir: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("horizon.txt")).ok()?;
    text.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        (k.trim() == key).then(|| v.trim().to_owned())
    })
}
/// Set (or with None remove) one preference, keeping the others.
pub(crate) fn set_preference(dir: &Path, key: &str, value: Option<&str>) {
    let text = std::fs::read_to_string(dir.join("horizon.txt")).unwrap_or_default();
    let mut lines: Vec<String> = text
        .lines()
        .filter(|l| l.split_once(':').is_none_or(|(k, _)| k.trim() != key))
        .map(str::to_owned)
        .collect();
    if let Some(v) = value {
        lines.push(format!("{key}:{v}"));
    }
    let mut out = lines.join("\n");
    out.push('\n');
    let _ = std::fs::write(dir.join("horizon.txt"), out);
}

/// The dark palette (`horizon_theme::DARK`) in the HUD's colour encoding, plus the
/// translucent plates the overlays draw over the game.
pub mod color {
    use horizon_theme::DARK as P;
    pub const fn hex(c: u32, a: u8) -> u32 {
        (a as u32) << 24 | (c & 0xff) << 16 | (c >> 8 & 0xff) << 8 | (c >> 16 & 0xff)
    }
    /// The page, under every veil.
    pub const BG: u32 = hex(P.page, 255);
    /// Hover plates, secondary buttons, fields, the segmented track: the
    /// dark palette's soft plate, as ink over the veil so the plates read the
    /// same over any backdrop (over the page colour they are `plate_soft`).
    pub const RAISED: u32 = hex(P.ink, 20);
    /// Pressed plates and a switch that is off (`plate`, the same way).
    pub const HOVER: u32 = hex(P.ink, 36);
    /// One-pixel rules between rows, slider tracks.
    pub const BORDER: u32 = hex(P.rule, 255);
    /// Text and selection (`ink`).
    pub const TEXT: u32 = hex(P.ink, 255);
    /// Values and secondary text (`ink_soft`).
    pub const MUTED: u32 = hex(P.ink_soft, 255);
    pub const ACCENT: u32 = hex(P.accent, 255);
    pub const ON_ACCENT: u32 = hex(P.on_accent, 255);
    pub(crate) fn with_alpha(c: u32, a: f32) -> u32 {
        let base = (c >> 24) as f32;
        (c & 0x00ff_ffff) | ((base * a.clamp(0.0, 1.0)).round() as u32) << 24
    }
}

/// Material Design 3 (Material You) dark theme palette and tokens for Nebula Launcher.
pub mod md3 {
    use super::color::hex;

    // Cosmic Surface Tonal Hierarchy
    pub const SURFACE: u32 = hex(0x0c0e14, 255);
    pub const SURFACE_CONTAINER_LOW: u32 = hex(0x131722, 255);
    pub const SURFACE_CONTAINER: u32 = hex(0x181e2b, 255);
    pub const SURFACE_CONTAINER_HIGH: u32 = hex(0x202737, 255);
    pub const SURFACE_CONTAINER_HIGHEST: u32 = hex(0x2a3346, 255);

    // Primary Accents (Emerald Mint / Nebula Green)
    pub const PRIMARY: u32 = hex(0x4ade80, 255);
    pub const ON_PRIMARY: u32 = hex(0x052e16, 255);
    pub const PRIMARY_CONTAINER: u32 = hex(0x166534, 255);
    pub const ON_PRIMARY_CONTAINER: u32 = hex(0xbbf7d0, 255);

    // Secondary Accents (Blue Slate)
    pub const SECONDARY: u32 = hex(0x93c5fd, 255);
    pub const SECONDARY_CONTAINER: u32 = hex(0x1e3a8a, 255);
    pub const ON_SECONDARY_CONTAINER: u32 = hex(0xdbeafe, 255);

    // Tertiary Accents (Indigo Violet)
    pub const TERTIARY: u32 = hex(0xc7b8ff, 255);
    pub const TERTIARY_CONTAINER: u32 = hex(0x4338ca, 255);
    pub const ON_TERTIARY_CONTAINER: u32 = hex(0xe0e7ff, 255);

    // Text & Outlines
    pub const ON_SURFACE: u32 = hex(0xf8fafc, 255);
    pub const ON_SURFACE_VARIANT: u32 = hex(0x94a3b8, 255);
    pub const OUTLINE: u32 = hex(0x475569, 255);
    pub const OUTLINE_VARIANT: u32 = hex(0x334155, 180);

    // Status Colors
    pub const SUCCESS: u32 = hex(0x34d399, 255);
    pub const WARNING: u32 = hex(0xfbbf24, 255);
    pub const ERROR: u32 = hex(0xf87171, 255);

    // Shapes in dp
    pub const SHAPE_S: f32 = 8.0;
    pub const SHAPE_M: f32 = 12.0;
    pub const SHAPE_L: f32 = 16.0;
    pub const SHAPE_XL: f32 = 24.0;
    pub const SHAPE_FULL: f32 = 999.0;

    /// Material Motion standard emphasized deceleration curve (0.05, 0.7, 0.1, 1.0)
    #[inline]
    pub fn emphasized_decelerate(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        1.0 - (1.0 - t).powi(3)
    }

    /// Material Motion standard easing curve
    #[inline]
    pub fn standard_easing(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// The launcher's type scale and shapes (`horizon_theme`), in dp.
pub(crate) const SMALL: f32 = horizon_theme::CAPTION.px;
pub(crate) const BODY: f32 = horizon_theme::LABEL.px;
/// The boot screen's wordmark (`DISPLAY`) and console lines (`MICRO`).
pub(crate) const DISPLAY: f32 = horizon_theme::DISPLAY.px;
pub(crate) const MICRO: f32 = horizon_theme::MICRO.px;
/// Fields; plates (hover rows, selection); pills (buttons, segments).
pub(crate) const RADIUS: f32 = horizon_theme::RADIUS;
pub(crate) const RADIUS_PLATE: f32 = horizon_theme::RADIUS_PLATE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    Regular,
    Medium,
    /// The wordmark and page titles (the design system's 500).
    SemiBold,
    /// Counts, timestamps and the boot screen's console lines.
    Mono,
}
struct Fonts {
    faces: [FontRef<'static>; 4],
}
impl Fonts {
    fn load() -> Self {
        let f = |b: &'static [u8]| FontRef::try_from_slice(b).expect("bundled font");
        Self {
            faces: [
                f(horizon_theme::fonts::GEIST_REGULAR),
                f(horizon_theme::fonts::GEIST_MEDIUM),
                f(horizon_theme::fonts::GEIST_SEMIBOLD),
                f(horizon_theme::fonts::GEIST_MONO),
            ],
        }
    }
    fn face(&self, f: Face) -> &FontRef<'static> {
        &self.faces[f as usize]
    }
    /// The ab_glyph scale for an em size in pixels.
    fn scale(&self, f: Face, px: f32) -> PxScale {
        let font = self.face(f);
        let upem = font.units_per_em().unwrap_or(1000.0);
        PxScale::from(px * font.height_unscaled() / upem)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Glyph(Face, u16, u16),
    /// A quarter disc (`false`) or a 1px quarter ring of this radius in pixels.
    Corner(u16, bool),
    AppIcon,
}
#[derive(Clone, Copy)]
struct Cached {
    rect: Option<Rect>,
    /// Offset of the bitmap from the pen (glyphs) in pixels.
    left: f32,
    top: f32,
}
/// Glyphs and corner masks rasterized into the atlas region `horizon/ui-text`.
struct Cache {
    fonts: Fonts,
    entries: HashMap<Key, Cached>,
    shelf: (u32, u32, u32),
    pending: Vec<crate::font::Patch>,
    /// The atlas the entries were uploaded to (a new atlas starts over).
    atlas: usize,
}
thread_local! {
    static CACHE: RefCell<Option<Cache>> = const { RefCell::new(None) };
}
/// Reloaded atlases can occupy the same address with empty run-time slots.
pub fn reset_atlas() {
    CACHE.with(|cache| *cache.borrow_mut() = None);
}
fn with_cache<R>(atlas: &HudAtlas, f: impl FnOnce(&mut Cache) -> R) -> R {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        let id = atlas as *const HudAtlas as usize;
        let cache = c.get_or_insert_with(|| Cache {
            fonts: Fonts::load(),
            entries: HashMap::new(),
            shelf: (0, 0, 0),
            pending: Vec::new(),
            atlas: id,
        });
        if cache.atlas != id {
            cache.atlas = id;
            cache.entries.clear();
            cache.shelf = (0, 0, 0);
            cache.pending.clear();
        }
        f(cache)
    })
}
impl Cache {
    fn region(atlas: &HudAtlas) -> Option<Rect> {
        atlas.sprites.get("horizon/ui-text").copied()
    }
    /// Reserve `w x h` pixels; on overflow forget everything and start over.
    fn place(&mut self, atlas: &HudAtlas, w: u32, h: u32) -> Option<(u32, u32, Rect)> {
        let region = Self::region(atlas)?;
        let (mut x, mut y, mut row) = self.shelf;
        if x + w + 1 > region.w {
            x = 0;
            y += row + 1;
            row = 0;
        }
        if y + h > region.h || w > region.w {
            self.entries.clear();
            self.shelf = (0, 0, 0);
            return None;
        }
        self.shelf = (x + w + 1, y, row.max(h));
        Some((x, y, region))
    }
    fn upload(&mut self, atlas: &HudAtlas, w: u32, h: u32, alpha: &[f32]) -> Option<Rect> {
        let (x, y, region) = self.place(atlas, w, h)?;
        let (ax, ay) = (
            (region.u0 * atlas.width as f32).round() as u32 + x,
            (region.v0 * atlas.height as f32).round() as u32 + y,
        );
        let rgba = alpha
            .iter()
            .flat_map(|a| [255, 255, 255, (a.clamp(0.0, 1.0) * 255.0).round() as u8])
            .collect();
        self.pending.push(crate::font::Patch {
            x: ax,
            y: ay,
            width: w,
            height: h,
            rgba,
        });
        let (aw, ah) = (atlas.width as f32, atlas.height as f32);
        Some(Rect {
            u0: ax as f32 / aw,
            v0: ay as f32 / ah,
            u1: (ax + w) as f32 / aw,
            v1: (ay + h) as f32 / ah,
            w,
            h,
        })
    }
    fn upload_rgba(&mut self, atlas: &HudAtlas, w: u32, h: u32, rgba: Vec<u8>) -> Option<Rect> {
        let (x, y, region) = self.place(atlas, w, h)?;
        let (ax, ay) = (
            (region.u0 * atlas.width as f32).round() as u32 + x,
            (region.v0 * atlas.height as f32).round() as u32 + y,
        );
        self.pending.push(crate::font::Patch {
            x: ax,
            y: ay,
            width: w,
            height: h,
            rgba,
        });
        let (aw, ah) = (atlas.width as f32, atlas.height as f32);
        Some(Rect {
            u0: ax as f32 / aw,
            v0: ay as f32 / ah,
            u1: (ax + w) as f32 / aw,
            v1: (ay + h) as f32 / ah,
            w,
            h,
        })
    }
    fn glyph(&mut self, atlas: &HudAtlas, face: Face, id: ab_glyph::GlyphId, px: f32) -> Cached {
        let size = (px * 4.0).round() as u16;
        let key = Key::Glyph(face, id.0, size);
        if let Some(c) = self.entries.get(&key) {
            return *c;
        }
        let scale = self.fonts.scale(face, f32::from(size) / 4.0);
        let glyph = id.with_scale_and_position(scale, ab_glyph::point(0.0, 0.0));
        let made = match self.fonts.face(face).outline_glyph(glyph) {
            Some(outline) => {
                let b = outline.px_bounds();
                let (w, h) = (b.width() as u32, b.height() as u32);
                let mut alpha = vec![0.0; (w * h) as usize];
                outline.draw(|x, y, c| {
                    if x < w && y < h {
                        alpha[(y * w + x) as usize] = c;
                    }
                });
                // Light text on dark backgrounds reads thin: a gentle boost.
                for a in &mut alpha {
                    *a = a.powf(0.85);
                }
                Cached {
                    rect: self.upload(atlas, w, h, &alpha),
                    left: b.min.x,
                    top: b.min.y,
                }
            }
            None => Cached {
                rect: None,
                left: 0.0,
                top: 0.0,
            },
        };
        if made.rect.is_some() || made.left == 0.0 {
            self.entries.insert(key, made);
        }
        made
    }
    fn corner(&mut self, atlas: &HudAtlas, r: u16, ring: bool) -> Option<Rect> {
        let key = Key::Corner(r, ring);
        if let Some(c) = self.entries.get(&key) {
            return c.rect;
        }
        // The top-left quadrant of a circle centred at (r, r), 4x4 supersampled.
        let n = u32::from(r);
        let rf = f32::from(r);
        let mut alpha = vec![0.0; (n * n) as usize];
        for y in 0..n {
            for x in 0..n {
                let mut hits = 0;
                for sy in 0..4 {
                    for sx in 0..4 {
                        let px = x as f32 + (sx as f32 + 0.5) / 4.0;
                        let py = y as f32 + (sy as f32 + 0.5) / 4.0;
                        let d = ((px - rf).powi(2) + (py - rf).powi(2)).sqrt();
                        if d <= rf && (!ring || d >= rf - 1.0) {
                            hits += 1;
                        }
                    }
                }
                alpha[(y * n + x) as usize] = hits as f32 / 16.0;
            }
        }
        let rect = self.upload(atlas, n, n, &alpha);
        if rect.is_some() {
            self.entries.insert(
                key,
                Cached {
                    rect,
                    left: 0.0,
                    top: 0.0,
                },
            );
        }
        rect
    }
}

/// One frame of design-system drawing, in dp: the
/// launcher's logical pixels, scaled so the screen reads like the launcher's
/// 1040x700 window at any size.
pub struct Ui<'a> {
    pub atlas: &'a HudAtlas,
    pub batch: &'a mut HudBatch,
    /// The screen in dp.
    pub w: f32,
    pub h: f32,
    /// The cursor in dp.
    pub cursor: (f32, f32),
    /// Physical pixels per dp.
    pub density: f32,
    scale: f32,
}
impl<'a> Ui<'a> {
    /// `cursor` is in GUI pixels, like every screen's input.
    pub fn new(atlas: &'a HudAtlas, batch: &'a mut HudBatch, cursor: (f32, f32)) -> Self {
        let scale = batch.scale;
        let (pw, ph) = (batch.width * scale, batch.height * scale);
        let density = (pw / 1040.0).min(ph / 700.0).clamp(1.0, 4.0);
        Self {
            atlas,
            w: pw / density,
            h: ph / density,
            cursor: (cursor.0 * scale / density, cursor.1 * scale / density),
            density,
            scale,
            batch,
        }
    }
    /// Native launcher density on tall phone screens; game HUD scale is untouched.
    pub fn mobile(atlas: &'a HudAtlas, batch: &'a mut HudBatch, cursor: (f32, f32)) -> Self {
        let mut ui = Self::new(atlas, batch, cursor);
        let pw = ui.batch.width * ui.scale;
        let ph = ui.batch.height * ui.scale;
        if ph > pw {
            ui.density = (pw / 480.0).clamp(1.0, 4.0);
            ui.w = pw / ui.density;
            ui.h = ph / ui.density;
            ui.cursor = (
                cursor.0 * ui.scale / ui.density,
                cursor.1 * ui.scale / ui.density,
            );
        } else if ph < 700.0 {
            ui.density = (pw / 1040.0).min(ph / 700.0).clamp(0.5, 4.0);
            ui.w = pw / ui.density;
            ui.h = ph / ui.density;
            ui.cursor = (
                cursor.0 * ui.scale / ui.density,
                cursor.1 * ui.scale / ui.density,
            );
        }
        ui
    }

    pub fn fit_text(&self, face: Face, size: f32, text: &str, width: f32) -> String {
        if self.measure(face, size, text) <= width {
            return text.to_owned();
        }
        let px = (size * self.density * 4.0).round() / 4.0;
        with_cache(self.atlas, |cache| {
            let font = cache.fonts.face(face);
            let scaled = font.as_scaled(cache.fonts.scale(face, px));
            crate::ui_text::fit(text, width * self.density, |previous, ch| {
                let id = font.glyph_id(ch);
                previous.map_or(0.0, |p| scaled.kern(font.glyph_id(p), id)) + scaled.h_advance(id)
            })
        })
    }
    /// Move rasterized glyphs into the batch's atlas uploads (real frames only).
    pub fn finish(self) {
        let pending = with_cache(self.atlas, |c| std::mem::take(&mut c.pending));
        self.batch.patches.extend(pending);
    }

    // ---- geometry, snapped to physical pixels ----
    fn px_x(&self, v: f32) -> f32 {
        (v * self.density).round()
    }
    fn px_y(&self, v: f32) -> f32 {
        (v * self.density).round()
    }
    fn raw_fill(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) {
        if x1 > x0 && y1 > y0 {
            let s = self.scale;
            self.batch.fill(
                self.atlas,
                x0 / s,
                y0 / s,
                (x1 - x0) / s,
                (y1 - y0) / s,
                color,
            );
        }
    }
    fn corner_quad(&mut self, rect: Rect, x: f32, y: f32, r: f32, flip: (bool, bool), color: u32) {
        let (u0, u1) = if flip.0 {
            (rect.u1, rect.u0)
        } else {
            (rect.u0, rect.u1)
        };
        let (v0, v1) = if flip.1 {
            (rect.v1, rect.v0)
        } else {
            (rect.v0, rect.v1)
        };
        let s = self.scale;
        let (gx, gy, gr) = (x / s, y / s, r / s);
        self.batch.quad(
            [[gx, gy], [gx + gr, gy], [gx + gr, gy + gr], [gx, gy + gr]],
            [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
            color,
        );
    }
    fn corners(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, ring: bool, color: u32) {
        if r < 1.0 {
            return;
        }
        let atlas = self.atlas;
        if let Some(rect) = with_cache(atlas, |c| c.corner(atlas, r as u16, ring)) {
            self.corner_quad(rect, x0, y0, r, (false, false), color);
            self.corner_quad(rect, x1 - r, y0, r, (true, false), color);
            self.corner_quad(rect, x1 - r, y1 - r, r, (true, true), color);
            self.corner_quad(rect, x0, y1 - r, r, (false, true), color);
        }
    }
    /// A filled rectangle with a vertical gradient from top to bottom.
    pub fn gradient(&mut self, x: f32, y: f32, w: f32, h: f32, top: u32, bottom: u32) {
        if top >> 24 == 0 && bottom >> 24 == 0 {
            return;
        }
        let (x0, y0, x1, y1) = (
            self.px_x(x),
            self.px_y(y),
            self.px_x(x + w),
            self.px_y(y + h),
        );
        let s = self.scale;
        self.batch.fill_gradient(
            self.atlas,
            (x0 / s, y0 / s, (x1 - x0) / s, (y1 - y0) / s),
            top,
            bottom,
        );
    }
    /// A filled rectangle with rounded corners.
    pub fn rounded(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: u32) {
        if color >> 24 == 0 {
            return;
        }
        let (x0, y0, x1, y1) = (
            self.px_x(x),
            self.px_y(y),
            self.px_x(x + w),
            self.px_y(y + h),
        );
        let r = (radius * self.density)
            .round()
            .min(((x1 - x0) / 2.0).floor())
            .min(((y1 - y0) / 2.0).floor())
            .max(0.0);
        if r < 1.0 {
            self.raw_fill(x0, y0, x1, y1, color);
            return;
        }
        self.raw_fill(x0 + r, y0, x1 - r, y1, color);
        self.raw_fill(x0, y0 + r, x0 + r, y1 - r, color);
        self.raw_fill(x1 - r, y0 + r, x1, y1 - r, color);
        self.corners(x0, y0, x1, y1, r, false, color);
    }
    /// A 1px rounded border.
    pub fn border(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: u32) {
        if color >> 24 == 0 {
            return;
        }
        let (x0, y0, x1, y1) = (
            self.px_x(x),
            self.px_y(y),
            self.px_x(x + w),
            self.px_y(y + h),
        );
        let r = (radius * self.density)
            .round()
            .min(((x1 - x0) / 2.0).floor())
            .min(((y1 - y0) / 2.0).floor())
            .max(0.0);
        self.raw_fill(x0 + r, y0, x1 - r, y0 + 1.0, color);
        self.raw_fill(x0 + r, y1 - 1.0, x1 - r, y1, color);
        self.raw_fill(x0, y0 + r, x0 + 1.0, y1 - r, color);
        self.raw_fill(x1 - 1.0, y0 + r, x1, y1 - r, color);
        self.corners(x0, y0, x1, y1, r, true, color);
    }

    /// Draw a 96x96 or arbitrary RGBA icon cached in the atlas.
    pub fn draw_icon_rgba(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rgba: &'static [u8],
        img_w: u32,
        img_h: u32,
    ) {
        let atlas = self.atlas;
        let rect = with_cache(atlas, |c| {
            if let Some(cached) = c.entries.get(&Key::AppIcon) {
                return cached.rect;
            }
            let r = c.upload_rgba(atlas, img_w, img_h, rgba.to_vec());
            c.entries.insert(Key::AppIcon, Cached { rect: r, left: 0.0, top: 0.0 });
            r
        });
        if let Some(rect) = rect {
            let (x0, y0) = (self.px_x(x), self.px_y(y));
            let (x1, y1) = (self.px_x(x + w), self.px_y(y + h));
            let s = self.scale;
            self.batch.rect_uv(
                x0 / s,
                y0 / s,
                (x1 - x0) / s,
                (y1 - y0) / s,
                rect,
                0xffffffff,
            );
        }
    }

    /// Draw a crisp vector play triangle pointing to the right.
    pub fn play_triangle(&mut self, x: f32, y: f32, size: f32, color: u32) {
        if color >> 24 == 0 {
            return;
        }
        let steps = (size * self.density).round() as i32;
        let x0 = self.px_x(x);
        let y0 = self.px_y(y);
        for i in 0..steps {
            let frac = i as f32 / steps.max(1) as f32;
            let half_h = (1.0 - frac) * (size * self.density * 0.5);
            let top = (size * self.density * 0.5) - half_h;
            let bottom = (size * self.density * 0.5) + half_h;
            self.raw_fill(x0 + i as f32, y0 + top, x0 + i as f32 + 1.0, y0 + bottom, color);
        }
    }

    /// Draw a vector sliders / settings icon.
    pub fn sliders_icon(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        if color >> 24 == 0 {
            return;
        }
        let (x0, y0) = (self.px_x(x), self.px_y(y));
        let (w_px, h_px) = (w * self.density, h * self.density);
        let bar_h = (h_px / 6.0).max(1.0);
        // Line 1
        self.raw_fill(x0, y0, x0 + w_px, y0 + bar_h, color);
        self.raw_fill(x0 + w_px * 0.2, y0 - bar_h, x0 + w_px * 0.45, y0 + bar_h * 2.0, color);
        // Line 2
        let y1 = y0 + (h_px - bar_h) * 0.5;
        self.raw_fill(x0, y1, x0 + w_px, y1 + bar_h, color);
        self.raw_fill(x0 + w_px * 0.55, y1 - bar_h, x0 + w_px * 0.8, y1 + bar_h * 2.0, color);
        // Line 3
        let y2 = y0 + h_px - bar_h;
        self.raw_fill(x0, y2, x0 + w_px, y2 + bar_h, color);
        self.raw_fill(x0 + w_px * 0.35, y2 - bar_h, x0 + w_px * 0.6, y2 + bar_h * 2.0, color);
    }

    /// Draw a Material Design 3 Card with surface container background and outline.
    pub fn md3_card(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, bg: u32, border_color: u32) {
        self.rounded(x, y, w, h, radius, bg);
        if border_color >> 24 != 0 {
            self.border(x, y, w, h, radius, border_color);
        }
    }

    /// Draw a Material Design 3 Badge with pill shape and centered text.
    pub fn md3_badge(&mut self, x: f32, y: f32, text: &str, bg: u32, fg: u32) {
        let text_w = self.measure(Face::Medium, 10.0, text);
        let pad_h = 7.0;
        let badge_w = text_w + 2.0 * pad_h;
        let badge_h = 18.0;
        self.rounded(x, y, badge_w, badge_h, 9.0, bg);
        self.text(Face::Medium, 10.0, x + pad_h, y + 3.0, fg, text);
    }

    /// Draw a Material Touch Ripple expanding within `rect` bounds.
    pub fn md3_ripple(
        &mut self,
        rect: [f32; 4],
        origin: (f32, f32),
        radius: f32,
        alpha: u8,
        color: u32,
    ) {
        if alpha == 0 || radius <= 0.0 {
            return;
        }
        let [rx, ry, rw, rh] = rect;
        let (cx, cy) = origin;
        // Bounding clamp of the circular ripple
        let x0 = (cx - radius).max(rx);
        let y0 = (cy - radius).max(ry);
        let x1 = (cx + radius).min(rx + rw);
        let y1 = (cy + radius).min(ry + rh);
        if x1 > x0 && y1 > y0 {
            let r_w = x1 - x0;
            let r_h = y1 - y0;
            let corner = (r_w.min(r_h) * 0.5).min(radius);
            let ripple_color = (color & 0x00ff_ffff) | ((alpha as u32) << 24);
            self.rounded(x0, y0, r_w, r_h, corner, ripple_color);
        }
    }

    /// Draw a crisp vector checkmark (✓).
    pub fn check_icon(&mut self, x: f32, y: f32, size: f32, color: u32) {
        if color >> 24 == 0 {
            return;
        }
        let (x0, y0) = (self.px_x(x), self.px_y(y));
        let s = size * self.density;
        let thick = (s * 0.16).max(1.0);
        // Short stroke (down-right)
        for i in 0..(s * 0.35) as i32 {
            let fi = i as f32;
            self.raw_fill(x0 + fi, y0 + s * 0.5 + fi, x0 + fi + thick, y0 + s * 0.5 + fi + thick, color);
        }
        // Long stroke (up-right)
        let mid_x = x0 + s * 0.35;
        let mid_y = y0 + s * 0.85;
        for i in 0..(s * 0.65) as i32 {
            let fi = i as f32;
            self.raw_fill(mid_x + fi, mid_y - fi * 1.2, mid_x + fi + thick, mid_y - fi * 1.2 + thick, color);
        }
    }

    /// Draw a vector gear / settings icon.
    pub fn gear_icon(&mut self, x: f32, y: f32, size: f32, color: u32) {
        if color >> 24 == 0 {
            return;
        }
        let (cx, cy) = (self.px_x(x + size * 0.5), self.px_y(y + size * 0.5));
        let r = size * self.density * 0.5;
        let teeth = 6;
        for i in 0..teeth {
            let angle = (i as f32 / teeth as f32) * std::f32::consts::PI;
            let dx = angle.cos() * r;
            let dy = angle.sin() * r;
            let w = (r * 0.3).max(1.0);
            self.raw_fill(cx - dx - w * 0.5, cy - dy - w * 0.5, cx + dx + w * 0.5, cy + dy + w * 0.5, color);
        }
    }

    /// Draw a vector folder icon.
    pub fn folder_icon(&mut self, x: f32, y: f32, size: f32, color: u32) {
        if color >> 24 == 0 {
            return;
        }
        let (x0, y0) = (self.px_x(x), self.px_y(y));
        let (w, h) = (size * self.density, size * self.density * 0.8);
        // Tab
        self.raw_fill(x0, y0, x0 + w * 0.45, y0 + h * 0.3, color);
        // Main body
        self.raw_fill(x0, y0 + h * 0.25, x0 + w, y0 + h, color);
    }

    /// Draw a vector vibration / haptic waves icon.
    pub fn vibrate_icon(&mut self, x: f32, y: f32, size: f32, color: u32) {
        if color >> 24 == 0 {
            return;
        }
        let (x0, y0) = (self.px_x(x), self.px_y(y));
        let (w, h) = (size * self.density, size * self.density);
        let phone_w = w * 0.45;
        let px = x0 + (w - phone_w) * 0.5;
        // Central phone device
        self.raw_fill(px, y0, px + phone_w, y0 + h, color);
        // Screen hollow
        let hollow = color::hex(0x0c0e14, 255);
        self.raw_fill(px + 2.0, y0 + 3.0, px + phone_w - 2.0, y0 + h - 3.0, hollow);
        // Left waves
        self.raw_fill(x0, y0 + h * 0.25, x0 + 2.0, y0 + h * 0.75, color);
        self.raw_fill(x0 + 3.0, y0 + h * 0.15, x0 + 5.0, y0 + h * 0.85, color);
        // Right waves
        self.raw_fill(x0 + w - 5.0, y0 + h * 0.15, x0 + w - 3.0, y0 + h * 0.85, color);
        self.raw_fill(x0 + w - 2.0, y0 + h * 0.25, x0 + w, y0 + h * 0.75, color);
    }

    // ---- text ----
    /// The advance of `text` in dp.
    pub fn measure(&self, face: Face, size: f32, text: &str) -> f32 {
        let px = (size * self.density * 4.0).round() / 4.0;
        with_cache(self.atlas, |c| {
            let font = c.fonts.face(face);
            let scaled = font.as_scaled(c.fonts.scale(face, px));
            let mut w = 0.0;
            let mut prev = None;
            for ch in text.chars() {
                let id = font.glyph_id(ch);
                if let Some(p) = prev {
                    w += scaled.kern(p, id);
                }
                w += scaled.h_advance(id);
                prev = Some(id);
            }
            w / self.density
        })
    }
    /// Draw a line of text in a `line`-tall box whose top is `y` (the
    /// launcher's line heights: about 1.45 × size for the interface);
    /// returns its width in dp.
    pub(crate) fn text_line(
        &mut self,
        face: Face,
        size: f32,
        line: f32,
        x: f32,
        y: f32,
        color: u32,
        text: &str,
    ) -> f32 {
        let px = (size * self.density * 4.0).round() / 4.0;
        let real = color >> 24 != 0;
        let (x0, y0) = (self.px_x(x), self.px_y(y));
        let box_px = line * self.density;
        let atlas = self.atlas;
        let density = self.density;
        let mut quads = Vec::new();
        let width = with_cache(atlas, |c| {
            let scale = c.fonts.scale(face, px);
            let (ascent, descent) = {
                let f = c.fonts.face(face).as_scaled(scale);
                (f.ascent(), f.descent())
            };
            let baseline = y0 + ((box_px - (ascent - descent)) / 2.0 + ascent).round();
            let mut pen = x0;
            let mut prev = None;
            for ch in text.chars() {
                let (id, advance, kern) = {
                    let font = c.fonts.face(face);
                    let id = font.glyph_id(ch);
                    let f = font.as_scaled(scale);
                    (id, f.h_advance(id), prev.map_or(0.0, |p| f.kern(p, id)))
                };
                pen += kern;
                if real && !ch.is_whitespace() {
                    let g = c.glyph(atlas, face, id, px);
                    if let Some(rect) = g.rect {
                        quads.push(((pen + g.left).round(), baseline + g.top, rect));
                    }
                }
                pen += advance;
                prev = Some(id);
            }
            (pen - x0) / density
        });
        let s = self.scale;
        for (gx, gy, rect) in quads {
            self.batch.rect_uv(
                gx / s,
                gy / s,
                rect.w as f32 / s,
                rect.h as f32 / s,
                rect,
                color,
            );
        }
        width
    }
    /// Text in a `1.45 × size` line box whose top is `y`.
    pub fn text(&mut self, face: Face, size: f32, x: f32, y: f32, color: u32, text: &str) -> f32 {
        self.text_line(face, size, (size * 1.45).round(), x, y, color, text)
    }
    pub(crate) fn text_centered(
        &mut self,
        face: Face,
        size: f32,
        cx: f32,
        y: f32,
        color: u32,
        text: &str,
    ) {
        let w = self.measure(face, size, text);
        self.text(face, size, cx - w / 2.0, y, color, text);
    }

    // ---- the launcher's components ----
}

// ---- the boot screen ----

/// The boot screen's palette on the pure black: a white wordmark and dimmed
/// console lines (the theme's ink is for light pages).
const BOOT_TEXT: u32 = color::hex(0xFFFFFF, 255);
const BOOT_LOG: u32 = color::hex(0x9BA1A6, 255);

/// The boot screen while the menu is not ready yet: the wordmark centred on
/// pure black and the console's last lines at the bottom right, as the
/// terminal prints them.
pub fn draw_boot(atlas: &HudAtlas, batch: &mut HudBatch, logs: &[String]) {
    // Pure black under everything: the frame's panorama (the title artwork
    // once the bundle is in) must not leak through.
    batch.fill(
        atlas,
        0.0,
        0.0,
        batch.width,
        batch.height,
        color::hex(0x000000, 255),
    );
    wordmark(atlas, batch);
    draw_boot_log(atlas, batch, logs);
}

/// The wordmark with its tagline, centred.
fn wordmark(atlas: &HudAtlas, batch: &mut HudBatch) {
    let mut ui = Ui::new(atlas, batch, (0.0, 0.0));
    let size = (ui.w / 16.0).clamp(DISPLAY, 96.0);
    let small = (size * 0.25).clamp(MICRO, 18.0);
    let block = size * 1.45 + small * 0.6 + small * 1.5;
    let top = (ui.h - block) / 2.0;
    ui.text_centered(Face::SemiBold, size, ui.w / 2.0, top, BOOT_TEXT, "Horizon");
    ui.text_centered(
        Face::Mono,
        small,
        ui.w / 2.0,
        top + size * 1.45 + small * 0.6,
        BOOT_LOG,
        "Rust based minecraft client",
    );
    ui.finish();
}

/// The console's last lines at the bottom left, over any loading screen: long
/// lines wrap word-wise, the block grows upwards.
pub fn draw_boot_log(atlas: &HudAtlas, batch: &mut HudBatch, logs: &[String]) {
    if logs.is_empty() {
        return;
    }
    let mut ui = Ui::new(atlas, batch, (0.0, 0.0));
    let (size, line, pad) = (MICRO, MICRO * 1.5, 16.0);
    let max_w = ui.w / 2.0;
    let max_rows = (((ui.h - pad) / line) as usize).clamp(1, 24);
    let mut y = ui.h - pad - line;
    let mut drawn = 0;
    'lines: for text in logs.iter().rev() {
        let chunks = wrapping_lines(&ui, Face::Mono, size, text, max_w);
        for chunk in chunks.iter().rev() {
            if drawn == max_rows {
                break 'lines;
            }
            ui.text(Face::Mono, size, pad, y, BOOT_LOG, chunk);
            y -= line;
            drawn += 1;
        }
    }
    ui.finish();
}

/// The lines `text` wraps into at `max_w`, word-wise; a word wider than
/// `max_w` on its own breaks at the edge.
fn wrapping_lines(ui: &Ui, face: Face, size: f32, text: &str, max_w: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let mut word = word;
        loop {
            let candidate = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if ui.measure(face, size, &candidate) <= max_w {
                line = candidate;
                break;
            }
            if line.is_empty() {
                // The word alone does not fit: hard-break it at the edge.
                let mut end = 0;
                for (i, ch) in word.char_indices() {
                    if i > 0 && ui.measure(face, size, &word[..i]) > max_w {
                        break;
                    }
                    end = i + ch.len_utf8();
                }
                lines.push(word[..end].to_owned());
                word = &word[end..];
            } else {
                lines.push(std::mem::take(&mut line));
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_reload_regenerates_text_and_corners_at_the_same_address() {
        let atlas = HudAtlas::build(Path::new("/nonexistent/horizon-test-assets")).unwrap();
        reset_atlas();
        let draw = |cache: &mut Cache| {
            let glyph = cache.fonts.face(Face::Regular).glyph_id('A');
            assert!(
                cache
                    .glyph(&atlas, Face::Regular, glyph, 16.0)
                    .rect
                    .is_some()
            );
            assert!(cache.corner(&atlas, 8, false).is_some());
            std::mem::take(&mut cache.pending)
        };
        let before = with_cache(&atlas, draw);
        assert_eq!(before.len(), 2);
        assert!(with_cache(&atlas, draw).is_empty());
        reset_atlas();
        let after = with_cache(&atlas, draw);
        assert_eq!(after.len(), 2);
        for (a, b) in before.iter().zip(&after) {
            assert_eq!((a.x, a.y, a.width, a.height), (b.x, b.y, b.width, b.height));
            assert_eq!(a.rgba, b.rgba);
        }
        reset_atlas();
    }

    #[test]
    fn the_boot_atlas_holds_the_run_time_glyph_region_without_a_pack() {
        let atlas = HudAtlas::boot();
        assert_eq!((atlas.width, atlas.height), (1024, 1024));
        assert_eq!(
            atlas.sprites["horizon/ui-text"].texels((atlas.width, atlas.height)),
            (1024, 512)
        );
        assert_eq!(
            atlas.sprites["horizon/white"].texels((atlas.width, atlas.height)),
            (4, 4)
        );
    }

    #[test]
    fn the_boot_screen_centres_the_wordmark_and_left_aligns_the_console() {
        reset_atlas();
        let atlas = HudAtlas::boot();
        let mut batch = HudBatch::new([1280, 720]);
        draw_boot(&atlas, &mut batch, &[]);
        // The first quad is the pure black backdrop over the frame's panorama.
        let (w, h) = (batch.width, batch.height);
        let bg = &batch.vertices[..6];
        assert!(bg.iter().all(|v| v.color == 0xFF00_0000));
        assert!(bg.iter().all(|v| v.pos[0] == 0.0 || v.pos[0] == w));
        assert!(bg.iter().all(|v| v.pos[1] == 0.0 || v.pos[1] == h));
        assert!(
            !batch.patches.is_empty(),
            "the glyphs are queued for the atlas"
        );

        let mut batch = HudBatch::new([1280, 720]);
        wordmark(&atlas, &mut batch);
        let (min, max) = batch
            .vertices
            .iter()
            .map(|v| v.pos[0])
            .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        assert!(
            min < batch.width / 2.0 && max > batch.width / 2.0,
            "the wordmark straddles the centre"
        );
        assert!(((min + max) / 2.0 - batch.width / 2.0).abs() < 1.0);

        let long = "[12:34:56] [Render thread/INFO]: Horizon: resources loaded in 1.0s; 1240 block states, 1500 items, 62 biomes; 8 worker threads".to_owned();
        let mut batch = HudBatch::new([1280, 720]);
        draw_boot_log(&atlas, &mut batch, &[long]);
        let (min, max) = batch
            .vertices
            .iter()
            .map(|v| v.pos[0])
            .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        assert!(min >= 4.0, "the line stays inside the left inset");
        assert!(
            max - min <= batch.width / 2.0 + 1.0,
            "a too wide line is cut to half the screen"
        );

        let mut batch = HudBatch::new([1280, 720]);
        let ui = Ui::new(&atlas, &mut batch, (0.0, 0.0));
        let lines = wrapping_lines(
            &ui,
            Face::Mono,
            MICRO,
            "the quick brown fox jumps over the lazy dog",
            90.0,
        );
        assert!(lines.len() > 1, "a long line wraps");
        assert!(
            lines
                .iter()
                .all(|l| ui.measure(Face::Mono, MICRO, l) <= 90.0),
            "every wrapped line fits"
        );
        assert_eq!(
            lines
                .iter()
                .map(|l| l.split_whitespace().count())
                .sum::<usize>(),
            9,
            "the wrap keeps every word"
        );

        let mut batch = HudBatch::new([1280, 720]);
        draw_boot_log(&atlas, &mut batch, &[]);
        assert!(batch.vertices.is_empty() && batch.patches.is_empty());
        reset_atlas();
    }
}
