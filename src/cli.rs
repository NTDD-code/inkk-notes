//! Command Line Interface (CLI) for Inkk & MathEngine.
//! Enables AI agents, automation pipelines, and headless terminal users to evaluate
//! MATLAB code, generate math entities, inspect notebooks, and manipulate pages without a GUI.

use crate::math_engine::{MathValue, MathWorkspace};
use crate::model::{MatrixEntity, Notebook, PlotEntity, TableGridEntity};
use std::fs;
use std::path::Path;

pub fn run_cli(args: &[String]) {
    if args.len() <= 1 || args[1] == "--help" || args[1] == "-h" || args[1] == "help" {
        print_help();
        return;
    }

    let command = args[1].as_str();
    match command {
        "eval" => {
            if args.len() < 3 {
                eprintln!("Error: 'eval' requires a MATLAB expression in quotes.");
                eprintln!("Usage: inkk eval \"A = [1 2; 3 4]; det(A)\" [--json]");
                std::process::exit(1);
            }
            let code = &args[2];
            let is_json = args.iter().any(|a| a == "--json");
            eval_expression(code, is_json);
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: 'run' requires a path to a .m script file.");
                eprintln!("Usage: inkk run script.m [--json]");
                std::process::exit(1);
            }
            let file_path = &args[2];
            let is_json = args.iter().any(|a| a == "--json");
            run_script_file(file_path, is_json);
        }
        "matrix" => {
            if args.len() < 3 {
                eprintln!("Error: 'matrix' requires MATLAB matrix syntax in quotes.");
                eprintln!("Usage: inkk matrix \"[1 2 3; 4 5 6]\" [--out file.json]");
                std::process::exit(1);
            }
            let code = &args[2];
            let out_file = args
                .iter()
                .position(|a| a == "--out")
                .and_then(|idx| args.get(idx + 1));
            generate_matrix_cli(code, out_file.map(|s| s.as_str()));
        }
        "axes" => {
            if args.len() < 3 {
                eprintln!("Error: 'axes' requires dimension (1D, 2D, or 3D) and bounds.");
                eprintln!("Usage: inkk axes 1D <xmin> <xmax> [xstep]");
                eprintln!("       inkk axes 2D <xmin> <xmax> <ymin> <ymax> [xstep] [ystep]");
                eprintln!("       inkk axes 3D <xmin> <xmax> <ymin> <ymax> <zmin> <zmax>");
                std::process::exit(1);
            }
            generate_axes_cli(&args[2..]);
        }
        "table" => {
            if args.len() < 4 {
                eprintln!("Error: 'table' requires <rows> and <cols>.");
                eprintln!("Usage: inkk table <rows> <cols> [csv_data]");
                std::process::exit(1);
            }
            let rows: usize = args[2].parse().unwrap_or(3);
            let cols: usize = args[3].parse().unwrap_or(3);
            let mut csv_data: Option<&str> = None;
            for i in 4..args.len() {
                if (args[i] == "--csv" || args[i] == "-csv") && i + 1 < args.len() {
                    csv_data = Some(&args[i + 1]);
                    break;
                }
            }
            if csv_data.is_none() && args.len() > 4 && !args[4].starts_with('-') {
                csv_data = Some(&args[4]);
            }
            generate_table_cli(rows, cols, csv_data);
        }
        "info" => {
            if args.len() < 3 {
                eprintln!("Error: 'info' requires a path to a .handnote file.");
                eprintln!("Usage: inkk info notebook.handnote");
                std::process::exit(1);
            }
            inspect_notebook(&args[2]);
        }
        "new" => {
            if args.len() < 3 {
                eprintln!("Error: 'new' requires a target filename.");
                eprintln!("Usage: inkk new my_notes.handnote");
                std::process::exit(1);
            }
            create_new_notebook(&args[2]);
        }
        other => {
            eprintln!("Unknown command: '{}'. Use 'inkk --help' for usage.", other);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"
==============================================================================
   Inkk - Handwritten Notes & MATLAB Math Engine CLI (Headless Interface)
==============================================================================

Designed for AI agent orchestration, automated math evaluation, and headless scripting.

USAGE:
    inkk                           Launch interactive GUI
    inkk <COMMAND> [OPTIONS]       Execute headless CLI command

COMMANDS:
    eval "<matlab code>" [--json]
        Execute MATLAB expressions or scripts directly in terminal.
        Example: inkk eval "A = [1 2; 3 4]; inv(A)"
                 inkk eval "x = 1:5; y = x.^2; plot(x, y)" --json

    run <file.m> [--json]
        Run a MATLAB script file from disk and print output or commands.
        Example: inkk run my_calc.m

    matrix "<matlab matrix>" [--out file.json]
        Generate a MatrixEntity from MATLAB syntax.
        Example: inkk matrix "[1 2 3; 4 5 6; 7 8 9]"

    axes <1D|2D|3D> [args...]
        Generate Coordinate Axes (1D number line, 2D Oxy, or 3D Oxyz).
        Example: inkk axes 1D -10 10 2
                 inkk axes 2D -10 10 -5 5 1 1
                 inkk axes 3D -5 5 -5 5 -5 5

    table <rows> <cols> [csv_data]
        Generate a structured TableGridEntity.
        Example: inkk table 3 3 "STT, Name, Score\n1, Alice, 95\n2, Bob, 88"

    info <notebook.handnote>
        Inspect a notebook file: page count, stroke count, images, and math items.

    new <notebook.handnote>
        Create a new blank notebook ready for hand-writing or scripted insertions.

    --help, -h
        Show this help message.
"#
    );
}

