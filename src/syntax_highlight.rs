//! Syntax highlighting for MATLAB/MathStudio code
use crate::theme::Palette;
use eframe::egui::{self, Color32};

pub fn syntax_highlight_matlab(
    text: &str,
    _ui: &egui::Ui,
    palette: &Palette,
    is_dark: bool,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();

    // Define color scheme
    let keyword_color = if is_dark {
        Color32::from_rgb(207, 142, 255) // Purple
    } else {
        Color32::from_rgb(175, 0, 219)
    };

    let function_color = if is_dark {
        Color32::from_rgb(100, 180, 255) // Light blue
    } else {
        Color32::from_rgb(0, 92, 197)
    };

    let number_color = if is_dark {
        Color32::from_rgb(181, 206, 168) // Light green
    } else {
        Color32::from_rgb(9, 134, 88)
    };

    let string_color = if is_dark {
        Color32::from_rgb(206, 145, 120) // Orange
    } else {
        Color32::from_rgb(163, 21, 21)
    };

    let comment_color = palette.text_muted;
    let operator_color = if is_dark {
        Color32::from_rgb(212, 212, 212)
    } else {
        Color32::from_rgb(80, 80, 80)
    };

    // MATLAB keywords
    let keywords = [
        "if",
        "else",
        "elseif",
        "end",
        "for",
        "while",
        "function",
        "return",
        "break",
        "continue",
        "switch",
        "case",
        "otherwise",
        "try",
        "catch",
        "hold",
        "on",
        "off",
        "clear",
        "clc",
        "clf",
        "close",
        "grid",
    ];

    // Built-in functions
    let builtin_funcs = [
        "plot",
        "scatter",
        "bar",
        "hist",
        "pie",
        "surf",
        "mesh",
        "sin",
        "cos",
        "tan",
        "exp",
        "log",
        "sqrt",
        "abs",
        "floor",
        "ceil",
        "round",
        "linspace",
        "logspace",
        "zeros",
        "ones",
        "eye",
        "rand",
        "randn",
        "sum",
        "mean",
        "median",
        "std",
        "var",
        "min",
        "max",
        "sort",
        "length",
        "size",
        "numel",
        "reshape",
        "transpose",
        "xlabel",
        "ylabel",
        "title",
        "legend",
        "xlim",
        "ylim",
        "axis",
        "disp",
        "fprintf",
        "sprintf",
        "input",
        "axes",
        "grid",
        "table",
        "matrix",
    ];

    let mut chars = text.char_indices().peekable();
    let default_format = egui::TextFormat {
        font_id: egui::FontId::monospace(13.0),
        color: palette.text,
        ..Default::default()
    };

    while let Some((i, c)) = chars.next() {
        // Comments
        if c == '%' {
            let start = i;
            let mut end = text.len();
            for (j, ch) in chars.by_ref() {
                if ch == '\n' {
                    end = j + 1;
                    break;
                }
            }
            job.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: start..end,
                format: egui::TextFormat {
                    color: comment_color,
                    ..default_format.clone()
                },
            });
            continue;
        }

        // String literals
        if c == '"' {
            let start = i;
            let mut end = text.len();
            let mut escaped = false;
            for (j, ch) in chars.by_ref() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    end = j + 1;
                    break;
                }
            }
            job.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: start..end,
                format: egui::TextFormat {
                    color: string_color,
                    ..default_format.clone()
                },
            });
            continue;
        }

        // Numbers
        if c.is_ascii_digit()
            || (c == '.' && chars.peek().map_or(false, |(_, ch)| ch.is_ascii_digit()))
        {
            let start = i;
            let mut end = i + c.len_utf8();
            let mut has_dot = c == '.';

            while let Some(&(j, ch)) = chars.peek() {
                if ch.is_ascii_digit() || (ch == '.' && !has_dot) || ch == 'e' || ch == 'E' {
                    if ch == '.' {
                        has_dot = true;
                    }
                    end = j + ch.len_utf8();
                    chars.next();
                } else if (ch == '+' || ch == '-') && text[..j].ends_with(|c| c == 'e' || c == 'E')
                {
                    end = j + ch.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }

            job.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: start..end,
                format: egui::TextFormat {
                    color: number_color,
                    ..default_format.clone()
                },
            });
            continue;
        }

        // Operators
        if "+-*/=<>!&|~^()[]{}.,;:".contains(c) {
            job.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: i..(i + c.len_utf8()),
                format: egui::TextFormat {
                    color: operator_color,
                    ..default_format.clone()
                },
            });
            continue;
        }

        // Identifiers
        if c.is_alphabetic() || c == '_' {
            let start = i;
            let mut end = i + c.len_utf8();

            while let Some(&(j, ch)) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    end = j + ch.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }

            let word = &text[start..end];
            let color = if keywords.contains(&word) {
                keyword_color
            } else if builtin_funcs.contains(&word) {
                function_color
            } else {
                palette.text
            };

            job.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: start..end,
                format: egui::TextFormat {
                    color,
                    ..default_format.clone()
                },
            });
            continue;
        }

        // Default
        job.sections.push(egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: i..(i + c.len_utf8()),
            format: default_format.clone(),
        });
    }

    if job.sections.is_empty() {
        job.sections.push(egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: 0..0,
            format: default_format,
        });
    }

    job.text = text.to_string();
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlight_matlab_layout() {
        let palette = Palette::get(false);
        let ctx = egui::Context::default();
        let test_scripts = [
            "x = linspace(-2*pi, 2*pi, 200);\ny = sin(x) .* exp(-0.15*x);\nplot(x, y, \"DisplayName\", \"Damped Wave\")",
            "",
            "% comment only",
            "% comment line 1\nx = 10;",
            "x = \"unclosed string",
            "A = [1 2; 3 4]; % test matrix\nB = A * 2;",
        ];

        for script in test_scripts {
            let _ = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let job = syntax_highlight_matlab(script, ui, &palette, false);
                    assert_eq!(job.text, script);
                    let _galley = ui.fonts(|f| f.layout_job(job));
                });
            });
        }
    }
}
