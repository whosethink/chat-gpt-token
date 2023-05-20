use std::fmt::{Display, Formatter};
use anyhow::{anyhow, bail};
use chrono::NaiveDateTime;
use clap::Parser;
use console::style;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use reqwest::{Proxy, Url};
use reqwest::redirect::Policy;

const HEADER_AGENT: &'static str = "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/113.0";
const HEADER_REFERER: &'static str = "https://ios.chat.openai.com/";
const IOS_CLIENT_ID: &'static str = "pdlLIX2Y72MIl2rhLhTE9VV9bN905kBh";

fn main() {
  let config = ApplicationParam::parse();
  match run_main(config) {
    Ok(code) => std::process::exit(code),
    Err(err) => {
      eprintln!("{}", style(err.to_string()).red());
      std::process::exit(1);
    }
  }
}

fn run_main(config: ApplicationParam) -> anyhow::Result<i32> {
  let proxy = match config.proxy.as_ref() {
    None => None,
    Some(p) => {
      Some(Proxy::all(p).map_err(|err| anyhow!("Create Proxy Error: {} {}", p, err))?)
    }
  };
  let client = creat_client(proxy)?;
  if let Some(at) = config.access_token.as_ref() {
    let segments = at.split('.').collect::<Vec<&str>>();
    if let Some(&token_info_str) = segments.get(0) {
      let token_info_str = base64_str_decode(token_info_str)?;
      let token_info = serde_json::from_str::<JwtTokenInfo>(&token_info_str)
        .map_err(|_| anyhow!("Parse Token Info Error: {}", token_info_str))?;
      print!("{}", token_info);
    }
    if let Some(&user_info_str) = segments.get(1) {
      let user_info_str = base64_str_decode(user_info_str)?;
      let user_info = serde_json::from_str::<JwtUserInfo>(&user_info_str)
        .map_err(|_| anyhow!("Parse User Info Error: {}", user_info_str))?;
      print!("{}", user_info);
    }
    return Ok(0);
  }
  if let Some(rt) = config.refresh_token.as_ref() {
    let access = refresh_token(rt, &client)?;
    println!("Access Token: {}", access);
    return Ok(0);
  }
  if config.username.is_none() || config.password.is_none() {
    bail!("Param Error: username/password is none");
  }
  let state = get_state(&client)?;
  let (access, refresh) = access_token(
    &get_code(&state, config.username.as_ref().unwrap(), config.password.as_ref().unwrap(), &client)?,
    &client
  )?;
  println!("Access Token: {}\n", access);
  println!("Refresh Token: {}", refresh);
  return Ok(0);
}

fn creat_client(proxy: Option<Proxy>) -> anyhow::Result<Client> {
  let mut headers = HeaderMap::with_capacity(2);
  headers.insert(USER_AGENT, HeaderValue::from_static(HEADER_AGENT));
  headers.insert(REFERER, HeaderValue::from_static(HEADER_REFERER));
  let mut client =  Client::builder()
    .default_headers(headers)
    .cookie_store(true)
    .redirect(Policy::none());
  if let Some(proxy) = proxy {
    client = client.proxy(proxy);
  }
  return client.build()
    .map_err(|err| anyhow!("Create Client Error: {}", err));
}

fn get_state(client: &Client) -> anyhow::Result<String> {
  let mut res = client.get("https://auth0.openai.com/authorize")
    .query(&[("client_id", IOS_CLIENT_ID)])
    .query(&[("audience", "https://api.openai.com/v1")])
    .query(&[("redirect_uri", "com.openai.chat://auth0.openai.com/ios/com.openai.chat/callback")])
    .query(&[("scope", "openid email profile offline_access model.request model.read organization.read offline")])
    .query(&[("response_type", "code")])
    .query(&[("code_challenge", "t1RM5eR6dToh4VAe85qAf4ANdsnob6ANiuyl_z67mr4")])
    .query(&[("code_challenge_method", "S256")])
    .query(&[("prompt", "login")])
    .send()
    .map_err(|err| anyhow!("Get State Error: {}", err))?;
  let location = res.headers().get("location")
    .ok_or(anyhow!("Get State Error: location is none"))?
    .to_str()
    .map_err(|err| anyhow!("Get State Error: {}", err))?;
  let url = Url::options().base_url(Some(&Url::parse("https://api.openai.com/").unwrap()))
    .parse(location)
    .map_err(|err| anyhow!("Get State Error: {}", err))?;
  let (_, state) = url.query_pairs()
    .filter(|(k, v)| k.eq_ignore_ascii_case("state"))
    .next()
    .ok_or(anyhow!("Get State Error: state is none"))?;
  return Ok(state.to_string());
}

