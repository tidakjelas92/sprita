use clap::Parser;

#[derive(Parser)]
pub struct Arguments {
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub output: String,

    #[arg(short, long)]
    pub force: bool,

    #[arg(short, long)]
    pub downsize: bool,

    #[arg(short, long)]
    pub max_size: Option<i32>
}
