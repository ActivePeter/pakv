mod pakv;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 获取键值
    Get(Get),
    /// 设置键值
    Set(Set),
    /// 移除键值
    Del(Del),
}

#[derive(Args)]
struct Get {
    #[clap(value_parser)]
    key: String,
}
#[derive(Args)]
struct Set {
    #[clap(value_parser)]
    key: String,
    #[clap(value_parser)]
    value: String,
}
#[derive(Args)]
struct Del {
    #[clap(value_parser)]
    key: String,
}

fn main() {
    let cli= Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Get(g) => {
            println!("get {:?}", g.key)
        }
        Commands::Set(s) => {
            println!("set {:?} {:?}", s.key,s.value)
        }
        Commands::Del(d) => {
            println!("get {:?}", d.key)
        }
    }
}