fn eval_expression(code: &str, is_json: bool) {
    let mut ws = MathWorkspace::new();
    let output = ws.execute(code);

    if is_json {
        let vars_json: Vec<serde_json::Value> = ws
            .variables
            .iter()
            .map(|(k, v)| {
                serde_json::json!({
                    "name": k,
                    "type": match v {
                        MathValue::Matrix { .. } => "matrix",
                        MathValue::Str(_) => "string",
                        MathValue::AnonymousFn { .. } => "function",
                    },
                    "display": v.to_display_string(),
                })
            })
            .collect();

        let cmds_json: Vec<String> = output
            .canvas_commands
            .iter()
            .map(|c| format!("{:?}", c))
            .collect();

        let res = serde_json::json!({
            "text_output": output.text_output,
            "error": output.error,
            "variables": vars_json,
            "canvas_commands": cmds_json,
        });

        println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
    } else {
        if !output.text_output.is_empty() {
            println!("{}", output.text_output);
        }
        if let Some(ref err) = output.error {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
        if !output.canvas_commands.is_empty() {
            println!(
                "\n[Canvas Commands Generated: {}]",
                output.canvas_commands.len()
            );
            for (idx, cmd) in output.canvas_commands.iter().enumerate() {
                println!("  {}. {:?}", idx + 1, cmd);
            }
        }
    }
}

fn run_script_file(file_path: &str, is_json: bool) {
    match fs::read_to_string(file_path) {
        Ok(code) => eval_expression(&code, is_json),
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    }
}

fn generate_matrix_cli(code: &str, out_file: Option<&str>) {
    let mut ws = MathWorkspace::new();
    let output = ws.execute(code);

    if let Some(err) = output.error {
        eprintln!("Error evaluating matrix: {}", err);
        std::process::exit(1);
    }

    let builtins = ["pi", "e", "Inf", "NaN", "eps"];
    let mut found_matrix: Option<(Vec<Vec<f64>>, String)> = None;

    // First check user-defined variable (not in builtins, not ans)
    for (name, val) in &ws.variables {
        if !builtins.contains(&name.as_str()) && name != "ans" {
            if let MathValue::Matrix { .. } = val {
                found_matrix = Some((val.to_2d_vec(), format!("{} =", name)));
                break;
            }
        }
    }

    // If no user named variable, check ans
    if found_matrix.is_none() {
        if let Some(ans_val) = ws.variables.get("ans") {
            if let MathValue::Matrix { .. } = ans_val {
                found_matrix = Some((ans_val.to_2d_vec(), String::new()));
            }
        }
    }

    let (data, label) = match found_matrix {
        Some(m) => m,
        None => {
            eprintln!("Error: Provided code did not evaluate to a matrix.");
            std::process::exit(1);
        }
    };

    let rows = data.len();
    let cols = data.first().map(|r| r.len()).unwrap_or(1);
    let matrix_entity = MatrixEntity {
        id: format!(
            "mat_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ),
        x: 100.0,
        y: 120.0,
        width: (cols as f32 * 56.0 + 80.0).max(180.0),
        height: (rows as f32 * 38.0 + 36.0).max(90.0),
        label,
        data,
        source_code: Some(code.to_string()),
    };

    let json_str = serde_json::to_string_pretty(&matrix_entity).unwrap_or_default();
    if let Some(path) = out_file {
        if let Err(e) = fs::write(path, &json_str) {
            eprintln!("Error saving matrix to '{}': {}", path, e);
            std::process::exit(1);
        }
        println!("Matrix saved to '{}'", path);
    } else {
        println!("{}", json_str);
    }
}

fn parse_flag_val(args: &[String], flags: &[&str]) -> Option<f64> {
    for i in 0..args.len() {
        if flags.contains(&args[i].as_str()) && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<f64>() {
                return Some(val);
            }
        }
    }
    None
}

fn get_pos_args(args: &[String]) -> Vec<f64> {
    let mut pos = Vec::new();
    let mut skip_next = false;
    for (i, a) in args.iter().enumerate() {
        if i == 0 {
            continue;
        } // dim
        if skip_next {
            skip_next = false;
            continue;
        }
        if a.starts_with("--")
            || (a.starts_with('-')
                && !a
                    .chars()
                    .nth(1)
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false))
        {
            skip_next = true;
            continue;
        }
        if let Ok(v) = a.parse::<f64>() {
            pos.push(v);
        }
    }
    pos
}

