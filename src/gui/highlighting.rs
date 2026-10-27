use eframe::egui;

use crate::assembler::keywords::{CONDITIONS, DIRECTIVES, MNEMONICS, REGISTERS};

use super::theme::AppTheme;

pub fn highlight(
    theme: &AppTheme,
    code: &str,
    highlight_line: Option<usize>,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let font_id = egui::FontId::monospace(AppTheme::MONO_FONT_SIZE);

    let lines: Vec<&str> = code.split('\n').collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let line = *line;
        let current_line_num = line_idx + 1;
        let background = if highlight_line == Some(current_line_num) {
            theme.pc_row
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
            let clean = upper.trim_end_matches(',');

            if DIRECTIVES.contains(&clean) {
                theme.syntax.directive
            } else if MNEMONICS.contains(&clean) {
                theme.syntax.mnemonic
            } else if REGISTERS.contains(&clean) {
                theme.syntax.register
            } else if CONDITIONS.contains(&clean) {
                theme.syntax.condition
            } else if clean.ends_with(':') {
                theme.syntax.label
            } else if clean.starts_with('"') || clean.starts_with('\'') {
                theme.syntax.string
            } else {
                if clean.chars().next().is_some_and(|c| c.is_ascii_digit())
                    || clean.starts_with('$')
                    || (clean.ends_with('H')
                        && clean.chars().next().is_some_and(|c| c.is_ascii_hexdigit()))
                {
                    theme.syntax.number
                } else {
                    theme.syntax.label
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
                        color: theme.syntax.plain,
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
                    color: theme.syntax.comment,
                    background,
                    ..Default::default()
                },
            );
        }

        if line_idx + 1 < lines.len() {
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
    }

    job
}

#[derive(Default)]
pub struct HighlightCache {
    code: String,
    highlight_line: Option<usize>,
    job: Option<egui::text::LayoutJob>,
    #[cfg(test)]
    builds: usize,
}

impl HighlightCache {
    pub fn galley(
        &mut self,
        ui: &egui::Ui,
        theme: &AppTheme,
        code: &str,
        highlight_line: Option<usize>,
    ) -> std::sync::Arc<egui::Galley> {
        let stale =
            self.job.is_none() || self.code != code || self.highlight_line != highlight_line;
        if stale {
            self.job = Some(highlight(theme, code, highlight_line));
            self.code.clear();
            self.code.push_str(code);
            self.highlight_line = highlight_line;
            #[cfg(test)]
            {
                self.builds += 1;
            }
        }
        ui.painter()
            .layout_job(self.job.as_ref().expect("populated above").clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{AppTheme, HighlightCache, highlight};
    use eframe::egui;

    fn section_text<'a>(
        job: &'a egui::text::LayoutJob,
        section: &egui::text::LayoutSection,
    ) -> &'a str {
        &job.text[section.byte_range.start.0..section.byte_range.end.0]
    }

