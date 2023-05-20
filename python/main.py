#!/bin/python3

import requests
from urllib.parse import urlparse
from urllib.parse import parse_qs


proxies = {'https': 'socks5://127.0.0.1:1020', 'http': 'socks5://127.0.0.1:1020'}
headers = {
  'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/113.0',
  'Referer': 'https://ios.chat.openai.com/',
}
client = requests.session()

def get_state():
  url = 'https://auth0.openai.com/authorize'
  params = {
    'client_id': 'pdlLIX2Y72MIl2rhLhTE9VV9bN905kBh',
    'audience': 'https://api.openai.com/v1',
    'redirect_uri': 'com.openai.chat://auth0.openai.com/ios/com.openai.chat/callback',
    'scope': 'openid email profile offline_access model.request model.read organization.read offline',
    'response_type': 'code',
    'code_challenge': 't1RM5eR6dToh4VAe85qAf4ANdsnob6ANiuyl_z67mr4',
    'code_challenge_method': 'S256',
    'prompt': 'login'
  }
  res = client.get(url, params=params, proxies=proxies)
  if res.status_code != 200:
    raise Exception("Get State Error:", res.status_code, res.reason)
  return parse_qs(urlparse(res.url).query).get('state')[0]


def get_code(state, username, password):
  url1 = 'https://auth0.openai.com/u/login/identifier'
  params = {"state": state}
  data1 = {
    "state": state,
    "username": username
  }
  res1 = client.post(url1, headers=headers, params=params, data=data1, proxies=proxies, allow_redirects=False)
  if res1.status_code != 302:
    raise Exception("Get Code Error:", res1.status_code, res1.reason)
  url2 = 'https://auth0.openai.com/u/login/password'
  data2 = {
    "state": state,
    "username": username,
    "password": password,
    "action": "default"
  }
  res2 = client.post(url2, headers=headers, params=params, data=data2, proxies=proxies, allow_redirects=False)
  if res2.status_code != 302:
    raise Exception("Get Code Error:", res2.status_code, res2.reason)
  url3 = "https://auth0.openai.com{}".format(res2.headers['Location'])
  res3 = client.get(url3, headers=headers, proxies=proxies, allow_redirects=False)
  if res3.status_code != 302:
    raise Exception("Get Code Error:", res3.status_code, res3.reason)
  return parse_qs(urlparse(res3.headers.get('Location')).query).get('code')[0]


# 以 ios 客户端的方式获取 access_token
def access_token(code):
  url = 'https://auth0.openai.com/oauth/token'
  data = {
    'redirect_uri': 'com.openai.chat://auth0.openai.com/ios/com.openai.chat/callback',
    'grant_type': 'authorization_code',
    'client_id': 'pdlLIX2Y72MIl2rhLhTE9VV9bN905kBh',
    'code': code,
    'code_verifier': 'IkrrBD89CBmwwzM-csfBnWKLMan5uE7laCMd2YTcPWE'
  }
  res = client.post(url, data=data, proxies=proxies)
  if res.status_code != 200:
    raise Exception("Access Token Error:", res.status_code, res.reason)
  result = res.json()
  print(result)
  return (result.get('access_token'), result.get('refresh_token'))


# 用 access_token 获取新的 access_token
def refresh_token(token):
  url = 'https://auth0.openai.com/oauth/token'
  data = {
    "refresh_token": token,
    "client_id":"pdlLIX2Y72MIl2rhLhTE9VV9bN905kBh",
    "grant_type":"refresh_token"
  }
  res = client.post(url, data=data, proxies=proxies)
  if res.status_code != 200:
    raise Exception("Refresh Token Error:", res.status_code, res.reason)
  print(res.json())
  return res.json().get('access_token')


if __name__ == '__main__':
  username = 'username'
  password = 'password'

  (access_token1, refresh_token1) = access_token(get_code(get_state(), username, password))
  print("Access Token1:", access_token1)
  access_token2 = refresh_token(refresh_token1)
  print("Access Token2:", access_token2)
