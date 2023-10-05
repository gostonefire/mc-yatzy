mod dices;
mod hand_worker;
mod score_box;
mod utils;
mod play_worker;
mod distr_worker;
mod game_worker;

use crate::hand_worker::load_hands;
use clap::{Parser, Subcommand};
use hand_worker::learn_hands;
use crate::distr_worker::{learn_hand_distributions, load_hand_distributions};
use crate::game_worker::mc_play;
use crate::play_worker::play_with_own_dices;
use crate::utils::check_path_create_folder;

static EXPORT_DIR: &str = "export";
static DEBUG_DIR: &str = "debug";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to directory holding models and exports
    #[arg(short, long, value_name = "DIR")]
    path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Learn models Monte Carlo style
    Learn {
        /// Learn models for yatzy hands
        #[arg(short, value_name="LAPS")]
        scores: Option<i64>,

        /// Choose specific yatzy hand to learn, leave value empty for all
        #[arg(short, value_name="HAND (zero based)")]
        rule: Option<usize>,

        /// Learn distribution for yatzy hands
        #[arg(short, value_name="LAPS")]
        distr: Option<i64>,

        /// Learn game strategies
        #[arg(short, value_name="LAPS")]
        game: Option<i64>,

        /// Export full output from yatzy hands learning
        #[arg(short)]
        full: bool,
    },

    /// Export models to readable format
    Export {
        /// Export score models for yatzy hands
        #[arg(short, long)]
        scores: bool,

        /// Export distribution for yatzy hands
        #[arg(short, long)]
        distr: bool,
    },

    /// Run game of yatzy
    Play {
        /// Human (own dices) vs MC
        #[arg(short, long)]
        interactive: bool,
    },
}

fn main() -> Result<(), String> {
    let args = Cli::parse();

    check_path_create_folder(&args.path, None)?;

    match args.command {
        Commands::Learn {scores, rule, distr, game, full} => {
            learn_models(&args.path, scores, rule, distr, game, full)?
        },
        Commands::Export {scores, distr} => {
            export_models(&args.path, scores, distr)?;
        },
        Commands::Play {interactive} => {
            play_game(&args.path, interactive)?;
        },
    }

    println!("Done!");
    Ok(())
}

fn learn_models(path: &str, scores: Option<i64>, rule: Option<usize>, distr: Option<i64>, game: Option<i64>, full: bool) -> Result<(), String> {
    if full {
        check_path_create_folder(path, Some(DEBUG_DIR))?;
    }

    if let Some(laps) = scores {
        println!("Start learning rules");
        learn_hands(laps, path, rule, full)?;
    }

    if let Some(laps) = distr {
        println!("Start learning hand distributions");
        learn_hand_distributions(laps, path, rule)?;
    }

    if let Some(laps) = game {
        println!("Start learning game strategies");
        mc_play(path, laps)?;
    }

    Ok(())
}

fn export_models(path: &str, scores: bool, distr: bool) -> Result<(), String> {
    check_path_create_folder(path, Some(EXPORT_DIR))?;

    if scores {
        println!("Start loading rules");
        let hand_rules = load_hands(path, false)?;
        println!("Start exporting rules");
        hand_rules.iter().for_each(|h| h.export_optimal_holds(path).unwrap());
    }

    if distr {
        println!("Start loading distributions");
        let hand_distr = load_hand_distributions(path, false)?;
        println!("Start exporting rules");
        hand_distr.iter().for_each(|h| h.export_distribution(path).unwrap());
    }

    Ok(())
}

fn play_game(path: &str, interactive: bool) -> Result<(), String> {

    if interactive {
        play_with_own_dices(path)?;
    }

    Ok(())
}
