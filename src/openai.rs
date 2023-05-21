use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use anyhow::{anyhow, bail};
use clap::Parser;
use chrono::NaiveDateTime;
use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use reqwest::{Client, Proxy, Url};
use reqwest::redirect::Policy;
use serde::{Serialize, Deserialize};

const HEADER_AGENT: &'static str = "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/113.0";
const HEADER_REFERER: &'static str = "https://ios.chat.openai.com/";
const IOS_CLIENT_ID: &'static str = "pdlLIX2Y72MIl2rhLhTE9VV9bN905kBh";

pub struct OpenAiClient {
  client: Client,
}

impl OpenAiClient {
  pub fn create(proxy: Option<Proxy>) -> anyhow::Result<OpenAiClient> {
    let mut headers = HeaderMap::with_capacity(2);
    headers.insert(USER_AGENT, HeaderValue::from_static(HEADER_AGENT));
    headers.insert(REFERER, HeaderValue::from_static(HEADER_REFERER));
    let mut client = Client::builder()
      .default_headers(headers)
      .cookie_store(true)
      .redirect(Policy::none());
    if let Some(proxy) = proxy {
      client = client.proxy(proxy);
    }
    let client = client.build().map_err(|err| anyhow!("Create Client Error: {}", err))?;
    return Ok(OpenAiClient { client });
  }

  pub async fn get_state(&self) -> anyhow::Result<String> {
    let res = self.client.get("https://auth0.openai.com/authorize")
      .query(&[("client_id", IOS_CLIENT_ID)])
      .query(&[("audience", "https://api.openai.com/v1")])
      .query(&[("redirect_uri", "com.openai.chat://auth0.openai.com/ios/com.openai.chat/callback")])
      .query(&[("scope", "openid email profile offline_access model.request model.read organization.read offline")])
      .query(&[("response_type", "code")])
      .query(&[("code_challenge", "t1RM5eR6dToh4VAe85qAf4ANdsnob6ANiuyl_z67mr4")])
      .query(&[("code_challenge_method", "S256")])
      .query(&[("prompt", "login")])
      .send().await
      .map_err(|err| anyhow!("Get State Error: {}", err))?;
    let location = res.headers().get("location")
      .ok_or(anyhow!("Get State Error: location is none"))?
      .to_str()
      .map_err(|err| anyhow!("Get State Error: {}", err))?;
    let url = Url::options().base_url(Some(&Url::parse("https://api.openai.com/").unwrap()))
      .parse(location)
      .map_err(|err| anyhow!("Get State Error: {}", err))?;
    let (_, state) = url.query_pairs()
      .filter(|(k, _)| k.eq_ignore_ascii_case("state"))
      .next()
      .ok_or(anyhow!("Get State Error: state is none"))?;
    return Ok(state.to_string());
  }

  pub async fn get_code(&self, state: &str, username: &str, password: &str) -> anyhow::Result<String> {
    let _ = self.client.post("https://auth0.openai.com/u/login/identifier")
      .query(&[("state", state)])
      .form(&PasswordLoginReq01::create(state, username))
      .send().await
      .map_err(|err| anyhow!("Get Code1 Error: {}", err))?;
    let res2 = self.client.post("https://auth0.openai.com/u/login/password")
      .json(&PasswordLoginReq02::create(state, username, password))
      .send().await
      .map_err(|err| anyhow!("Get Code2 Error: {}", err))?;
    let location2 = res2.headers().get("location")
      .ok_or(anyhow!("Get Code2 Error: location is none"))?
      .to_str()
      .map_err(|err| anyhow!("Get Code2 Error: {}", err))?;
    let res3 = self.client.get(format!("https://auth0.openai.com{}", location2))
      .send().await
      .map_err(|err| anyhow!("Get Code3 Error: {}", err))?;
    let location3 = res3.headers().get("location")
      .ok_or(anyhow!("Get Code3 Error: location is none"))?
      .to_str()
      .map_err(|err| anyhow!("Get Code3 Error: {}", err))?;
    let queries = Url::parse(location3)
      .map_err(|err| anyhow!("Get Code3 Error: {}", err))?;
    let (_, code) = queries.query_pairs()
      .filter(|(k, _)| k.eq_ignore_ascii_case("code"))
      .next()
      .ok_or(anyhow!("Get Code3 Error: code is none"))?;
    return Ok(code.to_string());
  }

  pub async fn access_token(&self, code: &str) -> anyhow::Result<(String, String)> {
    let param = AccessTokenReq::create(code);
    let res = self.client.post("https://auth0.openai.com/oauth/token")
      .json(&param)
      .send().await
      .map_err(|err| anyhow!("Access Token Error: {}", err))?;
    if !res.status().is_success() {
      bail!("Access Token Error: {}", res.status());
    }
    return res.json::<AccessTokenRes>().await
      .map_err(|err| anyhow!("Access Token Error: {}", err))
      .map(|r| (r.access_token.clone(), r.refresh_token.clone()));
  }

  pub async fn refresh_token(&self, token: &str) -> anyhow::Result<String> {
    let param = RefreshTokenReq::create(token);
    let res = self.client.post("https://auth0.openai.com/oauth/token")
      .json(&param)
      .send().await
      .map_err(|err| anyhow!("Refresh Token Error: {}", err))?;
    if !res.status().is_success() {
      bail!("Refresh Token Error: {}", res.status());
    }
    return res.json::<RefreshTokenRes>().await
      .map_err(|err| anyhow!("Refresh Token Error: {}", err))
      .map(|r| r.access_token());
  }

}

