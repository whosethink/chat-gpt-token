[中文](./README.md) | [English](./README-en.md)

### Tool For OpenAi Token

1. use account and password, to get, refresh and parse token of openai
2. lower quality requirements for request's ip, no rate limit
3. directly request auth0.openai.com

```text
Usage: chat-gpt-token [OPTIONS]

Options:
  -u, --username <USERNAME>            
  -p, --password <PASSWORD>            
      --access-token <ACCESS_TOKEN>    parse access token
      --refresh-token <REFRESH_TOKEN>  refresh access token
      --proxy <PROXY>                  like socks5://127.0.0.1:8080
  -h, --help                           Print help
  -V, --version                        Print version
```

### Usage Example (following Token is just for display)

#### 1. Get Token
```shell
./chat-gpt-token -u your_account -p your_password
```
```text
Access Token: eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ik1UaEVOVUpHTkVNMVFURTRNMEZCTWpkQ05UZzVNRFUxUlRVd1FVSkRNRU13UmtGRVFrRXpSZyJ9.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL3Byb2ZpbGUiOnsiZW1haWwiOiJ5b3VyX2FjY291bnQiLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZX0sImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6eyJ1c2VyX2lkIjoidXNlci1wWmVKODJwUlpXM3kxbTNRV21wUmsyaHROcCJ9LCJpc3MiOiJodHRwczovL2F1dGgwLm9wZW5haS5jb20vIiwic3ViIjoiYXV0aDB8NjYyMWNiNThhOGIyYjk4OTVlYTE4Y2Q4IiwiYXVkIjpbImh0dHBzOi8vYXBpLm9wZW5haS5jb20vdjEiLCJodHRwczovL29wZW5haS5vcGVuYWkuYXV0aDBhcHAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTY4NDU4NjcwMiwiZXhwIjoxNjg1Nzk4NDAyLCJhenAiOiJwZGxMSVgyWTcyTUlsMnJoTGhURTlWVjliTjkwNWtCaCIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgbW9kZWwucmVhZCBtb2RlbC5yZXF1ZXN0IG9yZ2FuaXphdGlvbi5yZWFkIG9mZmxpbmVfYWNjZXNzIn0K.NOE5GjjMQ2I_jrPf-v0QDf8nX3SXav5YCWjJQ19xnFYmFbpkPC16fXLVxf_kCJ3ge_-fo5GDoRLcSOSrNrM0qD6_0V25b1D5lHuWAr3cwhXcg7T0rF6weL7tLck1OvZeOYKmMaPewd8LHFpW89nuGZtydDfNhLtTODmvpB01IOgy73JG20Olbxr8Wel00r5GhKm7jB-Xqq8OUompZkQLp0cyPCAIN0yAI3Y85Um53I_kYRj--ffTz8Nw5hPl1Y0EDg7BPq6x1C9jHuFy-bYyQ3N5h7P84Xpk1EcxfSvcGj91pkpKeDAM3kwIZn5eeRebNayPOgJgIt5Vu6VSb895rw

Refresh Token: d7bee5010bcdc336374ee64176807fde5213fffb61300
```

#### 2. Refresh Token
```shell
./chat-gpt-token --refresh-token d7bee5010bcdc336374ee64176807fde5213fffb61300
```
```text
Access Token: eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ik1UaEVOVUpHTkVNMVFURTRNMEZCTWpkQ05UZzVNRFUxUlRVd1FVSkRNRU13UmtGRVFrRXpSZyJ9.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL3Byb2ZpbGUiOnsiZW1haWwiOiJ5b3VyX2FjY291bnQiLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZX0sImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6eyJ1c2VyX2lkIjoidXNlci1wWmVKODJwUlpXM3kxbTNRV21wUmsyaHROcCJ9LCJpc3MiOiJodHRwczovL2F1dGgwLm9wZW5haS5jb20vIiwic3ViIjoiYXV0aDB8NjYyMWNiNThhOGIyYjk4OTVlYTE4Y2Q4IiwiYXVkIjpbImh0dHBzOi8vYXBpLm9wZW5haS5jb20vdjEiLCJodHRwczovL29wZW5haS5vcGVuYWkuYXV0aDBhcHAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTY4NDU4NjcwMiwiZXhwIjoxNjg1Nzk4NDAyLCJhenAiOiJwZGxMSVgyWTcyTUlsMnJoTGhURTlWVjliTjkwNWtCaCIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgbW9kZWwucmVhZCBtb2RlbC5yZXF1ZXN0IG9yZ2FuaXphdGlvbi5yZWFkIG9mZmxpbmVfYWNjZXNzIn0K.NOE5GjjMQ2I_jrPf-v0QDf8nX3SXav5YCWjJQ19xnFYmFbpkPC16fXLVxf_kCJ3ge_-fo5GDoRLcSOSrNrM0qD6_0V25b1D5lHuWAr3cwhXcg7T0rF6weL7tLck1OvZeOYKmMaPewd8LHFpW89nuGZtydDfNhLtTODmvpB01IOgy73JG20Olbxr8Wel00r5GhKm7jB-Xqq8OUompZkQLp0cyPCAIN0yAI3Y85Um53I_kYRj--ffTz8Nw5hPl1Y0EDg7BPq6x1C9jHuFy-bYyQ3N5h7P84Xpk1EcxfSvcGj91pkpKeDAM3kwIZn5eeRebNayPOgJgIt5Vu6VSb895rw
```