fn generate_axes_cli(args: &[String]) {
    let dim = args[0].to_uppercase();
    let mut plot = PlotEntity::default();
    plot.dimension = Some(dim.clone());

    let pos = get_pos_args(args);
    let global_min = parse_flag_val(args, &["--min", "-min"]);
    let global_max = parse_flag_val(args, &["--max", "-max"]);
    let global_step = parse_flag_val(args, &["--step", "-step"]);

    match dim.as_str() {
        "1D" => {
            let xmin = parse_flag_val(args, &["--xmin"])
                .or(global_min)
                .or_else(|| pos.get(0).copied())
                .unwrap_or(-10.0);
            let xmax = parse_flag_val(args, &["--xmax"])
                .or(global_max)
                .or_else(|| pos.get(1).copied())
                .unwrap_or(10.0);
            let xstep = parse_flag_val(args, &["--xstep", "--step"])
                .or(global_step)
                .or_else(|| pos.get(2).copied())
                .unwrap_or(2.0);
            plot.x_min = xmin;
            plot.x_max = xmax;
            plot.x_step = Some(xstep);
            plot.width = 520.0;
            plot.height = 130.0;
            plot.title = "1D Number Line".into();
        }
        "3D" => {
            let xmin = parse_flag_val(args, &["--xmin"])
                .or(global_min)
                .or_else(|| pos.get(0).copied())
                .unwrap_or(-5.0);
            let xmax = parse_flag_val(args, &["--xmax"])
                .or(global_max)
                .or_else(|| pos.get(1).copied())
                .unwrap_or(5.0);
            let ymin = parse_flag_val(args, &["--ymin"])
                .or(global_min)
                .or_else(|| pos.get(2).copied())
                .unwrap_or(-5.0);
            let ymax = parse_flag_val(args, &["--ymax"])
                .or(global_max)
                .or_else(|| pos.get(3).copied())
                .unwrap_or(5.0);
            let zmin = parse_flag_val(args, &["--zmin"])
                .or(global_min)
                .or_else(|| pos.get(4).copied())
                .unwrap_or(-5.0);
            let zmax = parse_flag_val(args, &["--zmax"])
                .or(global_max)
                .or_else(|| pos.get(5).copied())
                .unwrap_or(5.0);
            let zstep = parse_flag_val(args, &["--zstep", "--step"])
                .or(global_step)
                .or_else(|| pos.get(6).copied())
                .unwrap_or(2.0);
            plot.x_min = xmin;
            plot.x_max = xmax;
            plot.y_min = ymin;
            plot.y_max = ymax;
            plot.z_min = Some(zmin);
            plot.z_max = Some(zmax);
            plot.x_step = global_step.or_else(|| pos.get(6).copied());
            plot.y_step = global_step.or_else(|| pos.get(6).copied());
            plot.z_step = Some(zstep);
            plot.width = 480.0;
            plot.height = 360.0;
            plot.title = "3D Coordinate Space (Oxyz)".into();
        }
        _ => {
            // 2D Oxy default
            let xmin = parse_flag_val(args, &["--xmin"])
                .or(global_min)
                .or_else(|| pos.get(0).copied())
                .unwrap_or(-10.0);
            let xmax = parse_flag_val(args, &["--xmax"])
                .or(global_max)
                .or_else(|| pos.get(1).copied())
                .unwrap_or(10.0);
            let ymin = parse_flag_val(args, &["--ymin"])
                .or(global_min)
                .or_else(|| pos.get(2).copied())
                .unwrap_or(-5.0);
            let ymax = parse_flag_val(args, &["--ymax"])
                .or(global_max)
                .or_else(|| pos.get(3).copied())
                .unwrap_or(5.0);
            let xstep = parse_flag_val(args, &["--xstep"])
                .or(global_step)
                .or_else(|| pos.get(4).copied());
            let ystep = parse_flag_val(args, &["--ystep"])
                .or(global_step)
                .or_else(|| pos.get(5).copied());
            plot.x_min = xmin;
            plot.x_max = xmax;
            plot.y_min = ymin;
            plot.y_max = ymax;
            plot.x_step = xstep;
            plot.y_step = ystep;
            plot.width = 480.0;
            plot.height = 320.0;
            plot.title = "2D Cartesian Axes (Oxy)".into();
        }
    }

    let json_str = serde_json::to_string_pretty(&plot).unwrap_or_default();
    println!("{}", json_str);
}

