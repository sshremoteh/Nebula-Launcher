//! Cosmic background visuals for Nebula Launcher:
//! Procedural twinkling stars, mythical constellations with shimmering starlight lines,
//! and gentle falling snow / cosmic stardust drifting through the nebula.

use horizon_client::{
    hud::rgba,
    ui_theme::Ui,
};

pub struct NebulaSky;

impl NebulaSky {
    /// Render the entire cosmic space background, nebulas, constellations, stars, and snow.
    pub fn render(ui: &mut Ui, time_sec: f32) {
        let (w, h) = (ui.w, ui.h);
        let scale = ui.density / ui.batch.scale;
        let atlas = ui.atlas;

        // 1. Deep space cosmic gradient
        ui.batch.fill_gradient(
            atlas,
            (0.0, 0.0, w * scale, h * scale),
            rgba(6, 9, 22, 255),  // Deep nebula dark blue/black
            rgba(3, 4, 10, 255),  // Abyss void
        );

        // 2. Nebula atmospheric glow clouds
        // Top-right violet/purple nebula cloud
        let tr_w = w * 0.65;
        let tr_h = h * 0.55;
        let tr_x = w - tr_w;
        let p_pulse = (time_sec * 0.7).sin() * 0.5 + 0.5;
        let purple_alpha = (26.0 + 14.0 * p_pulse) as u8;
        ui.gradient(
            tr_x,
            0.0,
            tr_w,
            tr_h,
            rgba(139, 92, 246, purple_alpha),
            rgba(139, 92, 246, 0),
        );

        // Bottom-left cyan/electric blue nebula cloud
        let bl_w = w * 0.6;
        let bl_h = h * 0.55;
        let bl_y = h - bl_h;
        let c_pulse = (time_sec * 0.9 + 1.5).sin() * 0.5 + 0.5;
        let cyan_alpha = (20.0 + 12.0 * c_pulse) as u8;
        ui.gradient(
            0.0,
            bl_y,
            bl_w,
            bl_h,
            rgba(6, 182, 212, 0),
            rgba(6, 182, 212, cyan_alpha),
        );

        // Center subtle indigo aura
        let center_pulse = (time_sec * 0.5 + 3.0).sin() * 0.5 + 0.5;
        let indigo_alpha = (16.0 + 10.0 * center_pulse) as u8;
        ui.gradient(
            w * 0.2,
            h * 0.2,
            w * 0.6,
            h * 0.6,
            rgba(99, 102, 241, indigo_alpha),
            rgba(99, 102, 241, 0),
        );

        // 3. Constellations with glowing stars & shimmering lines
        Self::render_constellations(ui, time_sec, w, h);

        // 4. Procedural twinkling star field
        Self::render_stars(ui, time_sec, w, h);

        // 5. Falling cosmic snow & stardust
        Self::render_snow(ui, time_sec, w, h);
    }

    /// Render mythical constellations with thin starlight lines and glowing star nodes.
    fn render_constellations(ui: &mut Ui, time_sec: f32, w: f32, h: f32) {
        // Shimmer factor for constellation line glow
        let shimmer = ((time_sec * 1.2).sin() * 0.5 + 0.5).powf(1.5);
        let line_alpha = (45.0 + 35.0 * shimmer) as u8;
        let line_color = rgba(165, 180, 252, line_alpha);
        let star_halo_alpha = (60.0 + 40.0 * shimmer) as u8;

        // Constellation 1: Major Dipper / Ursa-like pattern (scaled normalized coordinates)
        let c1_nodes = [
            [0.10, 0.15],
            [0.18, 0.19],
            [0.26, 0.26],
            [0.34, 0.32],
            [0.44, 0.30],
            [0.48, 0.42],
            [0.38, 0.44],
        ];
        let c1_edges = [
            (0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 3),
        ];

        // Constellation 2: Cassiopeia 'W'-like pattern in the top-right
        let c2_nodes = [
            [0.68, 0.14],
            [0.74, 0.22],
            [0.80, 0.17],
            [0.87, 0.25],
            [0.93, 0.16],
        ];
        let c2_edges = [(0, 1), (1, 2), (2, 3), (3, 4)];

        // Constellation 3: Cygnus / Northern Cross in lower-right
        let c3_nodes = [
            [0.75, 0.65],
            [0.82, 0.73],
            [0.89, 0.82],
            [0.74, 0.77],
            [0.88, 0.70],
        ];
        let c3_edges = [(0, 1), (1, 2), (3, 1), (1, 4)];

        let draw_constellation = |ui: &mut Ui, nodes: &[[f32; 2]], edges: &[(usize, usize)]| {
            for &(i1, i2) in edges {
                let p1 = [nodes[i1][0] * w, nodes[i1][1] * h];
                let p2 = [nodes[i2][0] * w, nodes[i2][1] * h];
                draw_line(ui, p1, p2, 1.2, line_color);
            }
            for (idx, node) in nodes.iter().enumerate() {
                let cx = node[0] * w;
                let cy = node[1] * h;
                let phase = idx as f32 * 1.3 + time_sec * 2.0;
                let node_pulse = (phase.sin() * 0.5 + 0.5).powi(2);
                let core_size = 3.0 + 1.2 * node_pulse;
                let halo_size = 8.0 + 4.0 * node_pulse;

                // Outer halo
                ui.rounded(
                    cx - halo_size * 0.5,
                    cy - halo_size * 0.5,
                    halo_size,
                    halo_size,
                    halo_size * 0.5,
                    rgba(147, 197, 253, (star_halo_alpha as f32 * 0.4) as u8),
                );
                // Diamond star core
                draw_sparkle(ui, cx, cy, core_size, rgba(240, 248, 255, 230));
            }
        };

        draw_constellation(ui, &c1_nodes, &c1_edges);
        draw_constellation(ui, &c2_nodes, &c2_edges);
        draw_constellation(ui, &c3_nodes, &c3_edges);
    }

