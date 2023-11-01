mod dices;
mod hand_worker;
mod score_box;
mod utils;
mod play_worker;
mod distr_worker;
mod weight_worker;

use crate::hand_worker::load_hands;
use clap::{Parser, Subcommand};
use hand_worker::learn_hands;
use crate::distr_worker::{learn_hand_distributions, load_hand_distributions};
use crate::weight_worker::{export_weights, load_weights, strategy_learn};
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

        /// Learn game strategies, supply both laps and sub-laps
        #[arg(short, value_name="LAPS", num_args(2))]
        game: Option<Vec<i64>>,

        /// Export full output from yatzy hands learning
        #[arg(short)]
        full: bool,

        /// Bonus to use in game strategy learning
        #[arg(short)]
        bonus: Option<u32>,
    },

    /// Export models to readable format
    Export {
        /// Export score models for yatzy hands
        #[arg(short, long)]
        scores: bool,

        /// Export distribution for yatzy hands
        #[arg(short, long)]
        distr: bool,

        /// Export weights for yatzy strategy
        #[arg(short, long, value_name="BONUS")]
        weights: Option<u32>,
    },

    /// Run game of yatzy
    Play {
        /// Human (own dices) vs MC
        #[arg(short, long, value_name="BONUS")]
        interactive: Option<u32>,
    },
}

fn main() -> Result<(), String> {
    let args = Cli::parse();

    check_path_create_folder(&args.path, None)?;

    match args.command {
        Commands::Learn {scores, rule, distr, game,full, bonus} => {
            learn_models(&args.path, scores, rule, distr, game, full, bonus)?
        },
        Commands::Export {scores, distr, weights} => {
            export_models(&args.path, scores, distr, weights)?;
        },
        Commands::Play {interactive} => {
            play_game(&args.path, interactive)?;
        },
    }

    println!("Done!");
    Ok(())
}

fn learn_models(path: &str, scores: Option<i64>, rule: Option<usize>, distr: Option<i64>, game: Option<Vec<i64>>, full: bool, bonus: Option<u32>) -> Result<(), String> {
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
        strategy_learn(path, laps, bonus)?;
    }

    Ok(())
}

fn export_models(path: &str, scores: bool, distr: bool, weights: Option<u32>) -> Result<(), String> {
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

    if let Some(bonus) = weights {
        println!("Start loading weights");
        if let Some((generation, weights)) = load_weights(path, Some(bonus))? {
            println!("Start exporting weights");
            export_weights(path, Some(bonus), generation, &weights)?;
        } else {
            println!("No weights found in weights file");
        };
    }

    Ok(())
}

fn play_game(path: &str, interactive: Option<u32>) -> Result<(), String> {

    if let Some(bonus) = interactive {
        play_with_own_dices(path, bonus)?;
    }

    Ok(())
}
