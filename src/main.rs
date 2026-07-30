use clap::Parser;
use tera::Tera;
use tera::Context;

use std::fs::File;
use std::io::Write;
use std::io;
use std::io::Read;
use std::process;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    template_glob: String,
    template_file: String,
    json_file: String,
    out_file: String
}

fn main() -> io::Result<()> {
    // read command line arguments
    let args = Args::parse();

    // relative glob to template file
    let template_glob = &args.template_glob;
    // template file path relative to 'template_glob'
    let template_file = &args.template_file;
    // relative path json file
    let json_file = &args.json_file;
    // relative path to output file
    let out_file = &args.out_file;

    // open and read json file
    let mut file = File::open(json_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Parse the string of data into serde_json::Value.
    let v: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: failed to parse JSON file '{}': {}", json_file, e);
            process::exit(1);
        }
    };

    // Convert serde_json::Value to tera::Context
    let ctx: Context = match Context::from_serialize(&v) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: failed to build template context from '{}': {}", json_file, e);
            process::exit(1);
        }
    };

    let tera = match Tera::new(template_glob) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: failed to parse templates matching '{}': {}", template_glob, e);
            process::exit(1);
        }
    };

    match tera.render(template_file, &ctx) {
        Ok(s) => {
            let mut f_out = match File::create(out_file) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error: unable to create output file '{}': {}", out_file, e);
                    process::exit(1);
                }
            };
            f_out.write_all(s.as_bytes())?;
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found in context") {
                eprintln!("Error: template variable not found in context.");
                eprintln!("  {}", msg);
                eprintln!(
                    "  Hint: make sure all variables referenced in the template are \
                     provided in the JSON context file '{}'.",
                    json_file
                );
            } else {
                eprintln!("Error: failed to render template '{}': {}", template_file, e);
            }
            process::exit(1);
        }
    };

    Ok(())
}
