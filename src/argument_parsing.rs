pub use clap::Parser;

#[derive(Parser)]
pub(crate) struct Args {
    #[arg(short, long)]
    pub input_file: String,

    #[arg(short, long)]
    pub palette_file: String,

    #[arg(short,long,default_value=None)]
    pub output_file: Option<String>,
}
