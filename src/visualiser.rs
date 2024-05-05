use eframe::{egui, App, Frame};
use egui::{CentralPanel, Context, RichText};

use crate::tokeniser::Tokeniser;



#[derive(Default)]
pub struct TokenVisualiser {
    pub text: String,
    pub tokeniser: Tokeniser,
    pub last_text: String, // To store the last state of the text
    pub tokenised_text: Vec<String>, // To store the tokenised text
}

impl TokenVisualiser {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            text,
            tokeniser,
            last_text,
            tokenised_text,
        } = self;

        let output = egui::TextEdit::multiline(text)
            .hint_text("Type something!")
            .show(ui);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Selected text: ");
            if let Some(text_cursor_range) = output.cursor_range {
                let selected_text = text_cursor_range.slice_str(text);
                ui.code(selected_text);
            }
        });

        let anything_selected = output
            .cursor_range
            .map_or(false, |cursor| !cursor.is_empty());

        ui.add_enabled(
            anything_selected,
            egui::Label::new("Press ctrl+Y to toggle the case of selected text (cmd+Y on Mac)"),
        );

        if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)) {
            if let Some(text_cursor_range) = output.cursor_range {
                use egui::TextBuffer as _;
                let selected_chars = text_cursor_range.as_sorted_char_range();
                let selected_text = text.char_range(selected_chars.clone());
                let upper_case = selected_text.to_uppercase();
                let new_text = if selected_text == upper_case {
                    selected_text.to_lowercase()
                } else {
                    upper_case
                };
                text.delete_char_range(selected_chars.clone());
                text.insert_text(&new_text, selected_chars.start);
            }
        }

        ui.horizontal(|ui| {
            ui.label("Move cursor to the:");

            if ui.button("start").clicked() {
                let text_edit_id = output.response.id;
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                    let ccursor = egui::text::CCursor::new(0);
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                    state.store(ui.ctx(), text_edit_id);
                    ui.ctx().memory_mut(|mem| mem.request_focus(text_edit_id));
                }
            }

            if ui.button("end").clicked() {
                let text_edit_id = output.response.id;
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                    let ccursor = egui::text::CCursor::new(text.chars().count());
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                    state.store(ui.ctx(), text_edit_id);
                    ui.ctx().memory_mut(|mem| mem.request_focus(text_edit_id));
                }
            }
        });

        // Check if the text has changed
        if &*last_text != text {
            *last_text = text.clone(); // Update last_text
            *tokenised_text = tokeniser.tokenize(self.text.clone()); // Update tokenised text
        }

        // Always display the tokenised text with background highlight
        ui.label("Tokenised text:");
        ui.horizontal(|ui| {
            let font_size = 26.0; // Adjust the font size to your preference
            for token in tokenised_text {
                let color = generate_color_for_token(&token);
                ui.label(RichText::new(token.clone())
                    .size(font_size)
                    .background_color(color)
                );
            }
        });
    }
}

struct MyApp {
    text_edit_demo: TokenVisualiser,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            text_edit_demo: TokenVisualiser {
                tokeniser: Tokeniser::new().unwrap(), // Initialize the Tokeniser instance here
                ..Default::default()
            },
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.text_edit_demo.ui(ui);
        });
    }
}

pub fn run() {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    let _ = eframe::run_native("Simple Text Editor", options, Box::new(|_cc| Box::new(MyApp::default())));
}



// Generate a deterministic color from a string.
fn generate_color_for_token(token: &str) -> egui::Color32 {
    let mut hash: u32 = 0;
    for byte in token.bytes() {
        hash = hash.wrapping_mul(37_u32).wrapping_add(u32::from(byte));
    }
    let r = ((hash & 0xFF0000) >> 16) as u8;
    let g = ((hash & 0x00FF00) >> 8) as u8;
    let b = (hash & 0x0000FF) as u8;

    // Convert to HSL and ensure lightness is above a certain threshold
    let (h, s, mut l) = rgb_to_hsl(r, g, b);
    if l < 0.7 {
        l = 0.7;
    }
    let (nr, ng, nb) = hsl_to_rgb(h, s, l);

    egui::Color32::from_rgb(nr, ng, nb)
}

// the following was mostly stolen

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    let mut h = 0.0;
    let mut s = 0.0;

    if max != min {
        let d = max - min;
        s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        h = match max {
            _ if max == r => (g - b) / d + if g < b { 6.0 } else { 0.0 },
            _ if max == g => (b - r) / d + 2.0,
            _ => (r - g) / d + 4.0,
        };
        h /= 6.0;
    }

    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let r;
    let g;
    let b;

    if s == 0.0 {
        r = l;
        g = l;
        b = l;
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        g = hue_to_rgb(p, q, h);
        b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    }

    (
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    } else if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}