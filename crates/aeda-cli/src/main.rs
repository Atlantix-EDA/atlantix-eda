//! Atlantix EDA CLI
//!
//! Component library management and generation tool.

mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aeda")]
#[command(author = "Atlantix EDA")]
#[command(version)]
#[command(about = "Atlantix EDA - Component library management and generation", long_about = None)]
struct Cli {
    /// Use a custom data directory instead of ~/atlantix-eda
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available component libraries
    List {
        /// Component type to list (resistors, capacitors, etc.)
        #[arg(default_value = "all")]
        component_type: String,
    },

    /// Generate component libraries
    Generate {
        #[command(subcommand)]
        what: GenerateCommands,
    },

    /// Export libraries to different formats
    Export {
        #[command(subcommand)]
        format: ExportCommands,
    },

    /// Show information about a specific library
    Info {
        /// Library path (e.g., resistor::E96_0603)
        library: String,
    },

    /// Initialize the data directory structure
    Init,

    /// Show current configuration and paths
    Config,
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate resistor libraries
    Resistors {
        /// E-series to generate (e.g., E96, E24, E12)
        #[arg(short, long, default_value = "E96")]
        series: String,

        /// Packages to generate (comma-separated: 0402,0603,0805,1206)
        #[arg(short, long, default_value = "0603,0805,1206")]
        packages: String,
    },

    /// Generate capacitor libraries
    Capacitors {
        /// Dielectric type (X7R, C0G, X5R)
        #[arg(short, long, default_value = "X7R")]
        dielectric: String,

        /// Packages to generate
        #[arg(short, long, default_value = "0603,0805,1206")]
        packages: String,
    },
}

#[derive(Subcommand)]
enum ExportCommands {
    /// Export to KiCad format
    Kicad {
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Export to Stencil DSL manifest format
    Stencil {
        /// Output directory (defaults to data/libraries/)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Export to Altium format (future)
    Altium {
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Determine data directory
    let data_dir = cli.data_dir.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("atlantix-eda"))
            .unwrap_or_else(|| PathBuf::from("atlantix-eda"))
    });

    let result = match cli.command {
        Commands::List { component_type } => {
            commands::list::run(&data_dir, &component_type)
        }
        Commands::Generate { what } => match what {
            GenerateCommands::Resistors { series, packages } => {
                commands::generate::resistors(&data_dir, &series, &packages)
            }
            GenerateCommands::Capacitors { dielectric, packages } => {
                commands::generate::capacitors(&data_dir, &dielectric, &packages)
            }
        },
        Commands::Export { format } => match format {
            ExportCommands::Kicad { output } => {
                commands::export::to_kicad(&data_dir, output.as_deref())
            }
            ExportCommands::Stencil { output } => {
                commands::export::to_stencil(&data_dir, output.as_deref())
            }
            ExportCommands::Altium { output } => {
                commands::export::to_altium(&data_dir, output.as_deref())
            }
        },
        Commands::Info { library } => {
            commands::info::run(&data_dir, &library)
        }
        Commands::Init => {
            commands::init::run(&data_dir)
        }
        Commands::Config => {
            commands::config::run(&data_dir)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