impl OpenAiClient {

  pub fn parse_token(token_str: &str) -> anyhow::Result<(Option<JwtTokenInfo>, Option<JwtUserInfo>)> {
    let segments = token_str.split('.').collect::<Vec<&str>>();
    let token_info = match segments.get(0) {
      None => None,
      Some(&value) => {
        let value = base64_str_decode(value)
          .map_err(|_| anyhow!("Parse Token Info Error: {}", value))?;
        Some(serde_json::from_str::<JwtTokenInfo>(&value)?)
      }
    };
    let user_info = match segments.get(1) {
      None => None,
      Some(&value) => {
        let value = base64_str_decode(value)
          .map_err(|_| anyhow!("Parse User Info Error: {}", value))?;
        Some(serde_json::from_str::<JwtUserInfo>(&value)?)
      }
    };
    return Ok((token_info, user_info));
  }
}

fn base64_str_decode(base64_str: &str) -> anyhow::Result<String> {
  let result = base64::decode(base64_str)
    .map_err(|_| anyhow!("Base64 Decode Error: {}", base64_str))?;
  return Ok(String::from_utf8(result).map_err(|_| anyhow!("Base64 String Error: {}", base64_str))?);
}

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

  #[arg(long = "server", help = "start server mode", help = "127.0.0.1:8000")]
  pub server: Option<SocketAddr>,

}

impl ApplicationParam {
  fn parse_proxy(proxy: &str) -> anyhow::Result<Proxy> {
    return Proxy::all(proxy).map_err(|err| anyhow!("Create Proxy Error: {}", err));
  }
}

#[derive(Debug, Serialize)]
struct PasswordLoginReq01<'a> {
  state: &'a str,
  username: &'a str,
}

impl<'a> PasswordLoginReq01<'a> {
  fn create(state: &'a str, username: &'a str) -> PasswordLoginReq01<'a> {
    return PasswordLoginReq01 {
      state,
      username,
    };
  }
}

#[derive(Debug, Serialize)]
struct PasswordLoginReq02<'a> {
  state: &'a str,
  username: &'a str,
  password: &'a str,
}

impl<'a> PasswordLoginReq02<'a> {
  fn create(state: &'a str, username: &'a str, password: &'a str) -> PasswordLoginReq02<'a> {
    return PasswordLoginReq02 {
      state,
      username,
      password,
    };
  }
}

#[derive(Debug, Serialize)]
struct AccessTokenReq<'a> {
  redirect_uri: &'a str,
  client_id: &'a str,
  grant_type: &'a str,
  code: &'a str,
  code_verifier: &'a str,
}

impl AccessTokenReq<'_> {
  fn create(code: &str) -> AccessTokenReq {
    return AccessTokenReq {
      redirect_uri: "com.openai.chat://auth0.openai.com/ios/com.openai.chat/callback",
      grant_type: "authorization_code",
      client_id: IOS_CLIENT_ID,
      code,
      code_verifier: "IkrrBD89CBmwwzM-csfBnWKLMan5uE7laCMd2YTcPWE",
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
  token_type: String,
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
  grant_type: &'a str,
}

impl RefreshTokenReq<'_> {
  fn create(token: &str) -> RefreshTokenReq {
    return RefreshTokenReq {
      refresh_token: token,
      client_id: IOS_CLIENT_ID,
      grant_type: "refresh_token",
    };
  }
}

#[derive(Debug, Deserialize)]
struct RefreshTokenRes {
  access_token: String,
  id_token: String,
  scope: String,
  expires_in: u32,
  token_type: String,
}

impl RefreshTokenRes {
  fn access_token(&self) -> String {
    return self.access_token.clone();
  }
}

#[derive(Debug, Deserialize)]
pub struct JwtTokenInfo {
  alg: String,
  typ: String,
  kid: String,
}

impl Display for JwtTokenInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "alg   : {}", self.alg)?;
    writeln!(f, "type  : {}", self.typ)?;
    return writeln!(f, "kid   : {}", self.kid);
  }
}

#[derive(Debug, Deserialize)]
pub struct JwtUserInfo {
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
  scope: Option<String>,
}

impl Display for JwtUserInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "email : {}", self.profile.email)?;
    writeln!(f, "user  : {}", self.auth.user_id)?;
    writeln!(f, "iss   : {}", self.iss)?;
    writeln!(f, "sub   : {}", self.sub)?;
    writeln!(f, "aud   : {}", self.aud.concat())?;
    let iat_time = NaiveDateTime::from_timestamp_opt(self.iat, 0);
    if let Some(iat) = iat_time {
      writeln!(f, "iat   : {}", iat)?;
    }
    let exp_time = NaiveDateTime::from_timestamp_opt(self.exp, 0);
    if let Some(exp) = exp_time {
      writeln!(f, "exp   : {}", exp)?;
    }
    if let Some(azp) = &self.azp {
      writeln!(f, "azp   : {}", azp)?;
    }
    if let Some(scope) = &self.scope {
      writeln!(f, "scope : {}", scope)?;
    }
    return Ok(());
  }
}

#[derive(Debug, Deserialize)]
struct JwtProfileInfo {
  email: String,
  #[serde(rename(deserialize = "email_verified"))]
  verified: bool,
}

#[derive(Debug, Deserialize)]
struct JwtAuthInfo {
  user_id: String,
}