    fn section_with_text<'a>(
        job: &'a egui::text::LayoutJob,
        text: &str,
    ) -> &'a egui::text::LayoutSection {
        job.sections
            .iter()
            .find(|s| section_text(job, s) == text)
            .unwrap_or_else(|| panic!("no section with text {text:?}"))
    }

    #[test]
    fn highlight_preserves_the_buffer_text() {
        let theme = AppTheme::ONE_DARK;
        for code in ["", "NOP", "NOP\n", "A\nB"] {
            let job = highlight(&theme, code, None);
            assert_eq!(job.text, code, "buffer text changed for {code:?}");
        }
    }

    #[test]
    fn highlight_colours_tokens_from_the_theme() {
        let theme = AppTheme::ONE_DARK;
        let job = highlight(&theme, "LD A, 1", None);

        assert_eq!(
            section_with_text(&job, "LD").format.color,
            theme.syntax.mnemonic
        );
        assert_eq!(
            section_with_text(&job, "A").format.color,
            theme.syntax.register
        );
        assert_eq!(
            section_with_text(&job, "1").format.color,
            theme.syntax.number
        );

        let job = highlight(&theme, "NOP ; note", None);
        let comment = job
            .sections
            .iter()
            .find(|s| section_text(&job, s) == "; note")
            .expect("comment section");
        assert_eq!(comment.format.color, theme.syntax.comment);
    }

    #[test]
    fn highlight_tints_only_the_requested_line() {
        let theme = AppTheme::ONE_DARK;
        let job = highlight(&theme, "NOP\nHALT", Some(2));

        let halted = job
            .sections
            .iter()
            .find(|s| section_text(&job, s).contains("HALT"))
            .expect("HALT section");
        assert_eq!(halted.format.background, theme.pc_row);

        let nop = job
            .sections
            .iter()
            .find(|s| section_text(&job, s).contains("NOP"))
            .expect("NOP section");
        assert_eq!(nop.format.background, egui::Color32::TRANSPARENT);
    }

    fn run_frame<R>(ctx: &egui::Context, f: impl FnOnce(&egui::Ui) -> R) -> R {
        let mut result = None;
        let mut f = Some(f);
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let f = f.take().expect("one frame");
            egui::CentralPanel::default().show(ui, |ui| {
                result = Some(f(ui));
            });
        });
        output.drop_without_applying_deltas();
        result.expect("the closure ran")
    }

    #[test]
    fn cache_reuses_the_tokens_when_nothing_changed() {
        let theme = AppTheme::ONE_DARK;
        let mut cache = HighlightCache::default();

        let (first, second) = run_frame(&egui::Context::default(), |ui| {
            let first = cache.galley(ui, &theme, "LD A, 1", None);
            let second = cache.galley(ui, &theme, "LD A, 1", None);
            (first, second)
        });

        assert_eq!(cache.builds, 1);
        assert_eq!(first.text(), "LD A, 1");
        assert_eq!(second.text(), "LD A, 1");
    }

    #[test]
    fn cache_rebuilds_when_the_code_changes() {
        let theme = AppTheme::ONE_DARK;
        let mut cache = HighlightCache::default();

        let (_first, second) = run_frame(&egui::Context::default(), |ui| {
            let first = cache.galley(ui, &theme, "LD A, 1", None);
            let second = cache.galley(ui, &theme, "LD A, 2", None);
            (first, second)
        });

        assert_eq!(cache.builds, 2);
        assert_eq!(second.text(), "LD A, 2");
    }

    #[test]
    fn cache_rebuilds_when_the_highlighted_line_changes() {
        let theme = AppTheme::ONE_DARK;
        let mut cache = HighlightCache::default();

        let (first, second) = run_frame(&egui::Context::default(), |ui| {
            let first = cache.galley(ui, &theme, "LD A, 1", None);
            let second = cache.galley(ui, &theme, "LD A, 1", Some(1));
            (first, second)
        });

        assert_eq!(cache.builds, 2);
        assert_eq!(first.text(), "LD A, 1");
        assert_eq!(second.text(), "LD A, 1");
    }

    #[test]
    fn cache_survives_across_frames() {
        let theme = AppTheme::ONE_DARK;
        let ctx = egui::Context::default();
        let mut cache = HighlightCache::default();

        let first = run_frame(&ctx, |ui| cache.galley(ui, &theme, "LD A, 1", None));
        let second = run_frame(&ctx, |ui| cache.galley(ui, &theme, "LD A, 1", None));

        assert_eq!(cache.builds, 1);
        assert_eq!(first.text(), "LD A, 1");
        assert_eq!(second.text(), "LD A, 1");
    }

    #[test]
    fn cache_job_is_relaid_out_when_pixels_per_point_changes() {
        let theme = AppTheme::ONE_DARK;
        let ctx = egui::Context::default();
        let mut cache = HighlightCache::default();

        let first = run_frame(&ctx, |ui| cache.galley(ui, &theme, "LD A, 1", None));

        ctx.set_pixels_per_point(2.0);
        let second = run_frame(&ctx, |ui| cache.galley(ui, &theme, "LD A, 1", None));

        assert_eq!(cache.builds, 1, "tokenisation should have been reused");
        assert!(
            !std::sync::Arc::ptr_eq(&first, &second),
            "egui should lay the job out again for the new scale"
        );
        assert_eq!(second.text(), "LD A, 1");
    }
}