#### 3. Parse Token
```shell
./chat-gpt-token --access-token eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ik1UaEVOVUpHTkVNMVFURTRNMEZCTWpkQ05UZzVNRFUxUlRVd1FVSkRNRU13UmtGRVFrRXpSZyJ9.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL3Byb2ZpbGUiOnsiZW1haWwiOiJ5b3VyX2FjY291bnQiLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZX0sImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6eyJ1c2VyX2lkIjoidXNlci1wWmVKODJwUlpXM3kxbTNRV21wUmsyaHROcCJ9LCJpc3MiOiJodHRwczovL2F1dGgwLm9wZW5haS5jb20vIiwic3ViIjoiYXV0aDB8NjYyMWNiNThhOGIyYjk4OTVlYTE4Y2Q4IiwiYXVkIjpbImh0dHBzOi8vYXBpLm9wZW5haS5jb20vdjEiLCJodHRwczovL29wZW5haS5vcGVuYWkuYXV0aDBhcHAuY29tL3VzZXJpbmZvIl0sImlhdCI6MTY4NDU4NjcwMiwiZXhwIjoxNjg1Nzk4NDAyLCJhenAiOiJwZGxMSVgyWTcyTUlsMnJoTGhURTlWVjliTjkwNWtCaCIsInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwgbW9kZWwucmVhZCBtb2RlbC5yZXF1ZXN0IG9yZ2FuaXphdGlvbi5yZWFkIG9mZmxpbmVfYWNjZXNzIn0K.NOE5GjjMQ2I_jrPf-v0QDf8nX3SXav5YCWjJQ19xnFYmFbpkPC16fXLVxf_kCJ3ge_-fo5GDoRLcSOSrNrM0qD6_0V25b1D5lHuWAr3cwhXcg7T0rF6weL7tLck1OvZeOYKmMaPewd8LHFpW89nuGZtydDfNhLtTODmvpB01IOgy73JG20Olbxr8Wel00r5GhKm7jB-Xqq8OUompZkQLp0cyPCAIN0yAI3Y85Um53I_kYRj--ffTz8Nw5hPl1Y0EDg7BPq6x1C9jHuFy-bYyQ3N5h7P84Xpk1EcxfSvcGj91pkpKeDAM3kwIZn5eeRebNayPOgJgIt5Vu6VSb895rw
```
```text
alg    : RS256
type   : JWT
kid    : MThENUJGNEM1QTE4M0FBMjdCNTg5MDU1RTUwQUJDMEMwRkFEQkEzRg
email  : your_account
user_id: user-pZeJ82pRZW3y1m3QWmpRk2htNp
iss    : https://auth0.openai.com/
sub    : auth0|6621cb58a8b2b9895ea18cd8
aud    : https://api.openai.com/v1https://openai.openai.auth0app.com/userinfo
iat    : 2023-05-20 12:45:02
exp    : 2023-06-03 13:20:02
azp    : pdlLIX2Y72MIl2rhLhTE9VV9bN905kBh
scope  : openid profile email model.read model.request organization.read offline_access
```

#### 4. With Proxy
```shell
./chat-gpt-token --proxy socks5://127.0.0.1:1080 --refresh-token d7bee5010bcdc336374ee64176807fde5213fffb61300
```