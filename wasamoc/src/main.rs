use wasamoc::{ast, check, emit, lexer, lower, parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("check") => match args.get(2) {
            Some(path) => run_check(path),
            None => {
                eprintln!("error: missing file argument");
                eprintln!("usage: wasamoc check <file.ui>");
                std::process::exit(1);
            }
        },
        Some("build") => match args.get(2) {
            Some(path) => {
                let out = args.get(3).cloned();
                run_build(path, out.as_deref());
            }
            None => {
                eprintln!("error: missing file argument");
                eprintln!("usage: wasamoc build <file.ui> [output.uic]");
                std::process::exit(1);
            }
        },
        Some(cmd) => {
            eprintln!("error: unknown command `{}`", cmd);
            eprintln!("usage: wasamoc check <file.ui>");
            eprintln!("       wasamoc build <file.ui> [output.uic]");
            std::process::exit(1);
        }
        None => {
            eprintln!("usage: wasamoc check <file.ui>");
            eprintln!("       wasamoc build <file.ui> [output.uic]");
            std::process::exit(1);
        }
    }
}

fn load_and_check(path: &str) -> (String, check::CheckResult, ast::ComponentDef) {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{}`: {}", path, e);
            std::process::exit(1);
        }
    };

    let tokens = match lexer::tokenize(&src, path) {
        Ok(t) => t,
        Err(diag) => {
            eprintln!("{}", diag.render(&src));
            std::process::exit(1);
        }
    };

    let ast = match parser::parse(&tokens, path) {
        Ok(a) => a,
        Err(diag) => {
            eprintln!("{}", diag.render(&src));
            std::process::exit(1);
        }
    };

    let result = check::check(&ast, path);
    (src, result, ast)
}

fn run_check(path: &str) {
    let (src, result, _ast) = load_and_check(path);
    for d in &result.diagnostics {
        eprintln!("{}", d.render(&src));
    }
    if result.has_errors() {
        std::process::exit(1);
    }
}

fn run_build(path: &str, out_path: Option<&str>) {
    let (src, result, ast) = load_and_check(path);
    for d in &result.diagnostics {
        eprintln!("{}", d.render(&src));
    }
    if result.has_errors() {
        std::process::exit(1);
    }

    let comp = lower::lower(&ast, &result.namespace);
    let ir_text = emit::emit(&comp);

    let output = match out_path {
        Some(p) => p.to_string(),
        None => {
            let stem = std::path::Path::new(path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "out".to_string());
            let dir = std::path::Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            if dir.is_empty() {
                format!("{}.uic", stem)
            } else {
                format!("{}/{}.uic", dir, stem)
            }
        }
    };

    if let Err(e) = std::fs::write(&output, &ir_text) {
        eprintln!("error: cannot write `{}`: {}", output, e);
        std::process::exit(1);
    }
}
