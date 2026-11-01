import json
u=open('user.txt').read().strip()
login=[["wait",6],["js","document.getElementById('identity').focus(); true"],["type",u],["js","document.getElementById('password').focus(); true"],["type","Zw!pe-Web-Test-42"],["js","document.querySelector('form').requestSubmit(); true"],["wait",6]]
gotit=["js","Array.from(document.querySelectorAll('button')).filter(b=>b.textContent.trim()==='Got it').forEach(b=>b.click()); true"]
def nav(path): return [["js",f"history.pushState({{}},'', '{path}'); window.dispatchEvent(new PopStateEvent('popstate')); true"],["wait",4],gotit,["wait",1]]
def click(text, sel='button', last=False, nth=None):
    pick = 'all[all.length-1]' if last else (f'all[{nth}]' if nth is not None else 'all[0]')
    return ["js",f"(()=>{{const t={json.dumps(text)}; const all=Array.from(document.querySelectorAll({json.dumps(sel)})).filter(b=>{{const x=b.textContent.trim(); return x===t || x.startsWith(t+' ') || x.startsWith(t+'\\u2197');}}).filter(b=>b.offsetParent!==null); const b={pick}; if(!b) return 'missing '+t; b.click(); return 'clicked '+t}})()"]
info=["js","location.pathname+' | '+(document.querySelector('.toast-container')?.innerText||'').replace(/\\n/g,' / ')"]
def js(x): return ["js",x]
def shot(n): return ["shot",n]
def wait(n): return ["wait",n]
def typ(t): return ["type",t]
def key(k): return ["key",k]
def focus(sel): return js(f"(()=>{{const e=document.querySelector({json.dumps(sel)}); if(!e) return 'nofocus'; e.focus(); return 'focused'}})()")
def dump(): return js("document.querySelector('.screen-content')?.innerText.slice(0,600)")
observe = js("window.__t=[]; new MutationObserver(()=>{document.querySelectorAll('.toast-tap').forEach(t=>{const x=t.innerText.replace(/\\n/g,' ').trim(); if(x && !t.__seen){t.__seen=1; window.__t.push(x)}})}).observe(document.body,{childList:true,subtree:true}); 'observing'")
toasts = js("(()=>{const x=window.__t||[]; window.__t=[]; return x.join(' | ')})()")
