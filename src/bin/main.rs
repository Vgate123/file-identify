use clap::Parser;
use file_identify::{tags_from_filename, tags_from_path};
use std::process;

#[derive(Parser)]
#[command(name = "file-identify")]
#[command(about = "File identification tool - determines file types based on extensions, content, and shebangs")]
#[command(version)]
struct Args {
    /// Only use filename for identification (don't read file contents)
    #[arg(long)]
    filename_only: bool,
    
    /// Path to the file to identify
    path: String,
}

fn main() {
    let args = Args::parse();

    let tags = if args.filename_only {
        tags_from_filename(&args.path)
    } else {
        match tags_from_path(&args.path) {
            Ok(tags) => tags,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    };

    if tags.is_empty() {
        process::exit(1);
    }

    // Sort tags for consistent output
    let mut sorted_tags: Vec<&str> = tags.iter().cloned().collect();
    sorted_tags.sort();

    // Output as JSON array (matching Python version behavior)
    match serde_json::to_string(&sorted_tags) {
        Ok(json) => println!("{}", json),
        Err(_) => process::exit(1),
    }
}