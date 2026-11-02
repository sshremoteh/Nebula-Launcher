use super::*;
use horizon_client::{hud::rgba, osk::Output, ui_theme::color};
use winit::{
    event::{KeyEvent, Touch},
    keyboard::{KeyCode, PhysicalKey},
};

static NEBULA_ICON: &[u8] = include_bytes!("nebula_icon.bin");

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Action {
    Page(Page),
    Play,
    Search,
    ShaderSource(crate::shader_catalog::Origin),
    ModSource(crate::shader_catalog::Origin),
    Field(&'static str),
    FinishNameEdit,
    Install(usize),
    Import(horizon_client::content_files::Kind),
    OpenFolder(horizon_client::content_files::Kind),
    ToggleSection(usize),
    ToggleLocal(usize),
    DeleteLocal(usize),
    ConfirmDeleteLocal,
    ConfirmExit,
    Next,
    Previous,
    Profile,
    Logs,
    RecordPerformance,
    ExportPerformance,
    ExportDiagnostics,
    Account,
    SignIn,
    DeviceSignIn,
    CopyLoginCode,
    CancelLogin,
    ReviewApp,
    Repository,
    SignOut,
    LoginLink,
    CloseLogin,
    Renderer,
    RenderScale,
    ChooseRenderScale(crate::display::RenderScale),
    Orientation,
    InputHost,
    ChooseInputHost(crate::catalog::InputHost),
    ChooseOrientation(crate::catalog::Orientation),
    ChooseRenderer(crate::catalog::RendererMode),
    ChooseProfile(u32),
    Dismiss,
    Immersive,
    Cutout,
    Aspect,
    ChooseAspect(crate::display::Aspect),
    DisplayHz,
    ChooseDisplayHz(u16),
    CacheBudget,
    ChooseCacheBudget(u32),
    ToggleBundled(&'static str),
    WorldFiles,
    GameFiles,
    WorkerLimit,
    ChooseWorkerLimit(u32),
    PerformanceHints,
    GpuProfile,
}

pub(super) fn button(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    label: &str,
    action: Action,
    accent: bool,
) {
    let [x, y, w, h] = rect;
    if accent {
        // Vibrant emerald gradient
        ui.gradient(x, y, w, h, rgba(34, 197, 94, 255), rgba(16, 149, 74, 255));
        ui.border(x, y, w, h, 6.0, rgba(74, 222, 128, 220));
    } else {
        // Modern glass slate gradient plate
        ui.gradient(x, y, w, h, rgba(32, 40, 56, 230), rgba(20, 25, 36, 230));
        ui.border(x, y, w, h, 6.0, rgba(58, 70, 94, 180));
    }
    let padding = (w * 0.08).clamp(6.0, 16.0);
    let base = if w < 80.0 { 12.0 } else if h > 46.0 { 16.0 } else { 14.0 };
    let font = if matches!(action, Action::Page(_)) {
        (base * ((w - 2.0 * padding) / ui.measure(Face::Medium, base, label).max(1.0)).min(1.0))
            .max(9.0)
    } else {
        base
    };
    let label = ui.fit_text(Face::Medium, font, label, w - 2.0 * padding);
    let text_y = y + (h - font * 1.05) * 0.5;
    ui.text(
        Face::Medium,
        font,
        x + padding,
        text_y,
        if accent {
            rgba(255, 255, 255, 255)
        } else {
            rgba(241, 245, 249, 255)
        },
        &label,
    );
    targets.push((rect, action));
}
fn toggle(
    ui: &mut Ui,
    targets: &mut Vec<([f32; 4], Action)>,
    rect: [f32; 4],
    on: bool,
    action: Action,
) {
    let [x, y, w, h] = rect;
    ui.text(
        Face::Regular,
        14.0,
        x,
        y + 11.0,
        color::TEXT,
        if on { "On" } else { "Off" },
    );
    ui.rounded(
        x + w - 52.0,
        y + 8.0,
        48.0,
        24.0,
        12.0,
        if on { color::TEXT } else { color::HOVER },
    );
    ui.rounded(
        x + w - if on { 26.0 } else { 50.0 },
        y + 10.0,
        20.0,
        20.0,
        10.0,
        if on { color::BG } else { color::TEXT },
    );
    targets.push(([x, y, w, h], action));
}
impl Launcher {
    /// The same spatial focus algorithm as the game's controller menus.
    pub(super) fn navigate(&mut self, key: &KeyEvent) -> bool {
        self.navigate_key(&key.logical_key)
    }
    pub(super) fn navigate_key(&mut self, key: &winit::keyboard::Key) -> bool {
        use horizon_client::gamepad::{self, Button};
        use winit::keyboard::{Key, NamedKey};
        if self.pending_focus.is_some() {
            return true;
        }
        if self.catalog.editing.is_some() || self.targets.is_empty() {
            return false;
        }
        let button = match key {
            Key::Named(NamedKey::ArrowUp) => Button::DPadUp,
            Key::Named(NamedKey::ArrowDown) => Button::DPadDown,
            Key::Named(NamedKey::ArrowLeft) => Button::DPadLeft,
            Key::Named(NamedKey::ArrowRight) => Button::DPadRight,
            Key::Named(NamedKey::Tab) => {
                self.focus = Some(self.focus.map_or(0, |i| (i + 1) % self.targets.len()));
                return true;
            }
            _ => return false,
        };
        if matches!(button, Button::DPadUp | Button::DPadDown)
            && let Some((_, action)) = self.focus.and_then(|i| self.targets.get(i))
            && let Some(index) = self.list_targets.iter().position(|(_, a)| a == action)
        {
            let rects: Vec<_> = self.list_targets.iter().map(|(r, _)| *r).collect();
            if let Some(next) = gamepad::focus_step_strict(rects[index], button, &rects)
                && let Some(view) = self.list_view
            {
                let (rect, action) = self.list_targets[next];
                let scroll = if let Some(chooser) = &mut self.chooser {
                    &mut chooser.scroll
                } else {
                    &mut self.catalog.scroll
                };
                *scroll = super::gesture::reveal(*scroll, rect, view);
                self.pending_focus = Some(action);
                self.focus = None;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                return true;
            }
        }
        self.focus = Some(
            if let Some(index) = self.focus.filter(|i| *i < self.targets.len()) {
                let rects: Vec<_> = self.targets.iter().map(|(rect, _)| *rect).collect();
                gamepad::focus_step_rect(rects[index], button, &rects).unwrap_or(index)
            } else {
                0
            },
        );
        true
    }

    pub(super) fn draw_pages(&mut self) -> Result<()> {
        let (Some(r), Some(gui)) = (&mut self.renderer, &self.gui) else {
            return Ok(());
        };
        if r.suspended {
            return Ok(());
        }
        self.targets.clear();
        self.list_targets.clear();
        self.list_view = None;
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let mut ui = Ui::mobile(&gui.atlas, &mut b, (0.0, 0.0));
        // OEM navigation bars can remain visible even in fullscreen (keypad phones).
        // Inset the launcher layout only; keep surface and input coordinates intact.
        if let Some(window) = &self.window {
            use winit::platform::android::WindowExtAndroid;
            let rect = window.content_rect();
            let input = winit::platform::android::native_input_scale();
            if rect.bottom > 0 {
                ui.h = ui.h.min(rect.bottom as f32 * input[1] as f32 / ui.density);
            }
            if rect.right > 0 {
                ui.w = ui.w.min(rect.right as f32 * input[0] as f32 / ui.density);
            }
        }
        if let Some(bottom) = crate::native_ime::viewport_bottom() {
            ui.h = ui.h.min(bottom as f32 / ui.density);
        }
        let (w, h) = (ui.w, ui.h);
        let scale = ui.density / ui.batch.scale;
        // Deep cosmic gradient backdrop
        ui.batch.fill_gradient(
            &gui.atlas,
            (0.0, 0.0, w * scale, h * scale),
            rgba(15, 23, 42, 255),
            rgba(8, 10, 16, 255),
        );

        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let pulse = ((now_millis % 2400) as f32 / 2400.0 * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        ui.text(
            Face::SemiBold,
            if w < 400.0 { 18.0 } else if w < 700.0 { 22.0 } else { 26.0 },
            24.0,
            if w < 620.0 || h > w { 16.0 } else { 22.0 },
            rgba(248, 250, 252, 255),
            "NEBULA LAUNCHER • 1.0 Release",
        );
        button(
            &mut ui,
            &mut self.targets,
            [
                if w < 700.0 { w - 210.0 } else { w - 290.0 },
                if w < 620.0 || h > w { 12.0 } else { 16.0 },
                if w < 700.0 { 194.0 } else { 266.0 },
                if w < 620.0 || h > w { 34.0 } else { 38.0 },
            ],
            &self.login.status(&self.catalog.config.name),
            Action::Account,
            self.login.signed_in,
        );
        let compact = w < 620.0 || h > w;
        let page = self.catalog.page;

        // Sub-navigation bar inside Settings (when not in Main Menu)
        if page != Page::Play {
            let tab_margin = if compact { 16.0 } else { 24.0 };
            let back_w = if compact { 80.0 } else { 100.0 };
            button(
                &mut ui,
                &mut self.targets,
                [tab_margin, 76.0, back_w, 38.0],
                "< Меню",
                Action::Page(Page::Play),
                false,
            );
            let tabs = Page::SETTINGS_TABS.len() as f32;
            let sub_start = tab_margin + back_w + 8.0;
            let tab_width = (w - sub_start - tab_margin) / tabs;
            for (i, tab) in Page::SETTINGS_TABS.iter().enumerate() {
                let x = sub_start + i as f32 * tab_width;
                button(
                    &mut ui,
                    &mut self.targets,
                    [x, 76.0, tab_width - 4.0, 38.0],
                    if compact {
                        match tab {
                            Page::Installed => "Файлы",
                            Page::Settings => "Опции",
                            Page::Packs => "Паки",
                            _ => tab.tab_title(),
                        }
                    } else {
                        tab.tab_title()
                    },
                    Action::Page(*tab),
                    *tab == page,
                );
            }
        }

        if page == Page::Play {
            use horizon_client::official_install::InstallStage;
            let installed = horizon_client::official_install::ready(&self.home);
            let installing = self.worker.is_some();
            let progress = *self.install.lock().unwrap();

            if compact {
                // Stacked layout for compact / portrait screens
                let pad = 16.0;
                let card_w = w - 2.0 * pad;

                // 1. Nickname card with gradient
                let nick_y = 64.0;
                let nick_h = 72.0;
                ui.gradient(pad, nick_y, card_w, nick_h, rgba(28, 36, 52, 230), rgba(18, 23, 34, 230));
                ui.border(pad, nick_y, card_w, nick_h, 6.0, rgba(59, 130, 246, 140));
                ui.text(Face::Medium, 11.0, pad + 10.0, nick_y + 10.0, rgba(148, 163, 184, 255), "ИМЯ ИГРОКА (АВТОСОХРАНЕНИЕ)");
                let field_w = card_w - 20.0 - 82.0;
                let nick_display = if self.catalog.config.name.trim().is_empty() {
                    "Игрок (введите ник)"
                } else {
                    &self.catalog.config.name
                };
                button(
                    &mut ui,
                    &mut self.targets,
                    [pad + 10.0, nick_y + 26.0, field_w, 36.0],
                    nick_display,
                    Action::Field("name"),
                    false,
                );
                let is_editing_name = self.catalog.editing == Some("name");
                button(
                    &mut ui,
                    &mut self.targets,
                    [pad + 10.0 + field_w + 6.0, nick_y + 26.0, 76.0, 36.0],
                    if is_editing_name { "OK" } else { "Сохранить" },
                    Action::FinishNameEdit,
                    is_editing_name,
                );

                // 2. Version & Profile row with gradient
                let ver_y = nick_y + nick_h + 8.0;
                let ver_h = 86.0;
                ui.gradient(pad, ver_y, card_w, ver_h, rgba(28, 36, 52, 230), rgba(18, 23, 34, 230));
                ui.border(pad, ver_y, card_w, ver_h, 6.0, rgba(71, 85, 105, 180));
                
                // Icon from icon.jpg
                let icon_sz = 34.0;
                ui.rounded(pad + 10.0, ver_y + 6.0, icon_sz, icon_sz, 6.0, rgba(15, 23, 42, 255));
                ui.draw_icon_rgba(pad + 10.0, ver_y + 6.0, icon_sz, icon_sz, NEBULA_ICON, 96, 96);
                ui.border(pad + 10.0, ver_y + 6.0, icon_sz, icon_sz, 6.0, rgba(129, 140, 248, (180.0 + pulse * 75.0) as u8));

                ui.text(Face::SemiBold, 14.0, pad + 10.0 + icon_sz + 8.0, ver_y + 8.0, rgba(248, 250, 252, 255), "Minecraft 26.3");
                ui.text(Face::Regular, 10.0, pad + 10.0 + icon_sz + 8.0, ver_y + 24.0, rgba(148, 163, 184, 255), "Nebula Engine");

                let (st_txt, st_col) = if installing {
                    ("Установка...", rgba(56, 189, 248, 255))
                } else if installed {
                    ("Готово к игре", rgba(52, 211, 153, 255))
                } else {
                    ("Нужна загрузка", rgba(251, 191, 36, 255))
                };
                let st_w = ui.measure(Face::Medium, 11.0, st_txt);
                let st_x = pad + card_w - st_w - 12.0;
                ui.rounded(st_x - 10.0, ver_y + 16.0, 6.0, 6.0, 3.0, st_col);
                ui.text(Face::Medium, 11.0, st_x, ver_y + 13.0, st_col, st_txt);

                let btn_half_w = (card_w - 26.0) * 0.5;
                button(
                    &mut ui,
                    &mut self.targets,
                    [pad + 10.0, ver_y + 44.0, btn_half_w, 36.0],
                    &format!("Профиль: {}", self.catalog.config.profile),
                    Action::Profile,
                    false,
                );
                button(
                    &mut ui,
                    &mut self.targets,
                    [pad + 10.0 + btn_half_w + 6.0, ver_y + 44.0, btn_half_w, 36.0],
                    &format!("Графика: {}", self.catalog.config.renderer.label()),
                    Action::Renderer,
                    false,
                );

                // 3. Play or Progress Area
                let play_y = ver_y + ver_h + 8.0;
                let play_h = 52.0;
                if installing {
                    let current = InstallStage::ALL.iter().position(|s| *s == progress.stage).unwrap_or(0);
                    let fraction = if progress.total > 0 { progress.done as f32 / progress.total as f32 } else { 0.0 };
                    let pct = (fraction * 100.0).clamp(0.0, 100.0);
                    let overall_pct = ((current as f32 + fraction.clamp(0.0, 1.0)) / 6.0 * 100.0).clamp(0.0, 100.0);

                    let box_h = 76.0;
                    ui.gradient(pad, play_y, card_w, box_h, rgba(22, 29, 44, 230), rgba(15, 20, 30, 230));
                    ui.border(pad, play_y, card_w, box_h, 8.0, rgba(56, 189, 248, 160));

                    ui.text(Face::SemiBold, 13.0, pad + 10.0, play_y + 8.0, rgba(248, 250, 252, 255), progress.stage.label());
                    ui.text(Face::SemiBold, 14.0, pad + card_w - 55.0, play_y + 8.0, rgba(52, 211, 153, 255), &format!("{:.1}%", pct));

                    let bar_y = play_y + 30.0;
                    let bar_w = card_w - 20.0;
                    ui.gradient(pad + 10.0, bar_y, bar_w, 10.0, rgba(15, 23, 42, 255), rgba(30, 41, 59, 255));
                    ui.border(pad + 10.0, bar_y, bar_w, 10.0, 5.0, rgba(51, 65, 85, 200));

                    let fill_w = (bar_w * fraction.clamp(0.02, 1.0)).min(bar_w);
                    ui.gradient(pad + 10.0, bar_y, fill_w, 10.0, rgba(52, 211, 153, 255), rgba(16, 185, 129, 255));

                    let wave_x = pad + 10.0 + (fill_w - 12.0).max(0.0) * ((now_millis % 1500) as f32 / 1500.0);
                    ui.rounded(wave_x, bar_y + 1.0, 10.0, 8.0, 4.0, rgba(255, 255, 255, (100.0 * pulse + 50.0) as u8));

                    let stat = format!("Общий: {:.0}%  •  Этап {}/6", overall_pct, current + 1);
                    ui.text(Face::Regular, 11.0, pad + 10.0, play_y + 48.0, rgba(148, 163, 184, 255), &stat);
                } else {
                    let play_label = if installed {
                        "И Г Р А Т Ь"
                    } else if self.error.is_some() {
                        "ПОВТОРИТЬ ЗАГРУЗКУ"
                    } else {
                        "СКАЧАТЬ MINECRAFT 26.3"
                    };
                    if installed {
                        let glow = (180.0 + pulse * 75.0) as u8;
                        ui.gradient(pad, play_y, card_w, play_h, rgba(34, 197, 94, 255), rgba(16, 149, 74, 255));
                        ui.border(pad, play_y, card_w, play_h, 8.0, rgba(74, 222, 128, glow));
                    } else {
                        ui.gradient(pad, play_y, card_w, play_h, rgba(59, 130, 246, 255), rgba(37, 99, 235, 255));
                        ui.border(pad, play_y, card_w, play_h, 8.0, rgba(147, 197, 253, 200));
                    }
                    let font_size = 17.0;
                    let label_fit = ui.fit_text(Face::SemiBold, font_size, play_label, card_w - 20.0);
                    let text_w = ui.measure(Face::SemiBold, font_size, &label_fit);
                    if installed {
                        let tri_sz = 12.0;
                        let total_w = tri_sz + 10.0 + text_w;
                        let text_start = pad + (card_w - total_w) * 0.5;
                        ui.play_triangle(text_start, play_y + (play_h - tri_sz) * 0.5, tri_sz, rgba(255, 255, 255, 255));
                        ui.text(Face::SemiBold, font_size, text_start + tri_sz + 10.0, play_y + (play_h - font_size * 1.1) * 0.5, rgba(255, 255, 255, 255), &label_fit);
                    } else {
                        let text_x = pad + (card_w - text_w) * 0.5;
                        let text_y = play_y + (play_h - font_size * 1.1) * 0.5;
                        ui.text(Face::SemiBold, font_size, text_x, text_y, rgba(255, 255, 255, 255), &label_fit);
                    }
                    self.targets.push(([pad, play_y, card_w, play_h], Action::Play));
                }

                // 4. Settings button (Indigo gradient)
                let set_y = play_y + (if installing { 84.0 } else { play_h + 8.0 });
                ui.gradient(pad, set_y, card_w, 42.0, rgba(99, 102, 241, 230), rgba(67, 56, 202, 230));
                ui.border(pad, set_y, card_w, 42.0, 6.0, rgba(165, 180, 252, 180));
                let set_label = "Настройки (моды, шейдеры, паки)";
                let set_txt_w = ui.measure(Face::Medium, 14.0, set_label);
                let sliders_w = 16.0;
                let sliders_h = 12.0;
                let total_set_w = sliders_w + 8.0 + set_txt_w;
                let set_start_x = pad + (card_w - total_set_w) * 0.5;
                ui.sliders_icon(set_start_x, set_y + (42.0 - sliders_h) * 0.5, sliders_w, sliders_h, rgba(255, 255, 255, 220));
                ui.text(Face::Medium, 14.0, set_start_x + sliders_w + 8.0, set_y + 13.0, rgba(255, 255, 255, 255), set_label);
                self.targets.push(([pad, set_y, card_w, 42.0], Action::Page(Page::Settings)));

                // 5. Changelog
                let cl_y = set_y + 50.0;
                let cl_h = (h - cl_y - 12.0).max(60.0);
                ui.gradient(pad, cl_y, card_w, cl_h, rgba(22, 28, 40, 230), rgba(14, 18, 26, 230));
                ui.border(pad, cl_y, card_w, cl_h, 6.0, rgba(51, 65, 85, 180));
                ui.text(Face::SemiBold, 13.0, pad + 10.0, cl_y + 10.0, rgba(248, 250, 252, 255), "Что нового в Nebula 1.0");
                let items = [
                    "• Оптимизация Vulkan & OpenGL ES с высоким FPS",
                    "• Сенсорное управление Bedrock Touch",
                    "• Поддержка оффлайн игры и Java серверов",
                    "• Менеджер модов, ресурспаков и шейдеров",
                ];
                for (idx, item) in items.iter().enumerate() {
                    let y = cl_y + 28.0 + idx as f32 * 18.0;
                    if y + 14.0 <= cl_y + cl_h {
                        ui.text(Face::Regular, 11.0, pad + 10.0, y, rgba(148, 163, 184, 255), item);
                    }
                }
            } else {
                // Split-panel dashboard for landscape screens (w >= 620.0 and w >= h)
                let margin = 20.0;
                let gap = 16.0;
                let col_w = (w - 2.0 * margin - gap) * 0.5;
                let left_x = margin;
                let right_x = margin + col_w + gap;
                let content_top = 64.0;
                let content_bottom = h - 16.0;

                // --- LEFT PANEL ---
                // 1. Nickname card with subtle gradient
                let nick_h = 66.0;
                ui.gradient(left_x, content_top, col_w, nick_h, rgba(28, 36, 52, 230), rgba(18, 23, 34, 230));
                ui.border(left_x, content_top, col_w, nick_h, 6.0, rgba(59, 130, 246, 140));
                ui.text(Face::Medium, 11.0, left_x + 12.0, content_top + 8.0, rgba(148, 163, 184, 255), "ИМЯ ИГРОКА (АВТОСОХРАНЕНИЕ)");
                let field_w = col_w - 24.0 - 86.0;
                let nick_display = if self.catalog.config.name.trim().is_empty() {
                    "Игрок (введите ник)"
                } else {
                    &self.catalog.config.name
                };
                button(
                    &mut ui,
                    &mut self.targets,
                    [left_x + 12.0, content_top + 24.0, field_w, 34.0],
                    nick_display,
                    Action::Field("name"),
                    false,
                );
                let is_editing_name = self.catalog.editing == Some("name");
                button(
                    &mut ui,
                    &mut self.targets,
                    [left_x + 12.0 + field_w + 6.0, content_top + 24.0, 80.0, 34.0],
                    if is_editing_name { "OK" } else { "Сохранить" },
                    Action::FinishNameEdit,
                    is_editing_name,
                );

                // 2. Settings button (Indigo gradient)
                let set_btn_top = content_top + nick_h + 8.0;
                let set_btn_h = 38.0;
                let repo_w = 76.0;
                let set_w = col_w - repo_w - 8.0;
                ui.gradient(left_x, set_btn_top, set_w, set_btn_h, rgba(99, 102, 241, 230), rgba(67, 56, 202, 230));
                ui.border(left_x, set_btn_top, set_w, set_btn_h, 6.0, rgba(165, 180, 252, 190));
                let set_txt = "Настройки лаунчера";
                let set_tw = ui.measure(Face::Medium, 13.0, set_txt);
                let sliders_w = 16.0;
                let sliders_h = 12.0;
                let total_set_w = sliders_w + 8.0 + set_tw;
                let set_start_x = left_x + (set_w - total_set_w) * 0.5;
                ui.sliders_icon(set_start_x, set_btn_top + (set_btn_h - sliders_h) * 0.5, sliders_w, sliders_h, rgba(255, 255, 255, 220));
                ui.text(Face::Medium, 13.0, set_start_x + sliders_w + 8.0, set_btn_top + 12.0, rgba(255, 255, 255, 255), set_txt);
                self.targets.push(([left_x, set_btn_top, set_w, set_btn_h], Action::Page(Page::Settings)));

                button(
                    &mut ui,
                    &mut self.targets,
                    [left_x + set_w + 8.0, set_btn_top, repo_w, set_btn_h],
                    "GitHub",
                    Action::Repository,
                    false,
                );

                // 3. Changelog card
                let cl_top = set_btn_top + set_btn_h + 8.0;
                let cl_h = (content_bottom - cl_top).max(80.0);
                ui.gradient(left_x, cl_top, col_w, cl_h, rgba(22, 28, 40, 230), rgba(14, 18, 26, 230));
                ui.border(left_x, cl_top, col_w, cl_h, 6.0, rgba(51, 65, 85, 180));
                ui.text(Face::SemiBold, 13.0, left_x + 12.0, cl_top + 10.0, rgba(248, 250, 252, 255), "Что нового в Nebula 1.0");
                ui.text(Face::Regular, 11.0, left_x + col_w - 74.0, cl_top + 12.0, rgba(148, 163, 184, 255), "v1.0 Release");

                let logs = [
                    "• Оптимизация Vulkan шейдинга и высокий FPS",
                    "• Сенсорное управление Bedrock Touch",
                    "• Поддержка оффлайн игры и Java серверов",
                    "• Менеджер модов, ресурспаков и шейдеров",
                    "• Экономия ОЗУ и стабильность от 3 ГБ",
                ];
                for (idx, title) in logs.iter().enumerate() {
                    let y = cl_top + 30.0 + idx as f32 * 20.0;
                    if y + 14.0 <= cl_top + cl_h {
                        ui.text(Face::Regular, 11.0, left_x + 12.0, y, rgba(148, 163, 184, 255), title);
                    }
                }

                // --- RIGHT PANEL ---
                // 1. Hero / Version card with gradient
                let ver_card_top = content_top;
                let ver_card_h = 100.0;
                ui.gradient(right_x, ver_card_top, col_w, ver_card_h, rgba(28, 36, 52, 230), rgba(18, 23, 34, 230));
                ui.border(right_x, ver_card_top, col_w, ver_card_h, 6.0, rgba(71, 85, 105, 180));
                
                // Real app icon from icon.jpg + Titles + Pulsing icon border
                let h_glow = (180.0 + pulse * 75.0) as u8;
                ui.rounded(right_x + 12.0, ver_card_top + 10.0, 38.0, 38.0, 8.0, rgba(15, 23, 42, 255));
                ui.draw_icon_rgba(right_x + 12.0, ver_card_top + 10.0, 38.0, 38.0, NEBULA_ICON, 96, 96);
                ui.border(right_x + 12.0, ver_card_top + 10.0, 38.0, 38.0, 8.0, rgba(129, 140, 248, h_glow));
                ui.text(Face::SemiBold, 15.0, right_x + 58.0, ver_card_top + 10.0, rgba(248, 250, 252, 255), "MINECRAFT JAVA");
                ui.text(Face::Regular, 11.0, right_x + 58.0, ver_card_top + 28.0, rgba(148, 163, 184, 255), "26.3 Release • Nebula Engine");

                let (st_txt, st_col) = if installing {
                    ("Установка...", rgba(56, 189, 248, 255))
                } else if installed {
                    ("Готово к игре", rgba(52, 211, 153, 255))
                } else {
                    ("Нужна загрузка", rgba(251, 191, 36, 255))
                };
                let st_w = ui.measure(Face::Medium, 11.0, st_txt);
                let st_x = right_x + col_w - st_w - 14.0;
                ui.rounded(st_x - 10.0, ver_card_top + 14.0, 6.0, 6.0, 3.0, st_col);
                ui.text(Face::Medium, 11.0, st_x, ver_card_top + 11.0, st_col, st_txt);

                // Profile and Renderer buttons side by side
                let sub_btn_w = (col_w - 24.0 - 8.0) * 0.5;
                button(
                    &mut ui,
                    &mut self.targets,
                    [right_x + 12.0, ver_card_top + 54.0, sub_btn_w, 36.0],
                    &format!("Профиль: {}", self.catalog.config.profile),
                    Action::Profile,
                    false,
                );
                button(
                    &mut ui,
                    &mut self.targets,
                    [right_x + 12.0 + sub_btn_w + 8.0, ver_card_top + 54.0, sub_btn_w, 36.0],
                    &format!("Графика: {}", self.catalog.config.renderer.label()),
                    Action::Renderer,
                    false,
                );

                // 2. Play or Progress Area
                let play_btn_h = 54.0;
                let play_top = (content_bottom - play_btn_h).max(ver_card_top + ver_card_h + 12.0);

                if installing {
                    let current = InstallStage::ALL.iter().position(|s| *s == progress.stage).unwrap_or(0);
                    let fraction = if progress.total > 0 { progress.done as f32 / progress.total as f32 } else { 0.0 };
                    let pct = (fraction * 100.0).clamp(0.0, 100.0);
                    let overall_pct = ((current as f32 + fraction.clamp(0.0, 1.0)) / 6.0 * 100.0).clamp(0.0, 100.0);

                    let box_y = (play_top - 40.0).max(ver_card_top + ver_card_h + 8.0);
                    let box_h = play_btn_h + 40.0;
                    ui.gradient(right_x, box_y, col_w, box_h, rgba(22, 29, 44, 230), rgba(15, 20, 30, 230));
                    ui.border(right_x, box_y, col_w, box_h, 8.0, rgba(56, 189, 248, 160));

                    // Title with pulsating animated status
                    ui.text(Face::SemiBold, 14.0, right_x + 12.0, box_y + 10.0, rgba(248, 250, 252, 255), progress.stage.label());
                    ui.text(Face::SemiBold, 15.0, right_x + col_w - 65.0, box_y + 10.0, rgba(52, 211, 153, 255), &format!("{:.1}%", pct));

                    // Exact detail string (bytes/files count)
                    let detail = match progress.stage {
                        InstallStage::Download => format!(
                            "{:.1} / {:.1} МБ  ({:.0}% файла)",
                            progress.done as f64 / 1_048_576.0,
                            progress.total as f64 / 1_048_576.0,
                            pct
                        ),
                        InstallStage::Controls => format!("{} / {} файлов управления", progress.done, progress.total),
                        InstallStage::WorldData => "Распаковка таблиц и генерации мира...".into(),
                        InstallStage::Extract => "Извлечение игровых ассетов и текстур...".into(),
                        _ => format!("Обработка данных... {:.0}%", pct),
                    };
                    ui.text(Face::Regular, 11.0, right_x + 12.0, box_y + 28.0, rgba(148, 163, 184, 255), &detail);

                    // Stage Progress bar
                    let bar_y = box_y + 46.0;
                    let bar_w = col_w - 24.0;
                    let bar_h = 10.0;
                    ui.gradient(right_x + 12.0, bar_y, bar_w, bar_h, rgba(15, 23, 42, 255), rgba(30, 41, 59, 255));
                    ui.border(right_x + 12.0, bar_y, bar_w, bar_h, 5.0, rgba(51, 65, 85, 200));

                    let fill_w = (bar_w * fraction.clamp(0.02, 1.0)).min(bar_w);
                    ui.gradient(right_x + 12.0, bar_y, fill_w, bar_h, rgba(52, 211, 153, 255), rgba(16, 185, 129, 255));

                    // Animated wave pulse on progress bar
                    let wave_pos = (now_millis % 1600) as f32 / 1600.0;
                    let wave_x = right_x + 12.0 + (fill_w - 14.0).max(0.0) * wave_pos;
                    ui.rounded(wave_x, bar_y + 1.0, 12.0, bar_h - 2.0, 4.0, rgba(255, 255, 255, (120.0 * pulse + 60.0) as u8));

                    // Overall percentage footer
                    let overall_text = format!("Общий прогресс: {:.0}%  •  Этап {} из 6", overall_pct, current + 1);
                    ui.text(Face::Medium, 11.0, right_x + 12.0, box_y + 64.0, rgba(203, 213, 225, 255), &overall_text);
                } else {
                    let play_label = if installed {
                        "И Г Р А Т Ь"
                    } else if self.error.is_some() {
                        "ПОВТОРИТЬ ЗАГРУЗКУ"
                    } else {
                        "СКАЧАТЬ MINECRAFT 26.3"
                    };

                    // Animated pulsing gradient Play button
                    if installed {
                        let glow = (180.0 + pulse * 75.0) as u8;
                        ui.gradient(right_x, play_top, col_w, play_btn_h, rgba(34, 197, 94, 255), rgba(16, 149, 74, 255));
                        ui.border(right_x, play_top, col_w, play_btn_h, 8.0, rgba(74, 222, 128, glow));
                    } else {
                        ui.gradient(right_x, play_top, col_w, play_btn_h, rgba(59, 130, 246, 255), rgba(37, 99, 235, 255));
                        ui.border(right_x, play_top, col_w, play_btn_h, 8.0, rgba(147, 197, 253, 200));
                    }
                    let font_size = 18.0;
                    let label_fit = ui.fit_text(Face::SemiBold, font_size, play_label, col_w - 24.0);
                    let text_w = ui.measure(Face::SemiBold, font_size, &label_fit);
                    let text_y = play_top + (play_btn_h - font_size * 1.1) * 0.5;
                    if installed {
                        let tri_sz = 14.0;
                        let total_w = tri_sz + 10.0 + text_w;
                        let text_start = right_x + (col_w - total_w) * 0.5;
                        ui.play_triangle(text_start, play_top + (play_btn_h - tri_sz) * 0.5, tri_sz, rgba(255, 255, 255, 255));
                        ui.text(Face::SemiBold, font_size, text_start + tri_sz + 10.0, text_y, rgba(255, 255, 255, 255), &label_fit);
                    } else {
                        let text_x = right_x + (col_w - text_w) * 0.5;
                        ui.text(Face::SemiBold, font_size, text_x, text_y, rgba(255, 255, 255, 255), &label_fit);
                    }
                    self.targets.push(([right_x, play_top, col_w, play_btn_h], Action::Play));
                }
            }
        } else if page.is_catalog() {
            ui.text(
                Face::Medium,
                18.0,
                32.0,
                142.0,
                color::TEXT,
                if page == Page::Mods {
                    "Моды"
                } else if page == Page::Shaders {
                    if w < 360.0 { "Shaders" } else { "Shader Packs" }
                } else {
                    "Resource Packs"
                },
            );
            button(
                &mut ui,
                &mut self.targets,
                [32.0, 174.0, w - 240.0, 44.0],
                &format!(
                    "{}{}",
                    if page == Page::Mods {
                        "Search / .hmod URL: "
                    } else {
                        "Search: "
                    },
                    self.catalog.query
                ),
                Action::Field("search"),
                false,
            );
            button(
                &mut ui,
                &mut self.targets,
                [w - 190.0, 174.0, 158.0, 44.0],
                if page == Page::Mods && self.catalog.query.trim().starts_with("https://") {
                    "Import .hmod"
                } else {
                    "Search"
                },
                Action::Search,
                true,
            );
            if matches!(page, Page::Shaders | Page::Mods) {
                use crate::shader_catalog::Origin;
                let width = ((w - 185.0) / 2.0).clamp(58.0, 120.0);
                for (i, origin) in [Origin::Horizon, Origin::Modrinth].into_iter().enumerate() {
                    let rect = [w - 32.0 - (2 - i) as f32 * width, 134.0, width - 5.0, 32.0];
                    button(
                        &mut ui,
                        &mut self.targets,
                        rect,
                        origin.source().label(),
                        if page == Page::Mods {
                            Action::ModSource(origin)
                        } else {
                            Action::ShaderSource(origin)
                        },
                        false,
                    );
                    if (if page == Page::Mods {
                        self.catalog.config.mod_source
                    } else {
                        self.catalog.config.shader_source
                    }) == origin
                    {
                        ui.border(rect[0], rect[1], rect[2], rect[3], 1.0, color::TEXT);
                    }
                }
            }
            let hint = match page {
                Page::Mods
                    if self.catalog.config.mod_source
                        == crate::shader_catalog::Origin::Modrinth =>
                {
                    "Modrinth Java mods are browse-only; Fabric/Forge/NeoForge cannot run here"
                }
                Page::Mods => "Поиск модов или ссылка на .hmod",
                Page::Shaders
                    if self.catalog.config.shader_source
                        == crate::shader_catalog::Origin::Modrinth =>
                {
                    "Modrinth shader ZIPs / experimental compatibility; some packs may not work"
                }
                Page::Shaders => "Нативные шейдер-паки .hshader",
                _ => "Verified downloads / enable in Installed",
            };
            let hint = ui.fit_text(Face::Regular, 14.0, hint, w - 64.0);
            ui.text(Face::Regular, 14.0, 32.0, 270.0, color::MUTED, &hint);
            let kind = match page {
                Page::Mods => horizon_client::content_files::Kind::Mods,
                Page::Shaders => horizon_client::content_files::Kind::Shaders,
                _ => horizon_client::content_files::Kind::Packs,
            };
            let file_button_width = ((w - 72.0) / 2.0).min(158.0);
            button(
                &mut ui,
                &mut self.targets,
                [32.0, 222.0, file_button_width, 38.0],
                "Open folder",
                Action::OpenFolder(kind),
                false,
            );
            button(
                &mut ui,
                &mut self.targets,
                [w - 32.0 - file_button_width, 222.0, file_button_width, 38.0],
                "Import file",
                Action::Import(kind),
                false,
            );
            let bundled: Vec<_> = if page == Page::Mods
                && self.catalog.config.mod_source == crate::shader_catalog::Origin::Horizon
            {
                let query = self.catalog.query.to_lowercase();
                horizon_client::mod_list::ANDROID_BUNDLED
                    .into_iter()
                    .filter(|(id, title, _)| {
                        id.contains(&query) || title.to_lowercase().contains(&query)
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let catalog_rows: Vec<_> = self
                .catalog
                .results
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    !(page == Page::Mods
                        && p.source == crate::browser::SourceId::Horizon
                        && horizon_client::mod_list::ANDROID_BUNDLED
                            .iter()
                            .any(|(id, _, _)| *id == p.id))
                })
                .collect();
            let row_count = bundled.len() + catalog_rows.len();
            let top = 296.0;
            let bottom = h - 112.0;
            self.catalog.scroll = self
                .catalog
                .scroll
                .clamp(0.0, (row_count as f32 * 78.0 - (bottom - top)).max(0.0));
            self.list_view = Some([
                top,
                bottom,
                (row_count as f32 * 78.0 - (bottom - top)).max(0.0),
            ]);
            for (row, (id, title, description)) in bundled.iter().enumerate() {
                let y = top + row as f32 * 78.0 - self.catalog.scroll;
                if y < top - 1.0 || y + 68.0 > bottom {
                    continue;
                }
                let title = ui.fit_text(Face::Medium, 19.0, title, w - 214.0);
                let summary = ui.fit_text(Face::Regular, 14.0, description, w - 214.0);
                ui.text(Face::Medium, 19.0, 32.0, y, color::TEXT, &title);
                ui.text(Face::Regular, 14.0, 32.0, y + 29.0, color::MUTED, &summary);
                button(
                    &mut ui,
                    &mut self.targets,
                    [w - 150.0, y, 118.0, 42.0],
                    if self.catalog.bundled_enabled.contains(*id) {
                        "Disable"
                    } else {
                        "Enable"
                    },
                    Action::ToggleBundled(id),
                    false,
                );
            }
            for (row, (i, p)) in catalog_rows.into_iter().enumerate() {
                let y = top + (row + bundled.len()) as f32 * 78.0 - self.catalog.scroll;
                let download = self.catalog.download_label(p);
                if download.is_none()
                    && !crate::catalog::source_only_mod(p)
                    && !(p.kind == crate::browser::Kind::Mod
                        && p.source == crate::browser::SourceId::Modrinth)
                {
                    self.list_targets
                        .push(([w - 150.0, y, 118.0, 70.0], Action::Install(i)));
                }
                if y < top || y + 70.0 > bottom {
                    continue;
                }
                let title = ui.fit_text(Face::Medium, 19.0, &p.title, w - 214.0);
                let summary = ui.fit_text(Face::Regular, 14.0, &p.summary, w - 214.0);
                ui.text(Face::Medium, 19.0, 32.0, y, color::TEXT, &title);
                ui.text(Face::Regular, 14.0, 32.0, y + 29.0, color::MUTED, &summary);
                if let Some(state) = download {
                    ui.text(Face::Medium, 16.0, w - 150.0, y + 12.0, color::TEXT, &state);
                } else if p.kind == crate::browser::Kind::Mod
                    && p.source == crate::browser::SourceId::Modrinth
                {
                    ui.text(
                        Face::Regular,
                        14.0,
                        w - 150.0,
                        y + 12.0,
                        color::MUTED,
                        "Java mod",
                    );
                } else if crate::catalog::source_only_mod(p) {
                    ui.text(
                        Face::Regular,
                        14.0,
                        w - 150.0,
                        y + 12.0,
                        color::MUTED,
                        ".hnmod: later",
                    );
                } else if p.kind == crate::browser::Kind::Mod && !cfg!(target_pointer_width = "64")
                {
                    ui.text(
                        Face::Regular,
                        14.0,
                        w - 150.0,
                        y + 12.0,
                        color::MUTED,
                        "64-bit required",
                    );
                } else {
                    button(
                        &mut ui,
                        &mut self.targets,
                        [w - 150.0, y, 118.0, 42.0],
                        self.catalog.install_button(p),
                        Action::Install(i),
                        false,
                    );
                }
            }
            button(
                &mut ui,
                &mut self.targets,
                [32.0, h - 100.0, 120.0, 40.0],
                "Previous",
                Action::Previous,
                false,
            );
            button(
                &mut ui,
                &mut self.targets,
                [166.0, h - 100.0, 120.0, 40.0],
                "Next",
                Action::Next,
                false,
            );
        } else if page == Page::Installed {
            use horizon_client::content_files::Kind;
            let top = 156.0;
            let bottom = h - 100.0;
            let row_height = if compact { 110.0 } else { 72.0 };
            let total = Kind::ALL
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    48.0 + if self.catalog.sections[i] {
                        self.catalog.local.iter().filter(|e| e.kind == *k).count() as f32
                            * row_height
                    } else {
                        0.0
                    }
                })
                .sum::<f32>();
            let maximum = (total - (bottom - top)).max(0.0);
            self.catalog.scroll = self.catalog.scroll.clamp(0.0, maximum);
            self.list_view = Some([top, bottom, maximum]);
            let mut offset = 0.0;
            for (section, kind) in Kind::ALL.into_iter().enumerate() {
                let count = self.catalog.local.iter().filter(|e| e.kind == kind).count();
                let y = top + offset - self.catalog.scroll;
                let label = format!(
                    "{} {} ({count})",
                    if self.catalog.sections[section] {
                        "-"
                    } else {
                        "+"
                    },
                    kind.label()
                );
                self.list_targets
                    .push(([32.0, y, w - 210.0, 40.0], Action::ToggleSection(section)));
                self.list_targets
                    .push(([w - 165.0, y, 133.0, 40.0], Action::Import(kind)));
                if y >= top && y + 40.0 <= bottom {
                    button(
                        &mut ui,
                        &mut self.targets,
                        [32.0, y, w - 210.0, 40.0],
                        &label,
                        Action::ToggleSection(section),
                        false,
                    );
                    button(
                        &mut ui,
                        &mut self.targets,
                        [w - 165.0, y, 133.0, 40.0],
                        "Import",
                        Action::Import(kind),
                        false,
                    );
                }
                offset += 48.0;
                if !self.catalog.sections[section] {
                    continue;
                }
                for (i, item) in self
                    .catalog
                    .local
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.kind == kind)
                {
                    let y = top + offset - self.catalog.scroll;
                    offset += row_height;
                    self.list_targets.push((
                        [
                            if compact { 32.0 } else { w - 300.0 },
                            y,
                            120.0,
                            row_height - 4.0,
                        ],
                        Action::ToggleLocal(i),
                    ));
                    self.list_targets.push((
                        [w - 165.0, y, 133.0, row_height - 4.0],
                        Action::DeleteLocal(i),
                    ));
                    if y < top || y + row_height - 4.0 > bottom {
                        continue;
                    }
                    let title = ui.fit_text(
                        Face::Medium,
                        19.0,
                        &item.title,
                        if compact { w - 64.0 } else { w - 364.0 },
                    );
                    ui.text(Face::Medium, 19.0, 32.0, y, color::TEXT, &title);
                    ui.text(
                        Face::Regular,
                        14.0,
                        32.0,
                        y + 28.0,
                        color::MUTED,
                        kind.label(),
                    );
                    if kind == Kind::Mods && !cfg!(target_pointer_width = "64") {
                        ui.text(
                            Face::Regular,
                            14.0,
                            if compact { 32.0 } else { w - 300.0 },
                            y + if compact { 62.0 } else { 12.0 },
                            color::MUTED,
                            "64-bit required",
                        );
                    } else {
                        toggle(
                            &mut ui,
                            &mut self.targets,
                            [
                                if compact { 32.0 } else { w - 300.0 },
                                y + if compact { 55.0 } else { 0.0 },
                                120.0,
                                40.0,
                            ],
                            item.enabled,
                            Action::ToggleLocal(i),
                        );
                    }
                    button(
                        &mut ui,
                        &mut self.targets,
                        [w - 165.0, y + if compact { 55.0 } else { 0.0 }, 133.0, 40.0],
                        "Remove",
                        Action::DeleteLocal(i),
                        false,
                    );
                }
            }
        } else {
            let c = &self.catalog.config;
            ui.text(
                Face::Medium,
                24.0,
                32.0,
                156.0,
                color::TEXT,
                "Launch settings",
            );
            let row_height = if compact { 82.0 } else { 66.0 };
            self.catalog.scroll = self
                .catalog
                .scroll
                .clamp(0.0, (18.0 * row_height - (h - 130.0 - 195.0)).max(0.0));
            self.list_view = Some([
                195.0,
                h - 130.0,
                (18.0 * row_height - (h - 130.0 - 195.0)).max(0.0),
            ]);
            for (i, (title, value, action)) in [
                ("Rendering", c.renderer.label().to_owned(), Action::Renderer),
                (
                    "Client resolution",
                    if self.catalog.editing == Some("render_scale") {
                        self.catalog.render_scale_text.clone()
                    } else {
                        format!("{}%", c.render_scale.percent())
                    },
                    Action::RenderScale,
                ),
                (
                    "Chunk worker limit",
                    if c.worker_limit == 0 {
                        "No additional limit".into()
                    } else {
                        c.worker_limit.to_string()
                    },
                    Action::WorkerLimit,
                ),
                (
                    "Reusable RAM cache",
                    if c.cache_budget_mib == 0 {
                        "Engine default (80 MiB)".into()
                    } else {
                        format!("{} MiB", c.cache_budget_mib)
                    },
                    Action::CacheBudget,
                ),
                (
                    "Game files",
                    "Open in system Files".into(),
                    Action::GameFiles,
                ),
                (
                    "World files",
                    "Import / copy / back up in Files".into(),
                    Action::WorldFiles,
                ),
                ("Profile", format!("Profile {}", c.profile), Action::Profile),
                (
                    "Orientation",
                    c.orientation.label().to_owned(),
                    Action::Orientation,
                ),
                ("Input", c.input_host.label().to_owned(), Action::InputHost),
                (
                    "Immersive fullscreen",
                    if c.screen.immersive { "On" } else { "Off" }.into(),
                    Action::Immersive,
                ),
                (
                    "Avoid camera cutout",
                    if c.screen.avoid_cutout { "On" } else { "Off" }.into(),
                    Action::Cutout,
                ),
                (
                    "Aspect ratio",
                    c.screen.aspect.label().into(),
                    Action::Aspect,
                ),
                (
                    "Display refresh rate",
                    c.device.display_label(),
                    Action::DisplayHz,
                ),
                (
                    "Performance hints",
                    if c.device.performance_hints {
                        "On (ADPF)"
                    } else {
                        "Off"
                    }
                    .into(),
                    Action::PerformanceHints,
                ),
                (
                    "GPU profiling",
                    if c.device.gpu_profile {
                        "On / included in report"
                    } else {
                        "Off"
                    }
                    .into(),
                    Action::GpuProfile,
                ),
                (
                    "Performance recording",
                    if c.record_performance {
                        "On / up to 30 min"
                    } else {
                        "Off"
                    }
                    .into(),
                    Action::RecordPerformance,
                ),
                (
                    "Diagnostic report",
                    "Export one ZIP to share".into(),
                    Action::ExportDiagnostics,
                ),
                (
                    "Raw performance data",
                    "Advanced / JSONL in Reports".into(),
                    Action::ExportPerformance,
                ),
                (
                    "Server",
                    if c.server.is_empty() {
                        "Singleplayer / tap to enter server".into()
                    } else {
                        c.server.clone()
                    },
                    Action::Field("server"),
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let y = 195.0 + i as f32 * row_height - self.catalog.scroll;
                self.list_targets.push((
                    [
                        if compact { 32.0 } else { w * 0.36 },
                        y,
                        if compact { w - 64.0 } else { w * 0.64 - 32.0 },
                        if compact { 70.0 } else { 46.0 },
                    ],
                    action,
                ));
                if y < 185.0 || y + if compact { 70.0 } else { 46.0 } > h - 130.0 {
                    continue;
                }
                ui.text(
                    Face::Regular,
                    16.0,
                    32.0,
                    y + if compact { 0.0 } else { 13.0 },
                    color::MUTED,
                    title,
                );
                if matches!(
                    action,
                    Action::PerformanceHints | Action::Immersive | Action::Cutout
                ) {
                    toggle(
                        &mut ui,
                        &mut self.targets,
                        [
                            if compact { 32.0 } else { w * 0.36 },
                            y + if compact { 24.0 } else { 0.0 },
                            if compact { w - 64.0 } else { w * 0.64 - 32.0 },
                            46.0,
                        ],
                        match action {
                            Action::PerformanceHints => c.device.performance_hints,
                            Action::Immersive => c.screen.immersive,
                            Action::Cutout => c.screen.avoid_cutout,
                            _ => c.device.gpu_profile,
                        },
                        action,
                    );
                } else {
                    button(
                        &mut ui,
                        &mut self.targets,
                        [
                            if compact { 32.0 } else { w * 0.36 },
                            y + if compact { 24.0 } else { 0.0 },
                            if compact { w - 64.0 } else { w * 0.64 - 32.0 },
                            46.0,
                        ],
                        &format!(
                            "{} {}",
                            value,
                            if matches!(
                                action,
                                Action::Renderer
                                    | Action::Aspect
                                    | Action::DisplayHz
                                    | Action::RenderScale
                                    | Action::Profile
                                    | Action::Orientation
                                    | Action::InputHost
                            ) {
                                "v"
                            } else {
                                ""
                            }
                        ),
                        action,
                        false,
                    );
                }
                ui.rounded(
                    32.0,
                    y + if compact { 78.0 } else { 62.0 },
                    w - 64.0,
                    1.0,
                    0.0,
                    color::BORDER,
                );
            }
            button(
                &mut ui,
                &mut self.targets,
                [w * 0.5 + 6.0, h - 114.0, w * 0.5 - 38.0, 42.0],
                "Microsoft / Xbox account",
                Action::Account,
                true,
            );
            button(
                &mut ui,
                &mut self.targets,
                [32.0, h - 114.0, w * 0.5 - 38.0, 42.0],
                "Diagnostics",
                Action::Logs,
                false,
            );
        }
        let status = self.error.as_deref().unwrap_or(&self.catalog.status);
        for (i, line) in status
            .chars()
            .collect::<Vec<_>>()
            .chunks(110)
            .take(2)
            .enumerate()
        {
            ui.text(
                Face::Regular,
                14.0,
                32.0,
                h - 47.0 + i as f32 * 18.0,
                color::MUTED,
                &ui.fit_text(
                    Face::Regular,
                    14.0,
                    &line.iter().collect::<String>(),
                    (w - 210.0).max(40.0),
                ),
            );
        }
        if self.catalog.page != Page::Play {
            button(
                &mut ui,
                &mut self.targets,
                [w - 142.0, h - 52.0, 110.0, 32.0],
                "GitHub",
                Action::Repository,
                false,
            );
        }
        if self.login.visible {
            self.list_targets.clear();
            self.list_view = None;
            self.login.draw(
                &mut ui,
                &mut self.targets,
                &self.catalog.config.name,
                self.catalog.editing == Some("name"),
            );
        }
        if let Some(chooser) = &mut self.chooser {
            chooser.draw(&mut ui, &mut self.targets);
            self.list_targets = chooser.navigation(ui.w);
            self.list_view = Some(chooser.viewport(ui.h));
        }
        if let Some([top, bottom, maximum]) = self.list_view
            && maximum > 0.0
            && bottom > top
        {
            let scroll = self
                .chooser
                .as_ref()
                .map_or(self.catalog.scroll, |c| c.scroll);
            let height = bottom - top;
            let thumb = (height * height / (height + maximum)).max(20.0).min(height);
            let y = top + (height - thumb) * (scroll / maximum).clamp(0.0, 1.0);
            ui.rounded(w - 12.0, top, 4.0, height, 2.0, color::BORDER);
            ui.rounded(w - 12.0, y, 4.0, thumb, 2.0, color::MUTED);
        }
        let caret_visible = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .is_ok_and(|t| t.as_millis() % 1000 < 500);
        if caret_visible && let Some(name) = self.catalog.editing {
            let value = match name {
                "name" => self.catalog.config.name.clone(),
                "server" => self.catalog.config.server.clone(),
                "render_scale" => self.catalog.render_scale_text.clone(),
                _ => format!("Search: {}", self.catalog.query),
            };
            for ([x, y, w, _], action) in &self.targets {
                if *action == Action::Field(name)
                    || (name == "render_scale" && *action == Action::RenderScale)
                {
                    let text = ui.fit_text(Face::Medium, 16.0, &value, w - 24.0);
                    let caret_x = x + 12.0 + ui.measure(Face::Medium, 16.0, &text);
                    ui.rounded(
                        caret_x.min(x + w - 10.0),
                        y + 12.0,
                        1.0,
                        18.0,
                        0.0,
                        color::TEXT,
                    );
                }
            }
        }
        if self.exit_confirmation {
            self.targets.clear();
            self.list_targets.clear();
            self.list_view = None;
            ui.rounded(0.0, 0.0, w, h, 0.0, horizon_client::hud::rgba(0, 0, 0, 160));
            let width = (w - 32.0).min(360.0);
            let x = (w - width) * 0.5;
            let y = (h - 160.0) * 0.5;
            ui.rounded(x, y, width, 160.0, 8.0, color::RAISED);
            ui.text(
                Face::Medium,
                22.0,
                x + 20.0,
                y + 24.0,
                color::TEXT,
                "Exit application?",
            );
            let bw = (width - 52.0) * 0.5;
            button(
                &mut ui,
                &mut self.targets,
                [x + 20.0, y + 92.0, bw, 44.0],
                "Cancel",
                Action::Dismiss,
                false,
            );
            button(
                &mut ui,
                &mut self.targets,
                [x + 32.0 + bw, y + 92.0, bw, 44.0],
                "Exit",
                Action::ConfirmExit,
                false,
            );
        }
        if let Some(action) = self.pending_focus.take() {
            self.focus = self.targets.iter().position(|(_, a)| *a == action);
        }
        if let Some(index) = self.focus
            && let Some(([x, y, w, h], _)) = self.targets.get(index)
        {
            ui.border(*x - 2.0, *y - 2.0, *w + 4.0, *h + 4.0, 2.0, color::TEXT);
            self.cursor = (
                f64::from((x + w / 2.0) * ui.density),
                f64::from((y + h / 2.0) * ui.density),
            );
        }
        if self.mouse_down || self.drag.as_ref().is_some_and(|drag| !drag.moved) {
            let point = [
                self.cursor.0 as f32 / ui.density,
                self.cursor.1 as f32 / ui.density,
            ];
            if let Some(([x, y, w, h], _)) = self.targets.iter().rev().find(|(rect, _)| {
                point[0] >= rect[0]
                    && point[0] <= rect[0] + rect[2]
                    && point[1] >= rect[1]
                    && point[1] <= rect[1] + rect[3]
            }) {
                ui.rounded(
                    *x,
                    *y,
                    *w,
                    *h,
                    2.0,
                    horizon_client::hud::rgba(255, 255, 255, 48),
                );
            }
        }
        if !self.mouse_down
            && self.drag.is_none()
            && let Some((action, time)) = self.feedback
            && time.elapsed() < std::time::Duration::from_millis(160)
            && let Some(([x, y, w, h], _)) = self.targets.iter().find(|(_, a)| *a == action)
        {
            ui.rounded(
                *x,
                *y,
                *w,
                *h,
                2.0,
                horizon_client::hud::rgba(255, 255, 255, 48),
            );
        }
        crate::documents::draw_ui(&mut ui);
        ui.finish();

        if !r.render(&game::loading_frame(&b))? && !b.patches.is_empty() {
            // A suspended/lost surface skips the frame, including its glyph
            // uploads. Rebuild them next time instead of caching invisible text.
            horizon_client::ui_theme::reset_atlas();
        }
        Ok(())
    }
    pub(super) fn activate(&mut self, events: &ActiveEventLoop) -> Result<()> {
        if self.session_worker.is_some() && self.chooser.is_none() && !self.exit_confirmation {
            return Ok(());
        }
        let Some(r) = &self.renderer else {
            return Ok(());
        };
        let mut b = HudBatch::new([r.config.width, r.config.height]);
        let ui = Ui::mobile(&self.gui.as_ref().unwrap().atlas, &mut b, (0.0, 0.0));
        let p = self.focus.and_then(|index| self.targets.get(index)).map_or(
            [
                self.cursor.0 as f32 / ui.density,
                self.cursor.1 as f32 / ui.density,
            ],
            |([x, y, w, h], _)| [x + w / 2.0, y + h / 2.0],
        );
        let action = self
            .targets
            .iter()
            .find(|([x, y, w, h], _)| {
                (*x..*x + *w).contains(&p[0]) && (*y..*y + *h).contains(&p[1])
            })
            .map(|(_, a)| *a);
        if self.catalog.busy() || crate::documents::busy() {
            return Ok(());
        }
        self.feedback = action.map(|action| (action, std::time::Instant::now()));
        self.focus = None;
        if action.is_some() && horizon_client::official_install::ready(&self.home) {
            let audio = self.audio.get_or_insert_with(|| {
                let mut audio = horizon_client::audio::Audio::new(
                    &self.home.join(".local/horizon-client/26.3"),
                );
                let settings = horizon_client::settings::Settings::load(
                    &self
                        .catalog
                        .config
                        .game_home(&self.home)
                        .join("options.txt"),
                );
                audio.master = settings.volumes[0];
                let mut volumes = settings.volumes;
                volumes[0] = 1.0;
                horizon_client::settings::set_source_volumes(volumes);
                audio
            });
            audio.play_button_click();
        }
        match action {
            Some(Action::ConfirmExit) => {
                events.exit();
            }
            Some(Action::Immersive) => {
                self.catalog.config.screen.immersive ^= true;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
            }
            Some(Action::Cutout) => {
                self.catalog.config.screen.avoid_cutout ^= true;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
            }
            Some(Action::Aspect) => self.open_chooser(chooser::Kind::Aspect),
            Some(Action::ChooseAspect(aspect)) => {
                self.catalog.config.screen.aspect = aspect;
                self.catalog.config.save(&self.home)?;
                self.configure_screen();
                self.chooser = None;
            }
            Some(Action::CacheBudget) => self.open_chooser(chooser::Kind::CacheBudget),
            Some(Action::ChooseCacheBudget(mib)) => {
                self.catalog.config.cache_budget_mib = mib.min(1024);
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                self.catalog.status = "Disposable cache budget saved. Applies on next Play; not a total RAM or VRAM limit.".into();
            }
            Some(Action::ToggleBundled(id)) => {
                let game = self.catalog.config.game_home(&self.home);
                let mut list = horizon_client::mod_list::ModList::load(&game);
                let enabled = list.enabled.contains(id) && !list.disabled.contains(id);
                if enabled {
                    list.enabled.remove(id);
                    list.disabled.insert(id.into());
                } else {
                    list.disabled.remove(id);
                    list.enabled.insert(id.into());
                }
                std::fs::create_dir_all(&game)?;
                std::fs::write(game.join(horizon_client::mod_list::FILE), list.serialize())?;
                self.catalog.refresh(&self.home);
                self.catalog.status = format!(
                    "Bundled {id} {}. Applies on Play; settings are in Options > Mods.",
                    if enabled { "disabled" } else { "enabled" }
                );
            }
            Some(Action::GameFiles) => {
                self.catalog.status = if crate::platform::open_documents(crate::document_store::ROOT) {
                    "Game files opened. Worlds, mods, packs, shaders and reports are available here."
                } else { "Open system Files > Nebula Launcher." }.into();
            }
            Some(Action::WorldFiles) => {
                let id = crate::document_store::folder_id(self.catalog.config.profile, "worlds");
                self.catalog.status = if crate::platform::open_documents(&id) {
                    "Copy a complete world folder to back up or import. Finish copying before Play; never replace an existing world silently."
                } else { "Open system Files > Nebula Launcher > Worlds. Copy complete world folders while the game is closed." }.into();
            }
            Some(Action::WorkerLimit) => self.open_chooser(chooser::Kind::WorkerLimit),
            Some(Action::ChooseWorkerLimit(limit)) => {
                self.catalog.config.worker_limit = limit.min(64);
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                self.catalog.status = "Chunk worker ceiling saved. Applies on the next Play; other engine threads are unchanged.".into();
            }
            Some(Action::DisplayHz) => self.open_chooser(chooser::Kind::DisplayHz),
            Some(Action::ChooseDisplayHz(hz)) => {
                self.catalog.config.device.display_hz = hz;
                self.catalog.config.device.sanitize();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
                self.chooser = None;
            }
            Some(Action::PerformanceHints) => {
                self.catalog.config.device.performance_hints ^= true;
                self.catalog.status="ADPF gives Android CPU frame-work hints on supported devices. It does not cap FPS or guarantee a speedup.".into();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
            }
            Some(Action::GpuProfile) => {
                self.catalog.config.device.gpu_profile ^= true;
                self.catalog.status="Detailed GPU profiling records diagnostics and has overhead. Reports remain on this device.".into();
                self.catalog.config.save(&self.home)?;
                crate::device_runtime::configure(self.catalog.config.device);
            }

            Some(Action::ModSource(origin)) => {
                self.catalog.config.mod_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::ShaderSource(origin)) => {
                self.catalog.config.shader_source = origin;
                self.catalog.config.save(&self.home)?;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::OpenFolder(kind)) => {
                crate::platform::hide();
                let folder = match kind {
                    horizon_client::content_files::Kind::Mods => "mods",
                    horizon_client::content_files::Kind::Packs => "packs",
                    horizon_client::content_files::Kind::Shaders => "shaders",
                };
                let id = crate::document_store::folder_id(self.catalog.config.profile, folder);
                self.catalog.status = if crate::platform::open_documents(&id) {
                    "Opened in Files. Return here after copying files.".into()
                } else {
                    "Open the system Files app and choose Nebula Launcher.".into()
                };
            }
            Some(Action::Import(kind)) => {
                crate::platform::hide();
                let home = self.catalog.config.game_home(&self.home);
                if let Err(e) = crate::documents::begin(kind, home) {
                    self.catalog.status = e;
                }
            }
            Some(Action::ToggleSection(i)) => {
                self.catalog.sections[i] = !self.catalog.sections[i];
                self.catalog.scroll = 0.0;
            }
            Some(Action::ToggleLocal(i)) => self.catalog.toggle_local(&self.home, i),
            Some(Action::DeleteLocal(i)) => {
                self.catalog.pending_delete = self.catalog.local.get(i).cloned();
                if let Some(item) = &self.catalog.pending_delete {
                    self.chooser = Some(super::chooser::Chooser::confirm_delete(item.file.clone()));
                }
            }
            Some(Action::ConfirmDeleteLocal) => {
                self.catalog.remove_local(&self.home);
                self.chooser = None;
            }
            Some(Action::Repository) => {
                self.catalog.status = if crate::platform::open_repository() {
                    "Repository opened in browser."
                } else {
                    "No browser available to open the repository."
                }
                .into();
            }
            Some(Action::Account) => {
                self.login.visible = true;
            }
            Some(Action::CloseLogin) => {
                self.login.hide();
                crate::platform::hide();
                self.catalog.editing = None;
            }
            Some(Action::DeviceSignIn) => {
                self.session_ready = false;
                self.prepared_session = None;
                self.login
                    .start_device(&self.catalog.config.microsoft_client_id);
            }
            Some(Action::CopyLoginCode) => self.login.copy_code(),
            Some(Action::CancelLogin) => {
                self.login.close();
                self.login.visible = true;
            }
            Some(Action::ReviewApp) => self.login.review_app(),
            Some(Action::SignIn) => {
                self.session_ready = false;
                self.prepared_session = None;
                self.login.start(&self.catalog.config.microsoft_client_id);
            }
            Some(Action::SignOut) => {
                self.session_ready = false;
                self.prepared_session = None;
                self.login.sign_out();
            }
            Some(Action::LoginLink) => self.login.open_link(),
            Some(Action::Page(page)) => {
                self.catalog.editing = None;
                if crate::platform::active() {
                    crate::platform::hide();
                }
                self.catalog.page = page;
                self.catalog.scroll = 0.0;
                self.error = None;
                self.catalog.offset = 0;
                self.catalog.browse(&self.home);
            }
            Some(Action::Play) => self.action(events)?,
            Some(Action::Search) => {
                crate::platform::hide();
                self.catalog.offset = 0;
                self.catalog.search(&self.home);
            }
            Some(Action::FinishNameEdit) => {
                self.catalog.editing = None;
                crate::platform::hide();
                self.catalog.config.save(&self.home)?;
            }
            Some(Action::Field(name)) => {
                if name == "render_scale" {
                    self.catalog.render_scale_text =
                        self.catalog.config.render_scale.percent().to_string();
                    self.chooser = None;
                }
                self.catalog.editing = Some(name);
                let initial = match name {
                    "name" => &self.catalog.config.name,
                    "server" => &self.catalog.config.server,
                    "render_scale" => &self.catalog.render_scale_text,
                    "client_id" => &self.catalog.config.microsoft_client_id,
                    _ => &self.catalog.query,
                };
                crate::platform::show(initial);
            }
            Some(Action::Install(i)) => self.catalog.install(&self.home, i),
            Some(Action::Next) => {
                self.catalog.offset += 12;
                self.catalog.search(&self.home);
            }
            Some(Action::Previous) => {
                self.catalog.offset = self.catalog.offset.saturating_sub(12);
                self.catalog.search(&self.home);
            }
            Some(Action::Profile) => self.open_chooser(chooser::Kind::Profile),
            Some(Action::Renderer) => self.open_chooser(chooser::Kind::Renderer),
            Some(Action::RenderScale) => self.open_chooser(chooser::Kind::RenderScale),
            Some(Action::ChooseRenderScale(scale)) => {
                self.catalog.config.render_scale = scale;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::display::set_scale(scale);
            }
            Some(Action::Orientation) => self.open_chooser(chooser::Kind::Orientation),
            Some(Action::InputHost) => self.open_chooser(chooser::Kind::InputHost),
            Some(Action::ChooseInputHost(mode)) => {
                self.catalog.config.input_host = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::configure_input_mode(mode);
            }
            Some(Action::ChooseOrientation(mode)) => {
                self.catalog.config.orientation = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
                crate::platform::set_orientation(mode);
            }
            Some(Action::ChooseRenderer(mode)) => {
                self.catalog.config.renderer = mode;
                self.catalog.config.save(&self.home)?;
                self.chooser = None;
            }
            Some(Action::ChooseProfile(profile)) => {
                std::fs::create_dir_all(if profile == 0 {
                    self.home.clone()
                } else {
                    self.home
                        .join("profiles")
                        .join(format!("profile-{profile}"))
                })?;
                self.catalog.config.profile = profile;
                self.catalog.config.save(&self.home)?;
                self.catalog.refresh(&self.home);
                self.chooser = None;
            }
            Some(Action::Dismiss) => {
                self.exit_confirmation = false;
                self.chooser = None;
                if self.session_worker.is_none() {
                    self.session_ready = false;
                    self.prepared_session = None;
                }
            }
            Some(Action::Logs) => {
                self.catalog.status = crate::diagnostics::summary(&self.home);
                if let Ok(error) = std::fs::read_to_string(self.home.join("android-error.txt")) {
                    self.catalog.status = format!(
                        "Last handled error: {}. {}",
                        crate::diagnostics::redact(&error),
                        self.catalog.status
                    );
                }
            }
            Some(Action::ExportDiagnostics) if self.diagnostic_export.is_none() => {
                let home = self.home.clone();
                let (sender, receiver) = mpsc::channel();
                self.diagnostic_export = Some(receiver);
                self.catalog.status =
                    "Preparing local diagnostic ZIP. No upload; review before sharing.".into();
                std::thread::spawn(move || {
                    let result = crate::diagnostics::export(&home).map(|p| {
                        p.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned()
                    });
                    let _ = sender.send(result);
                });
            }
            Some(Action::ExportDiagnostics) => {}
            Some(Action::RecordPerformance) => {
                self.catalog.config.record_performance = !self.catalog.config.record_performance;
                if !self.catalog.config.record_performance {
                    crate::performance::stop();
                }
                self.catalog.config.save(&self.home)?;
                self.catalog.status = if self.catalog.config.record_performance {
                    "Recording starts on Play. Detailed timings add overhead. Return here to save the report."
                } else { "Performance recording off." }.into();
            }
            Some(Action::ExportPerformance) => {
                self.catalog.status = match crate::performance::export(&self.home) {
                    Ok(()) => "Reports opened in Files. Copy or share a report from there.".into(),
                    Err(e) => e,
                };
            }
            None => {}
        }
        Ok(())
    }
    pub(super) fn ui_density(&self) -> f32 {
        let (Some(r), Some(gui)) = (&self.renderer, &self.gui) else {
            return 1.0;
        };
        let mut batch = HudBatch::new([r.config.width, r.config.height]);
        Ui::mobile(&gui.atlas, &mut batch, (0.0, 0.0)).density
    }

    pub(super) fn touch_event(&mut self, events: &ActiveEventLoop, t: Touch) -> Result<()> {
        self.focus = None;
        self.pending_focus = None;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        self.cursor = (t.location.x, t.location.y);
        match t.phase {
            TouchPhase::Started => {
                self.drag = Some(super::gesture::Drag::new(t.location.x, t.location.y));
            }
            TouchPhase::Moved => {
                let density = self.ui_density() as f64;
                if let Some(drag) = &mut self.drag {
                    let delta = drag.update(t.location.x, t.location.y, density);
                    if drag.moved {
                        if let Some(chooser) = &mut self.chooser {
                            chooser.scroll = (chooser.scroll + delta).max(0.0);
                        } else if self
                            .list_view
                            .is_some_and(|view| drag.origin_y() / density >= view[0] as f64)
                        {
                            self.catalog.scroll = (self.catalog.scroll + delta).max(0.0);
                        }
                    }
                }
            }
            TouchPhase::Ended => {
                if self.drag.take().is_some_and(|drag| !drag.moved) {
                    self.activate(events)?;
                }
            }
            TouchPhase::Cancelled => {
                self.drag = None;
            }
        }
        Ok(())
    }
    pub(super) fn type_key(&mut self, key: KeyEvent) {
        if key.physical_key == PhysicalKey::Code(KeyCode::Escape)
            || key.logical_key
                == winit::keyboard::Key::Named(winit::keyboard::NamedKey::BrowserBack)
        {
            if !key.repeat {
                self.request_back();
            }
            return;
        }
        if self.catalog.editing.is_none() {
            return;
        }
        let out = match key.physical_key {
            PhysicalKey::Code(KeyCode::Backspace) => Some(Output::Backspace),
            PhysicalKey::Code(KeyCode::Enter) => {
                crate::platform::hide();
                Some(Output::Enter)
            }
            _ => key.text.map(|s| Output::Text(s.into())),
        };
        if let Some(out) = out {
            self.catalog.edit(out, &self.home);
        }
    }
}
