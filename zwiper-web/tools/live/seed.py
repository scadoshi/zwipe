"""Seeds a throwaway user on the local zerver: verified, decks with commanders and cards,
skips, a removal, a year of daily activity, and a maybeboard. Prints the username."""
import json, os, random, subprocess, sys, urllib.request

API = "http://localhost:3000"
DB = os.environ["DATABASE_URL"]
NAME = sys.argv[1]
PASSWORD = "Zw!pe-Web-Test-42"

def call(method, path, body=None, token=None):
    req = urllib.request.Request(API + path, method=method, data=json.dumps(body).encode() if body is not None else None)
    req.add_header("Content-Type", "application/json")
    if token: req.add_header("Authorization", "Bearer " + token)
    try:
        with urllib.request.urlopen(req) as r:
            raw = r.read()
            return json.loads(raw) if raw else None
    except urllib.error.HTTPError as e:
        print("HTTP", e.code, method, path, e.read()[:300].decode(errors="replace"))
        raise

def sql(q):
    return subprocess.run(["psql", DB, "-Atc", q], capture_output=True, text=True, check=True).stdout.strip()

def card(name):
    row = sql(f"select id, oracle_id from scryfall_data where name = $${name}$$ and lang='en' and image_uris is not null order by released_at desc limit 1")
    if not row:
        row = sql(f"select id, oracle_id from scryfall_data where name = $${name}$$ and lang='en' order by released_at desc limit 1")
    sid, oid = row.split("|")
    return sid, oid

call("POST", "/api/auth/register", {"username": NAME, "email": f"{NAME}@example.com", "password": PASSWORD})
sql(f"update users set email_verified_at = now() where username = '{NAME}'")
session = call("POST", "/api/auth/login", {"identifier": NAME, "password": PASSWORD})
tok = session["access_token"]["value"]
uid = session["user"]["id"]

decks = {
    "Goblin Horde": ("Krenko, Mob Boss", ["Goblin Bombardment", "Skirk Prospector", "Goblin Chieftain", "Purphoros, God of the Forge", "Impact Tremors", "Lightning Bolt", "Chaos Warp", "Sol Ring", "Arcane Signet", "Mountain"]),
    "Spellslinger": ("Talrand, Sky Summoner", ["Counterspell", "Brainstorm", "Ponder", "Preordain", "Rhystic Study", "Cyclonic Rift", "Sol Ring", "Arcane Signet", "Swords to Plowshares", "Island"]),
    "Graveyard": ("Meren of Clan Nel Toth", ["Eternal Witness", "Sakura-Tribe Elder", "Viscera Seer", "Reassembling Skeleton", "Sol Ring", "Arcane Signet", "Cultivate", "Kodama's Reach", "Demonic Tutor", "Forest"]),
    "Counters": ("Atraxa, Praetors' Voice", ["Doubling Season", "Deepglow Skate", "Sol Ring", "Arcane Signet", "Cultivate", "Counterspell", "Swords to Plowshares", "Rhystic Study", "Demonic Tutor", "Plains"]),
}
deck_ids = []
for name, (commander, cards) in decks.items():
    csid, _ = card(commander)
    d = call("POST", "/api/deck", {"name": name, "commander_id": csid, "format": "commander"}, tok)
    did = d["id"]; deck_ids.append(did)
    text = "\n".join(f"1 {c}" for c in cards)
    call("POST", f"/api/deck/{did}/card/import", {"text": text, "mode": "add"}, tok)

skips = ["Ancient Tomb", "Mana Crypt", "Smothering Tithe", "Esper Sentinel", "Dockside Extortionist", "Fierce Guardianship", "Deflecting Swat", "Jeweled Lotus"]
for did in deck_ids[:2]:
    for s in skips:
        _, oid = card(s)
        call("POST", f"/api/deck/{did}/suppressions", {"oracle_id": oid}, tok)
for did in deck_ids:
    for a in ["Command Tower", "Swiftfoot Boots", "Lightning Greaves"]:
        sid, oid = card(a)
        try:
            call("POST", f"/api/deck/{did}/card", {"scryfall_data_id": sid, "oracle_id": oid, "quantity": 1}, tok)
        except Exception:
            pass
sid, _ = card("Lightning Greaves")
call("DELETE", f"/api/deck/{deck_ids[0]}/card/{sid}", None, tok)

for m in ["Edgar Markov", "The Ur-Dragon", "Kenrith, the Returned King"]:
    _, oid = card(m)
    call("POST", f"/api/user/commander-maybeboard/{oid}", None, tok)

random.seed(7)
rows = []
for back in range(0, 365):
    if random.random() < 0.45:
        n = random.randint(3, 60) * (6 if random.random() < 0.04 else 1)
        rows.append(f"('{uid}', (now() at time zone 'UTC')::date - {back}, {n}, {n//2}, {n//6}, {n//10}, {random.randint(0, 8)})")
sql("insert into user_daily_activity (user_id, day, swipes_right, swipes_left, swipes_up, swipes_down, searches) values " + ",".join(rows) +
    " on conflict (user_id, day) do update set swipes_right = excluded.swipes_right, swipes_left = excluded.swipes_left, swipes_up = excluded.swipes_up, swipes_down = excluded.swipes_down, searches = excluded.searches")
sql(f"update user_lifetime_counters set swipes_right = (select sum(swipes_right) from user_daily_activity where user_id='{uid}'), swipes_left = (select sum(swipes_left) from user_daily_activity where user_id='{uid}'), swipes_up = (select sum(swipes_up) from user_daily_activity where user_id='{uid}'), swipes_down = (select sum(swipes_down) from user_daily_activity where user_id='{uid}'), searches = (select sum(searches) from user_daily_activity where user_id='{uid}') where user_id='{uid}'")
print("seeded", NAME, uid)