fn generate_table_cli(rows: usize, cols: usize, csv_data: Option<&str>) {
    let mut cells = vec![vec!["".to_string(); cols]; rows];
    if let Some(csv) = csv_data {
        for (r_idx, line) in csv.lines().enumerate() {
            if r_idx >= rows {
                break;
            }
            for (c_idx, part) in line.split(',').enumerate() {
                if c_idx >= cols {
                    break;
                }
                cells[r_idx][c_idx] = part.trim().to_string();
            }
        }
    }

    let grid_entity = TableGridEntity {
        id: format!(
            "grid_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ),
        x: 80.0,
        y: 120.0,
        rows,
        cols,
        col_width: 110.0,
        row_height: 36.0,
        cells,
        source_code: csv_data.map(|s| s.to_string()),
    };

    let json_str = serde_json::to_string_pretty(&grid_entity).unwrap_or_default();
    println!("{}", json_str);
}

fn inspect_notebook(path: &str) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading notebook '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let nb: Notebook = match serde_json::from_str(&content) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error parsing notebook JSON: {}", e);
            std::process::exit(1);
        }
    };

    let total_strokes: usize = nb.pages.iter().map(|p| p.strokes.len()).sum();
    let total_images: usize = nb.pages.iter().map(|p| p.images.len()).sum();
    let total_plots: usize = nb.pages.iter().map(|p| p.plots.len()).sum();
    let total_matrices: usize = nb.pages.iter().map(|p| p.matrices.len()).sum();
    let total_grids: usize = nb.pages.iter().map(|p| p.grids.len()).sum();

    println!("============================================================");
    println!("Notebook: \"{}\"", nb.title);
    println!(
        "File:     {}",
        Path::new(path)
            .canonicalize()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.to_string())
    );
    println!("Pages:    {}", nb.pages.len());
    println!("------------------------------------------------------------");
    println!("Strokes:  {} total", total_strokes);
    println!("Images:   {} total", total_images);
    println!("Plots:    {} total", total_plots);
    println!("Matrices: {} total", total_matrices);
    println!("Tables:   {} total", total_grids);
    println!("============================================================");
}

fn create_new_notebook(path: &str) {
    let mut nb = Notebook::default();
    nb.title = "New Notebook".to_string();

    let json_str = serde_json::to_string_pretty(&nb).unwrap_or_default();
    if let Err(e) = fs::write(path, &json_str) {
        eprintln!("Error writing new notebook to '{}': {}", path, e);
        std::process::exit(1);
    }
    println!("Created new notebook at '{}'", path);
}
