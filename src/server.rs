use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};
use warp::reject::Reject;
use warp::reply::Json;
use crate::common::ApplicationParam;
use crate::openai::{OpenAiAccessRes, OpenAiClient, OpenAiRefreshRes};

pub fn server(param: &ApplicationParam) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
  return access_route(param.proxy.clone()).or(refresh_route(param.proxy.clone()));
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

async fn access_handler( req: AccessTokenReq, client: OpenAiClient) -> Result<Json, Rejection> {
  let state = (&client).get_state().await
    .map_err(|err| warp::reject::custom(AnyhowRejection(err)))?;
  let code = (&client).get_code(&state, &req.username, &req.password).await
    .map_err(|err| warp::reject::custom(AnyhowRejection(err)))?;
  let res = (&client).access_token(&code).await
    .map_err(|err| warp::reject::custom(AnyhowRejection(err)))?;
  return Ok(warp::reply::json(&AccessTokenRes::create(res)));
}

async fn refresh_handler( req: RefreshTokenReq, client: OpenAiClient) -> Result<Json, Rejection> {
  let res = (&client).refresh_token(&req.refresh).await
    .map_err(|err| warp::reject::custom(AnyhowRejection(err)))?;
  return Ok(warp::reply::json(&RefreshTokenRes::create(res)));
}

#[derive(Debug)]
struct AnyhowRejection(anyhow::Error);

impl Reject for AnyhowRejection {}

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