import json, sys
from lib import *
import ids
d = ids.decks(); g = d['Goblin Horde']
which = sys.argv[1] if len(sys.argv) > 1 else 'all'
dark = js("document.documentElement.setAttribute('data-theme-mode','dark'); true")
pub = [wait(6), shot('login')] + [js("history.pushState({},'','/register'); dispatchEvent(new PopStateEvent('popstate')); true"), wait(2), shot('register'),
  js("history.pushState({},'','/forgot-password'); dispatchEvent(new PopStateEvent('popstate')); true"), wait(2), shot('forgot-password'),
  js("history.pushState({},'','/login'); dispatchEvent(new PopStateEvent('popstate')); true"), wait(2)]
screens = {
 'home': '/', 'activity': '/activity', 'playstyle': '/playstyle', 'deck-view': f'/decks/{g}',
 'account': '/account', 'config': '/settings', 'privacy': '/privacy', 'changelog': '/changelog', 'decks': '/decks',
 'create': '/decks/new', 'edit': f'/decks/{g}/edit', 'import': f'/decks/{g}/import', 'export': f'/decks/{g}/export',
 'maybeboard': '/maybeboard', 'add': f'/decks/{g}/add', 'deck-cards': f'/decks/{g}/cards',
 'remove': f'/decks/{g}/remove', 'not-found': '/nope/nowhere',
}
pick = list(screens) if which == 'all' else which.split(',')
steps = (pub if which == 'all' or 'login' in pick else [wait(6)]) + login[1:]
for name in pick:
    if name in ('login',): continue
    steps += nav(screens[name]) + [wait(4), gotit, wait(1), js("document.querySelector('.screen-content')?.scrollTo(0,0); true"), shot(name)]
json.dump(steps, open('s_all.json','w'))
