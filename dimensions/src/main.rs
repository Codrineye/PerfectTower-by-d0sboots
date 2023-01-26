//! Utility to make use of the dimension library.

use clap::{Args, Parser, Subcommand, ValueEnum};
use dimensions::Dimension;
use serde_json;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    Pretty,
    Json,
    JsonPretty,
    Debug,
    Compact,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(short, long, default_value = "pretty")]
    /// Format output according to the given formatter
    format: Format,

    #[arg(short, long)]
    /// Output file to write to
    out: Option<PathBuf>,
}

fn print_dim(dim: &Dimension, file: &mut Option<File>, format: Format) {
    fn print_internal<T: Write>(dim: &Dimension, writer: &mut T, format: Format) {
        match format {
            Format::Pretty => writeln!(writer, "{}", dim).unwrap(),
            Format::Json => {
                serde_json::to_writer(&mut *writer, dim).unwrap();
                writer.write_all(b"\n").unwrap();
            }
            Format::JsonPretty => {
                serde_json::to_writer_pretty(&mut *writer, dim).unwrap();
                writer.write_all(b"\n").unwrap();
            }
            Format::Debug => writeln!(writer, "{:#?}", dim).unwrap(),
            Format::Compact => dim.write_compact(writer).unwrap(),
        }
    }
    match file {
        None => {
            let mut writer = io::stdout().lock();
            print_internal(dim, &mut writer, format)
        }
        Some(f) => print_internal(dim, f, format),
    }
}

#[derive(Subcommand)]
enum Command {
    /// Show values for a specific dimension, with "x y" coords
    Show(Show),

    /// Search for specific dimensions
    Search(Search),
}

#[derive(Args)]
struct Show {
    #[arg(allow_negative_numbers = true)]
    #[arg(num_args = 2, value_names(["X", "Y"]))]
    show: Vec<i32>,
}

#[derive(Args)]
struct Search {
    #[arg(long, allow_negative_numbers = true)]
    #[arg(num_args = 2, value_names(["X", "Y"]))]
    #[arg(requires = "coord_max")]
    /// Minimum bound for coords
    coord_min: Vec<i32>,

    #[arg(long, allow_negative_numbers = true)]
    #[arg(num_args = 2, value_names(["X", "Y"]))]
    #[arg(requires = "coord_min")]
    /// Maximum bound for coords, inclusive
    coord_max: Vec<i32>,
}

fn main() {
    let args = Cli::parse();
    let mut file: Option<File> = match args.out {
        None => None,
        Some(path) => Some(File::create(path).unwrap()),
    };
    match &args.command {
        Command::Show(Show { show }) => {
            if let [x, y] = show[..] {
                let dim = Dimension::new(x, y);
                print_dim(&dim, &mut file, args.format);
            } else {
                panic!("No args in {show:?}");
            }
        }
        Command::Search(search) => {
            let [x_min, y_min] = match search.coord_min[..] {
                [x, y] => [x, y],
                _ => [-60000, -60000],
            };
            let [x_max, y_max] = match search.coord_max[..] {
                [x, y] => [x, y],
                _ => [60000, 60000],
            };
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    let dim = Dimension::new(x, y);
                    print_dim(&dim, &mut file, args.format);
                }
            }
        }
    }
}
