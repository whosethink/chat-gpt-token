mod openai;

use std::time::Duration;
use anyhow::anyhow;
use clap::Parser;
use tokio::io::AsyncWriteExt;
use crate::openai::{ApplicationParam, OpenAiClient};

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
  todo!()
}

async fn run_client(param: ApplicationParam) -> anyhow::Result<()> {
  if let Some(token_str) = param.parse.as_ref() {
    let (token_info, user_info) = OpenAiClient::parse_token(token_str)?;
    if let Some(token) = token_info {
      let _ = print_stdout(format!("\n{}", token).as_bytes()).await;
    }
    if let Some(user) = user_info {
      let _ = print_stdout(format!("{}\n", user).as_bytes()).await;
    }
    return Ok(());
  }
  let client = OpenAiClient::create(param.proxy.clone())
    .map_err(|err| anyhow!("Create Client Error: {}", err))?;
  if let Some(refresh_token) = param.refresh.as_ref() {
    let access = client.refresh_token(refresh_token).await?;
    return print_stdout(format!("\nAccess Token: {}\n", access).as_bytes()).await;
  }
  if let (Some(username), Some(password)) = (param.username.as_ref(), param.password.as_ref()) {
    let code = client.get_code(&client.get_state().await?, username, password).await?;
    let (access, refresh) = client.access_token(&code).await?;
    let _ = print_stdout(format!("\nAccess Token: {}\n\n", access).as_bytes()).await;
    return print_stdout(format!("Refresh Token: {}\n", refresh).as_bytes()).await;
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

