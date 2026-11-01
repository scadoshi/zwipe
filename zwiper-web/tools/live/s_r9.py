import json, sys
from lib import *
import ids
g = ids.decks()['Goblin Horde']
def go(p): return js(f"history.pushState({{}},'','{p}'); dispatchEvent(new PopStateEvent('popstate')); true")
MEASURE = r"""(()=>{const s=document.querySelector('.web-scroll-screen'); if(!s) return 'no scroll screen';
const bar=s.querySelector(':scope > .action-bar'); const any=document.querySelector('.action-bar');
const hint=s.querySelector(':scope > .web-screen-hint'); const H=innerHeight; const out=[];
if(!bar){ out.push(any&&getComputedStyle(any).display!=='none' ? 'BAR NOT DIRECT CHILD' : 'no bar'); }
else if(getComputedStyle(bar).display==='none'){ out.push('bar hidden'); }
else { const b=bar.getBoundingClientRect(); out.push('bar bottom '+Math.round(b.bottom)+'/'+H+(Math.abs(b.bottom-H)>1?' OFF':''));
 if(hint){const h=hint.getBoundingClientRect(); const inside=h.top>=b.top-1&&h.bottom<=b.bottom+1&&h.right<=b.right+1;
  out.push('hint '+(inside?'in bar':'OUTSIDE bar')+' dy '+Math.round((h.top+h.bottom)/2-(b.top+b.bottom)/2)+' right gap '+Math.round(b.right-h.right));
  const btns=[...bar.querySelectorAll('button,a')].filter(x=>x.offsetParent); if(btns.length){const last=btns.map(x=>x.getBoundingClientRect()).reduce((a,c)=>c.right>a.right?c:a); out.push('hint overlap '+(last.right>h.left-2?'YES':'no'));}}
 s.scrollTo(0,s.scrollHeight); const c=s.querySelector(':scope > .screen-content');
 const lastEl=[...c.querySelectorAll('*')].filter(e=>e.offsetParent&&e.children.length===0&&e.getBoundingClientRect().height>0).map(e=>e.getBoundingClientRect().bottom).reduce((a,v)=>Math.max(a,v),0);
 const b2=bar.getBoundingClientRect(); out.push('end: last content '+Math.round(lastEl)+' bar top '+Math.round(b2.top)+(lastEl>b2.top+1?' HIDDEN':' ok')); s.scrollTo(0,0);}
return location.pathname+' | '+out.join(' | ');})()"""
m = js(MEASURE)
which = sys.argv[1]
screens = {'home':'/','activity':'/activity','playstyle':'/deck/suggested-commanders','deck-view':f'/deck/get/{g}','account':'/user','config':'/config','privacy':'/privacy','changelog':'/changelog','decks':'/deck','create':'/deck/create','edit':f'/deck/update/{g}','import':f'/deck/card/import/{g}','export':f'/deck/card/export/{g}','maybeboard':'/deck/commander-maybeboard','add':f'/deck/card/add/{g}','deck-cards':f'/deck/card/view/{g}','remove':f'/deck/card/remove/{g}','not-found':'/nope/nowhere'}
scroll = lambda f: js(f"(()=>{{const s=document.querySelector('.web-scroll-screen'); s.scrollTo(0,{f}); return s.scrollTop+'/'+s.scrollHeight}})()")
pub = [wait(6), shot('login'), go('/register'), wait(2), shot('register'),
  js("(()=>{const b=[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='?'&&b.offsetParent); if(!b) return 'no ?'; b.click(); return 'opened'})()"), wait(1), shot('register-hint'), gotit, wait(1),
  go('/forgot-password'), wait(2), shot('forgot-password'),
  go('/privacy'), wait(3), js("(()=>{const t=document.body.innerText; return 'privacy signed out: nav pill '+!![...document.querySelectorAll('nav a, nav button')].find(a=>a.textContent.trim()==='Privacy policy')+', fan content '+/Fan Content/i.test(t)})()"), m,
  go('/login'), wait(2)]
steps = pub + login[1:]
if which == 'shots':
    for n in ('deck-view','add'):
        steps += nav(screens[n]) + [wait(3), gotit, wait(1), scroll(0), shot(n), scroll(500), wait(1), shot(n+'-mid'), scroll(100000), wait(1), shot(n+'-end')]
else:
    for n,p in screens.items():
        steps += nav(p) + [wait(3), gotit, wait(1), m]
json.dump(steps, open(f's_r9_{which}.json','w'))
