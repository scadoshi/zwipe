import json, urllib.request
API="http://localhost:3000"; NAME=open('user.txt').read().strip()
def call(method, path, body=None, token=None):
    req = urllib.request.Request(API + path, method=method, data=json.dumps(body).encode() if body is not None else None)
    req.add_header("Content-Type", "application/json")
    if token: req.add_header("Authorization", "Bearer " + token)
    with urllib.request.urlopen(req) as r:
        raw = r.read(); return json.loads(raw) if raw else None
def decks():
    s = call("POST", "/api/auth/login", {"identifier": NAME, "password": "Zw!pe-Web-Test-42"})
    return {d["name"]: d["id"] for d in call("GET", "/api/deck", None, s["access_token"]["value"])}