    /// Render 64 procedural twinkling stars scattered across the space background.
    fn render_stars(ui: &mut Ui, time_sec: f32, w: f32, h: f32) {
        // Pre-calculated deterministic pseudo-random star field
        // (x_frac, y_frac, size, freq, phase, base_color_tint)
        const STAR_COUNT: usize = 60;
        for i in 0..STAR_COUNT {
            let seed = (i * 2654435761) as u32;
            let x_frac = ((seed & 0xFFFF) as f32) / 65535.0;
            let y_frac = (((seed >> 16) & 0xFFFF) as f32) / 65535.0;
            let size = 1.6 + (((seed % 17) as f32) / 17.0) * 2.0; // 1.6 - 3.6 dp
            let freq = 0.8 + (((seed % 23) as f32) / 23.0) * 2.2;
            let phase = (((seed % 37) as f32) / 37.0) * std::f32::consts::TAU;

            let x = x_frac * w;
            let y = y_frac * h;

            let brightness = ((time_sec * freq + phase).sin() * 0.5 + 0.5).powi(2);
            let alpha = (40.0 + 205.0 * brightness) as u8;

            let is_blue_tint = (i % 3) == 0;
            let is_gold_tint = (i % 7) == 0;
            let color = if is_blue_tint {
                rgba(191, 219, 254, alpha)
            } else if is_gold_tint {
                rgba(254, 240, 138, alpha)
            } else {
                rgba(241, 245, 249, alpha)
            };

            if size > 2.8 && brightness > 0.65 {
                draw_sparkle(ui, x, y, size * (0.8 + 0.4 * brightness), color);
            } else {
                ui.rounded(
                    x - size * 0.5,
                    y - size * 0.5,
                    size,
                    size,
                    size * 0.5,
                    color,
                );
            }
        }
    }

    /// Render 48 falling snowflakes / cosmic stardust drifting gently across the viewport.
    fn render_snow(ui: &mut Ui, time_sec: f32, w: f32, h: f32) {
        const SNOW_COUNT: usize = 48;
        for i in 0..SNOW_COUNT {
            let seed = (i * 1597334677) as u32;
            let base_x = (((seed & 0xFFFF) as f32) / 65535.0) * w;
            let base_y = ((((seed >> 16) & 0xFFFF) as f32) / 65535.0) * h;

            let speed = 22.0 + (((seed % 31) as f32) / 31.0) * 38.0; // 22 - 60 dp/sec
            let sway_amp = 12.0 + (((seed % 19) as f32) / 19.0) * 22.0;
            let sway_freq = 0.6 + (((seed % 13) as f32) / 13.0) * 1.2;
            let wind = 6.0 + (((seed % 11) as f32) / 11.0) * 12.0;

            let particle_size = 1.6 + (((seed % 17) as f32) / 17.0) * 2.2; // 1.6 - 3.8 dp

            // Smooth cyclic movement with horizontal sway
            let x = (base_x + (time_sec * sway_freq + i as f32).sin() * sway_amp + time_sec * wind)
                .rem_euclid(w);
            let y = (base_y + time_sec * speed).rem_euclid(h);

            let pulse = ((time_sec * 2.0 + i as f32 * 0.7).sin() * 0.5 + 0.5) * 0.3 + 0.7;
            let alpha = (140.0 * pulse) as u8;

            let snow_color = rgba(224, 242, 254, alpha);
            ui.rounded(
                x - particle_size * 0.5,
                y - particle_size * 0.5,
                particle_size,
                particle_size,
                particle_size * 0.5,
                snow_color,
            );
        }
    }
}

/// Draw a 4-point cross star sparkle.
fn draw_sparkle(ui: &mut Ui, cx: f32, cy: f32, size: f32, color: u32) {
    let arm = size * 1.3;
    let thick = 1.0;
    // Horizontal sparkle arm
    ui.rounded(cx - arm, cy - thick * 0.5, arm * 2.0, thick, thick * 0.5, color);
    // Vertical sparkle arm
    ui.rounded(cx - thick * 0.5, cy - arm, thick, arm * 2.0, thick * 0.5, color);
    // Center bright core
    ui.rounded(cx - size * 0.4, cy - size * 0.4, size * 0.8, size * 0.8, size * 0.4, color);
}

/// Draw a smooth rotated line between two points in UI density coordinates.
fn draw_line(ui: &mut Ui, p0: [f32; 2], p1: [f32; 2], thickness: f32, color: u32) {
    let dx = p1[0] - p0[0];
    let dy = p1[1] - p0[1];
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let half = thickness * 0.5;
    let nx = -dy / len * half;
    let ny = dx / len * half;
    let density = ui.density;
    let s = ui.batch.scale;
    let x0 = (p0[0] * density).round() / s;
    let y0 = (p0[1] * density).round() / s;
    let x1 = (p1[0] * density).round() / s;
    let y1 = (p1[1] * density).round() / s;
    let nx_px = nx * density / s;
    let ny_px = ny * density / s;

    let pts = [
        [x0 + nx_px, y0 + ny_px],
        [x1 + nx_px, y1 + ny_px],
        [x1 - nx_px, y1 - ny_px],
        [x0 - nx_px, y0 - ny_px],
    ];
    let atlas = ui.atlas;
    ui.batch.fill_quad(atlas, pts, color);
}