fn get_code(state: &str, username: &str, password: &str, client: &Client) -> anyhow::Result<String> {
  let req1 = client.post("https://auth0.openai.com/u/login/identifier")
    .query(&[("state", state)])
    .form(&PasswordLoginReq01::create(state, username));
  let _ = req1.send()
    .map_err(|err| anyhow!("Get Code1 Error: {}", err))?;
  let res2 = client.post("https://auth0.openai.com/u/login/password")
    .json(&PasswordLoginReq02::create(state, username, password))
    .send()
    .map_err(|err| anyhow!("Get Code2 Error: {}", err))?;
  let location2 = res2.headers().get("location")
    .ok_or(anyhow!("Get Code2 Error: location is none"))?
    .to_str()
    .map_err(|err| anyhow!("Get Code2 Error: {}", err))?;
  let res3 = client.get(format!("https://auth0.openai.com{}", location2))
    .send()
    .map_err(|err| anyhow!("Get Code3 Error: {}", err))?;
  let location3 = res3.headers().get("location")
    .ok_or(anyhow!("Get Code3 Error: location is none"))?
    .to_str()
    .map_err(|err| anyhow!("Get Code3 Error: {}", err))?;
  let queries = Url::parse(location3)
    .map_err(|err| anyhow!("Get Code3 Error: {}", err))?;
  let (_, code) =  queries.query_pairs()
    .filter(|(k, v)| k.eq_ignore_ascii_case("code"))
    .next()
    .ok_or(anyhow!("Get Code3 Error: code is none"))?;
  return Ok(code.to_string());
}

fn access_token(code: &str, client: &Client) -> anyhow::Result<(String, String)> {
  let param = AccessTokenReq::create(code);
  let mut req = client.post("https://auth0.openai.com/oauth/token")
    .json(&param);
  let res = req.send()
    .map_err(|err| anyhow!("Access Token Error: {}", err))?;
  if !res.status().is_success() {
    bail!("Access Token Error: {} {}", res.status(), res.text().unwrap_or("Bad Response".to_string()));
  }
  return res.json::<AccessTokenRes>()
    .map_err(|err| anyhow!("Access Token Error: {}", err))
    .map(|r| (r.access_token.clone(), r.refresh_token.clone()));
}

fn refresh_token(token: &str, client: &Client) -> anyhow::Result<String> {
  let param = RefreshTokenReq::create(token);
  let mut req = client.post("https://auth0.openai.com/oauth/token")
    .json(&param);
  let res = req.send()
    .map_err(|err| anyhow!("Refresh Token Error: {}", err))?;
  if !res.status().is_success() {
    bail!("Refresh Token Error: {} {}", res.status(), res.text().unwrap_or("Bad Response".to_string()));
  }
  return res.json::<RefreshTokenRes>()
    .map_err(|err| anyhow!("Refresh Token Error: {}", err))
    .map(|r| r.access_token());
}

fn base64_str_decode(base64_str: &str) -> anyhow::Result<String> {
  let result = base64::decode(base64_str)
    .map_err(|_| anyhow!("Base64 Decode Error: {}", base64_str))?;
  return Ok(String::from_utf8(result).map_err(|_| anyhow!("Base64 String Error: {}", base64_str))?);
}

#[derive(Parser)]
#[command(author, version, about)]
struct ApplicationParam {

  #[arg(short = 'u', long = "username")]
  username: Option<String>,

  #[arg(short = 'p', long = "password")]
  password: Option<String>,

  #[arg(long = "access-token", help = "parse access token")]
  access_token: Option<String>,

  #[arg(long = "refresh-token", help = "refresh access token")]
  refresh_token: Option<String>,

  #[arg(long = "proxy", help = "like socks5://127.0.0.1:8080")]
  proxy: Option<String>,
}

#[derive(Debug, Serialize)]
struct PasswordLoginReq01<'a> {
  state: &'a str,
  username: &'a str
}

impl <'a> PasswordLoginReq01<'a> {
  fn create(state: &'a str, username: &'a str) -> PasswordLoginReq01<'a> {
    return PasswordLoginReq01 {
      state,
      username
    };
  }
}

