#![allow(dead_code)]

mod openai;
mod server;
mod common;

use anyhow::anyhow;
use clap::Parser;
use tokio::io::AsyncWriteExt;
use crate::common::ApplicationParam;
use crate::openai::OpenAiClient;

#[tokio::main]
async fn main() {
  let param = ApplicationParam::parse();
  match run_main(param).await {
    Ok(_) => std::process::exit(0),
    Err(err) => {
      let _ = print_stderr(err.to_string().as_bytes()).await;
      std::process::exit(1);
    }
  }
}

async fn run_main(param: ApplicationParam) -> anyhow::Result<()> {
  if param.server.as_ref().is_some() {
    return run_server(param).await;
  }
  return run_client(param).await;
}

async fn run_server(param: ApplicationParam) -> anyhow::Result<()> {
  warp::serve(server::server(&param))
    .run(param.server.unwrap())
    .await;
  return Ok(());
}

async fn run_client(param: ApplicationParam) -> anyhow::Result<()> {
  if let Some(token_str) = param.parse.as_ref() {
    let token = OpenAiClient::parse_token(token_str)?;
    return print_stdout(format!("\n{}\n", token).as_bytes()).await;
  }
  let client = OpenAiClient::create(param.proxy.clone(), true)
    .map_err(|err| anyhow!("Create Client Error: {}", err))?;
  if let Some(refresh_token) = param.refresh.as_ref() {
    let result = client.refresh_token(refresh_token).await?;
    return print_stdout(format!("\nAccess Token: {}\n", result.access_token).as_bytes()).await;
  }
  if let (Some(username), Some(password)) = (param.username.as_ref(), param.password.as_ref()) {
    let code = client.get_code(&client.get_state().await?, username, password).await?;
    let result = client.access_token(&code).await?;
    return print_stdout(format!("\nAccess Token: {}\n\nRefresh Token: {}\n", result.access_token, result.refresh_token).as_bytes()).await;
  }
  return Ok(());
}

async fn print_stdout(byte: &[u8]) -> anyhow::Result<()> {
  let mut stdout = tokio::io::stdout();
  let _ = stdout.write_all(byte).await
    .map_err(|err| anyhow!("Write Stdout Error: {}", err));
  let _ = stdout.flush().await;
  return Ok(());
}

async fn print_stderr(byte: &[u8]) -> anyhow::Result<()> {
  let mut stderr = tokio::io::stderr();
  let _ = stderr.write_all(byte).await
    .map_err(|err| anyhow!("Write Stderr Error: {}", err));
  let _ = stderr.flush().await;
  return Ok(());
}