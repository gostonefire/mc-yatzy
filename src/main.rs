mod dices;
mod game_box;
mod game_worker;
mod hand_worker;
mod score_box;
mod utils;
mod play_worker;

use crate::game_worker::{learn_game, load_game, load_mcscore};
use crate::hand_worker::load_hands;
use clap::{Parser, Subcommand};
use hand_worker::learn_hands;
use crate::play_worker::play_with_own_dices;

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

        /// Learn the game model
        #[arg(short, value_name="LAPS")]
        game: Option<i64>,

        /// Export debug output from yatzy hands learning
        #[arg(short)]
        debug: bool,
    },

    /// Export models to readable format
    Export {
        /// Export score models for yatzy hands
        #[arg(short, long)]
        scores: bool,

        /// Export the game model
        #[arg(short, long)]
        game: bool,

        /// Export the mc score model
        #[arg(short, long)]
        mcscore: bool,
    },

    /// Print some statistics
    Stats {
        #[arg(short, long)]
        mcscore: bool,
    },

    /// Run game of yatzy
    Play,
}

fn main() -> Result<(), String> {
    let args = Cli::parse();

    match args.command {
        Commands::Learn {scores, rule, game, debug} => {
            learn_models(&args.path, scores, rule, game, debug)?
        },
        Commands::Export {scores, game, mcscore} => {
            export_models(&args.path, scores, game, mcscore)?;
        },
        Commands::Stats {mcscore} => {
            print_models_statistics(&args.path, mcscore)?
        },
        Commands::Play {} => {
            play_game(&args.path)?;
        },
    }

    println!("Done!");
    Ok(())
}

fn learn_models(path: &str, scores: Option<i64>, rule: Option<usize>, game: Option<i64>, debug: bool) -> Result<(), String> {
    if let Some(laps) = scores {
        println!("Start learning rules");
        learn_hands(laps, path, rule, debug)?;
    }

    if let Some(laps) = game {
        println!("Start learning game");
        learn_game(laps, path)?;
    }

    Ok(())
}

fn print_models_statistics(path: &str, mcgame: bool) -> Result<(), String> {
    if mcgame {
        println!("Start loading mc score file");
        let mc_score = load_mcscore(path)?;
        println!("...calculating statistics");
        println!("{}\n", mc_score.statistics()?);
    }

    Ok(())
}

fn export_models(path: &str, scores: bool, game: bool, mcscore: bool) -> Result<(), String> {
    if scores {
        println!("Start loading rules");
        let hand_rules = load_hands(path)?;
        println!("Start exporting rules");
        hand_rules.iter().for_each(|h| h.export_optimal_holds(path).unwrap());
    }

    if game {
        println!("Start loading game");
        let game_rules = load_game(path)?;
        println!("Start exporting game");
        game_rules.export_optimal_games(path)?;
    }

    if mcscore {
        println!("Start loading mc score file");
        let mc_score = load_mcscore(path)?;
        println!("Start exporting scores");
        mc_score.export_score(path)?;
    }

    Ok(())
}

fn play_game(path: &str) -> Result<(), String> {
    play_with_own_dices(path)
}
