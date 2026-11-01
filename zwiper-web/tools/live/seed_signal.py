"""Gives the throwaway user card signals and weekly outcomes, so Activity's rates and Most skipped fill."""
import os, subprocess, sys
DB = os.environ["DATABASE_URL"]; NAME = open('user.txt').read().strip()
def sql(q): return subprocess.run(["psql", DB, "-Atc", q], capture_output=True, text=True, check=True).stdout.strip()
uid = sql(f"select id from users where username='{NAME}'")
cmd = sql("select oracle_id from latest_cards where name='Krenko, Mob Boss' limit 1")
sql(f"""insert into user_card_signal (user_id, commander_oracle_id, card_oracle_id, shown, added, skipped, maybed, removed)
select '{uid}', '{cmd}', lc.oracle_id, 20, (random()*6)::int, (random()*12)::int, (random()*3)::int, (random()*2)::int
from latest_cards lc join card_profiles cp on cp.scryfall_data_id = lc.id
where cp.card_roles is not null and jsonb_array_length(cp.card_roles) > 0 and lc.oracle_id is not null
order by md5(lc.oracle_id::text) limit 400 on conflict do nothing""")
sql(f"""insert into user_week_signal (user_id, week_start, swipes_right, swipes_left, swipes_up, swipes_down, searches, added, skipped, maybed, removed)
select '{uid}', date_trunc('week', now())::date - 7*g, 50, 80, 10, 5, 12, 40, 70, 9, 6 from generate_series(0, 20) g on conflict do nothing""")
print(sql(f"select count(*) from user_card_signal where user_id='{uid}'"))
