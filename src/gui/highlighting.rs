use eframe::egui;

use crate::assembler::keywords::{CONDITIONS, DIRECTIVES, MNEMONICS, REGISTERS};

pub struct CodeTheme {
    pub comment: egui::Color32,
    pub mnemonic: egui::Color32,
    pub register: egui::Color32,
    pub condition: egui::Color32,
    pub number: egui::Color32,
    pub label: egui::Color32,
    pub directive: egui::Color32,
    pub string: egui::Color32,
    pub plain: egui::Color32,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::one_dark_pro_vivid()
    }
}

impl CodeTheme {
    pub fn one_dark_pro_vivid() -> Self {
        Self {
            comment: egui::Color32::from_rgb(127, 132, 142), // #7f848e (Grey)
            mnemonic: egui::Color32::from_rgb(198, 120, 221), // #c678dd (Purple)
            register: egui::Color32::from_rgb(229, 192, 123), // #e5c07b (Gold/Yellow)
            condition: egui::Color32::from_rgb(224, 108, 117), // #e06c75 (Red/Pink)
            number: egui::Color32::from_rgb(209, 154, 102),  // #d19a66 (Orange)
            label: egui::Color32::from_rgb(97, 175, 239),    // #61afef (Blue)
            directive: egui::Color32::from_rgb(86, 182, 194), // #56b6c2 (Cyan)
            string: egui::Color32::from_rgb(152, 195, 121),  // #98c379 (Green)
            plain: egui::Color32::from_rgb(171, 178, 191),   // #abb2bf (White-ish)
        }
    }
}

pub fn highlight(
    _ctx: &egui::Context,
    theme: &CodeTheme,
    code: &str,
    highlight_line: Option<usize>,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let font_id = egui::FontId::monospace(14.0); // Standard mono size

    for (line_idx, line) in code.lines().enumerate() {
        let current_line_num = line_idx + 1;
        let background = if highlight_line == Some(current_line_num) {
            egui::Color32::from_rgba_premultiplied(40, 40, 40, 255) // Dark Gray Highlight
        } else {
            egui::Color32::TRANSPARENT
        };

        let comment_index = line.char_indices().find_map(|(index, character)| {
            if character == ';' {
                let quote_count = line[..index].chars().filter(|&c| c == '"').count();
                (quote_count % 2 == 0).then_some(index)
            } else {
                None
            }
        });
        let (code_part, comment_part) = if let Some(idx) = comment_index {
            (&line[..idx], Some(&line[idx..]))
        } else {
            (line, None)
        };

        let get_color = |token: &str| -> egui::Color32 {
            let upper = token.to_uppercase();
            // Remove trailing commas for checking
            let clean = upper.trim_end_matches(',');

            if DIRECTIVES.contains(&clean) {
                theme.directive
            } else if MNEMONICS.contains(&clean) {
                theme.mnemonic
            } else if REGISTERS.contains(&clean) {
                theme.register
            } else if CONDITIONS.contains(&clean) {
                theme.condition
            } else if clean.ends_with(':') {
                theme.label
            } else if clean.starts_with('"') || clean.starts_with('\'') {
                theme.string
            } else {
                // Check if number
                if clean.chars().next().map_or(false, |c| c.is_digit(10)) || clean.starts_with('$')
                {
                    theme.number
                } else if clean.ends_with('H')
                    && clean
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_ascii_hexdigit())
                {
                    theme.number
                } else {
                    // Fallback: Assume any other identifier is a label reference
                    theme.label
                }
            }
        };

        let mut start_token = None;

        let mut in_string = false;
        let mut string_quote = '\0';

        for (i, c) in code_part.char_indices() {
            if in_string {
                if c == string_quote {
                    in_string = false;
                }
                continue;
            }

            if (c == '"' || c == '\'') && start_token.is_none() {
                start_token = Some(i);
                in_string = true;
                string_quote = c;
                continue;
            }

            let is_delimiter =
                c.is_whitespace() || c == ',' || c == '(' || c == ')' || c == '+' || c == '-';

            if is_delimiter {
                if let Some(start) = start_token {
                    let token = &code_part[start..i];
                    let color = get_color(token);
                    job.append(
                        token,
                        0.0,
                        egui::TextFormat {
                            font_id: font_id.clone(),
                            color,
                            background,
                            ..Default::default()
                        },
                    );
                    start_token = None;
                }
                job.append(
                    &code_part[i..i + c.len_utf8()],
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color: theme.plain, // Delimiters are plain
                        background,
                        ..Default::default()
                    },
                );
            } else {
                if start_token.is_none() {
                    start_token = Some(i);
                }
            }
        }

        if let Some(start) = start_token {
            let token = &code_part[start..];
            let color = get_color(token);
            job.append(
                token,
                0.0,
                egui::TextFormat {
                    font_id: font_id.clone(),
                    color,
                    background,
                    ..Default::default()
                },
            );
        }

        if let Some(comment) = comment_part {
            job.append(
                comment,
                0.0,
                egui::TextFormat {
                    font_id: font_id.clone(),
                    color: theme.comment,
                    background,
                    ..Default::default()
                },
            );
        }

        job.append(
            "\n",
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                background,
                ..Default::default()
            },
        );
    }

    job
}
