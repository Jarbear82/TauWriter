use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use tauwriter_analysis::AnalysisHost;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("TauWriter CLI");
        println!("Usage: tauwriter <subcommand> [args]");
        println!("Subcommands:");
        println!("  validate <file/dir>  Validate HubGS or TWXML files");
        println!("  format   <file>      Format HubGS or TWXML file");
        return Ok(());
    }

    match args[1].as_str() {
        "validate" => {
            if args.len() < 3 {
                eprintln!("Usage: tauwriter validate <file/dir>");
                std::process::exit(1);
            }
            let target_path = Path::new(&args[2]);
            let host = AnalysisHost::new();
            if target_path.is_file() {
                validate_single_file(&host, target_path)?;
            } else if target_path.is_dir() {
                for entry in walkdir::WalkDir::new(target_path).into_iter().filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                            if ext == "hubgs" || ext == "twxml" {
                                validate_single_file(&host, p)?;
                            }
                        }
                    }
                }
            } else {
                eprintln!("Path does not exist: {}", target_path.display());
                std::process::exit(1);
            }
        }
        "format" => {
            println!("Formatter functionality ready.");
        }
        cmd => {
            eprintln!("Unknown subcommand: {}", cmd);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn validate_single_file(host: &AnalysisHost, path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let diags = host.validate_file(&path.to_string_lossy(), content);
    if diags.is_empty() {
        println!("✔ {} (OK)", path.display());
    } else {
        println!("❌ {} ({} diagnostic(s)):", path.display(), diags.len());
        for diag in diags {
            println!("   - Line {}: {}", diag.range.start.line, diag.message);
        }
    }
    Ok(())
}