#[derive(Debug, Serialize)]
struct PasswordLoginReq02<'a> {
  state: &'a str,
  username: &'a str,
  password: &'a str,
}

impl <'a> PasswordLoginReq02<'a> {
  fn create(state: &'a str, username: &'a str, password: &'a str) -> PasswordLoginReq02<'a> {
    return PasswordLoginReq02 {
      state,
      username,
      password
    };
  }
}

#[derive(Debug, Serialize)]
struct AccessTokenReq<'a> {
  redirect_uri: &'a str,
  client_id: &'a str,
  grant_type: &'a str,
  code: &'a str,
  code_verifier: &'a str
}

impl AccessTokenReq<'_> {
  fn create(code: &str) -> AccessTokenReq {
    return AccessTokenReq {
      redirect_uri: "com.openai.chat://auth0.openai.com/ios/com.openai.chat/callback",
      grant_type: "authorization_code",
      client_id: IOS_CLIENT_ID,
      code,
      code_verifier: "IkrrBD89CBmwwzM-csfBnWKLMan5uE7laCMd2YTcPWE"
    };
  }
}

#[derive(Debug, Deserialize)]
struct AccessTokenRes {
  access_token: String,
  refresh_token: String,
  id_token: String,
  scope: String,
  expires_in: u32,
  token_type: String
}

impl AccessTokenRes {
  fn access_token(&self) -> String {
    return self.access_token.clone();
  }

  fn refresh_token(&self) -> String {
    return self.refresh_token.clone();
  }
}

#[derive(Debug, Serialize)]
struct RefreshTokenReq<'a> {
  refresh_token: &'a str,
  client_id: &'a str,
  grant_type: &'a str
}

impl RefreshTokenReq<'_> {
  fn create(token: &str) -> RefreshTokenReq {
    return RefreshTokenReq {
      refresh_token: token,
      client_id: IOS_CLIENT_ID,
      grant_type: "refresh_token"
    };
  }
}

#[derive(Debug, Deserialize)]
struct RefreshTokenRes {
  access_token: String,
  id_token: String,
  scope: String,
  expires_in: u32,
  token_type: String
}

impl RefreshTokenRes {
  fn access_token(&self) -> String {
    return self.access_token.clone();
  }
}

#[derive(Debug, Deserialize)]
struct JwtTokenInfo {
  alg: String,
  typ: String,
  kid: String
}

impl Display for JwtTokenInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "alg    : {}", self.alg)?;
    writeln!(f, "type   : {}", self.typ)?;
    return writeln!(f, "kid    : {}", self.kid);
  }
}

#[derive(Debug, Deserialize)]
struct JwtUserInfo {
  #[serde(rename(deserialize = "https://api.openai.com/profile"))]
  profile: JwtProfileInfo,
  #[serde(rename(deserialize = "https://api.openai.com/auth"))]
  auth: JwtAuthInfo,
  iss: String,
  sub: String,
  aud: Vec<String>,
  iat: i64,
  exp: i64,
  azp: Option<String>,
  scope: Option<String>
}

impl Display for JwtUserInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "email  : {}", self.profile.email)?;
    writeln!(f, "user_id: {}", self.auth.user_id)?;
    writeln!(f, "iss    : {}", self.iss)?;
    writeln!(f, "sub    : {}", self.sub)?;
    writeln!(f, "aud    : {}", self.aud.concat())?;
    let iat_time = NaiveDateTime::from_timestamp_opt(self.iat, 0);
    if let Some(iat) = iat_time {
      writeln!(f, "iat    : {}", iat)?;
    }
    let exp_time = NaiveDateTime::from_timestamp_opt(self.exp, 0);
    if let Some(exp) = exp_time {
      writeln!(f, "exp    : {}", exp)?;
    }
    if let Some(azp) = &self.azp {
      writeln!(f, "azp    : {}", azp)?;
    }
    if let Some(scope) = &self.scope {
      writeln!(f, "scope  : {}", scope)?;
    }
    return Ok(());
  }
}

#[derive(Debug, Deserialize)]
struct JwtProfileInfo {
  email: String,
  #[serde(rename(deserialize = "email_verified"))]
  verified: bool
}

#[derive(Debug, Deserialize)]
struct JwtAuthInfo {
  user_id: String
}