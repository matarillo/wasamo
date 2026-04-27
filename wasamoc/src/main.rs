mod ast;
mod check;
mod diagnostic;
mod lexer;
mod parser;

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
        Some(cmd) => {
            eprintln!("error: unknown command `{}`", cmd);
            eprintln!("usage: wasamoc check <file.ui>");
            std::process::exit(1);
        }
        None => {
            eprintln!("usage: wasamoc check <file.ui>");
            std::process::exit(1);
        }
    }
}

fn run_check(path: &str) {
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

    let warnings = check::check(&ast, path);
    for w in &warnings {
        eprintln!("{}", w.render(&src));
    }
}
