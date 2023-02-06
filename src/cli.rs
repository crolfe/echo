use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Args {
    #[arg(short, long,default_value_t=8080)]
    pub port: u16,

    #[arg(short, long, default_value_t = String::from("localhost"))]
    pub address: String
}

pub fn parse_args() -> Args {
    Args::parse()
}
