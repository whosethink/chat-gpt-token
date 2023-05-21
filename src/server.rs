use std::convert::Infallible;
use anyhow::anyhow;
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};
use warp::reply::Json;
use crate::common::ApplicationParam;
use crate::openai::{OpenAiAccessRes, OpenAiClient, OpenAiRefreshRes};

pub fn server(param: &ApplicationParam) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
  return access_route(param.proxy.clone()).or(refresh_route(param.proxy.clone())).recover(handle_rejection);
}

fn access_route(proxy: Option<Proxy>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
  let client = OpenAiClient::create(proxy, true).unwrap();
  return warp::path("access")
    .and(warp::post())
    .and(warp::body::json::<AccessTokenReq>())
    .and(warp::any().map(move || client.clone()))
    .and_then(access_handler);
}

fn refresh_route(proxy: Option<Proxy>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
  let client = OpenAiClient::create(proxy, false).unwrap();
  return warp::path("refresh")
    .and(warp::post())
    .and(warp::body::json::<RefreshTokenReq>())
    .and(warp::any().map(move || client.clone()))
    .and_then(refresh_handler);
}

async fn access_handler( req: AccessTokenReq, client: OpenAiClient) -> Result<Json, Infallible> {
  let state = (&client).get_state().await;
  if state.is_err() {
    return Ok(warp::reply::json(&CommonRes::anyhow(state)));
  }
  let code = (&client).get_code(&state.unwrap(), &req.username, &req.password).await;
  if code.is_err() {
    return Ok(warp::reply::json(&CommonRes::anyhow(code)));
  }
  let access = (&client).access_token(&code.unwrap()).await
    .map(|value| AccessTokenRes::create(value));
  return Ok(warp::reply::json(&CommonRes::anyhow(access)));
}

async fn refresh_handler( req: RefreshTokenReq, client: OpenAiClient) -> Result<Json, Infallible> {
  let res = (&client).refresh_token(&req.refresh).await
    .map(|value| RefreshTokenRes::create(value));
  return Ok(warp::reply::json(&CommonRes::anyhow(res)));
}

async fn handle_rejection(err: Rejection) -> Result<Json, Rejection> {
  let result = if true { Err(anyhow!(format!("Error: {:?}", err))) } else { Ok(()) };
  return Ok(warp::reply::json(&CommonRes::anyhow(result)));
}

#[derive(Debug, Deserialize)]
struct AccessTokenReq {
  username: String,
  password: String,
}

#[derive(Debug, Serialize)]
struct AccessTokenRes {
  access: String,
  refresh: String
}

impl AccessTokenRes {
  fn create(res: OpenAiAccessRes) -> AccessTokenRes {
    return AccessTokenRes {
      access: res.access_token,
      refresh: res.refresh_token
    };
  }
}

#[derive(Debug, Deserialize)]
struct RefreshTokenReq {
  refresh: String,
}

#[derive(Debug, Serialize)]
struct RefreshTokenRes {
  access: String
}

impl RefreshTokenRes {
  fn create(res: OpenAiRefreshRes) -> RefreshTokenRes {
    return RefreshTokenRes {
      access: res.access_token
    };
  }
}

#[derive(Debug, Serialize)]
struct CommonRes<T: Serialize> {
  code: i32,
  message: String,
  data: Option<T>
}

const SUCCESS_MESSAGE: &'static str = "OK";

impl <T: Serialize> CommonRes<T> {
  fn success(data: T) -> CommonRes<T> {
    return CommonRes {
      code: 0,
      message: SUCCESS_MESSAGE.to_string(),
      data: Some(data)
    };
  }

  fn error(message: &str) -> CommonRes<T> {
    return CommonRes {
      code: -1,
      message: message.to_string(),
      data: None
    };
  }

  fn anyhow(res: anyhow::Result<T>) -> CommonRes<T> {
    return match res {
      Ok(data) => CommonRes::success(data),
      Err(err) => CommonRes::error(&err.to_string())
    }
  }
}