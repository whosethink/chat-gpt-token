use std::net::SocketAddr;
use anyhow::anyhow;
use clap::Parser;
use reqwest::Proxy;

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct ApplicationParam {
  #[arg(short = 'u', long = "username", help = "openai username", requires = "password")]
  pub username: Option<String>,

  #[arg(short = 'p', long = "password", help = "openai password", requires = "username")]
  pub password: Option<String>,

  #[arg(long = "refresh", help = "refresh new token")]
  pub refresh: Option<String>,

  #[arg(long = "parse", help = "parse token info")]
  pub parse: Option<String>,

  #[arg(long = "proxy", help = "socks5://127.0.0.1:8080", value_parser = ApplicationParam::parse_proxy)]
  pub proxy: Option<Proxy>,

  #[arg(long = "server", help = "127.0.0.1:8000")]
  pub server: Option<SocketAddr>,

}

impl ApplicationParam {
  fn parse_proxy(proxy: &str) -> anyhow::Result<Proxy> {
    return Proxy::all(proxy).map_err(|err| anyhow!("Create Proxy Error: {}", err));
  }
}