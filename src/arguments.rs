use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// Path to a singular file or an entire directory.
    #[arg(short, long)]
    pub input: String,

    /// Must be path to a singular file if the input is a singular file, and vice versa.
    #[arg(short, long)]
    pub output: String,

    /// Optional flag to overwrite files at the output path.
    #[arg(short, long)]
    pub force: bool,

    /// Optional flag to downsize the image according to a max_size.
    #[arg(short, long)]
    pub downsize: bool,

    /// Must be specified if downsize is specified.
    #[arg(short, long)]
    pub max_size: Option<u32>
}
