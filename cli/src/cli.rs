use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(long, short)]
    pub dev: Option<String>,
    #[clap(long, short)]
    pub length: u16,
    #[clap(long, short)]
    pub red: Option<u8>,
    #[clap(long, short)]
    pub green: Option<u8>,
    #[clap(long, short)]
    pub blue: Option<u8>,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
}
