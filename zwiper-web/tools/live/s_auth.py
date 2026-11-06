import json, sys, time
from lib import *
mode = sys.argv[1]
go = lambda p: js(f"history.pushState({{}},'','{p}'); dispatchEvent(new PopStateEvent('popstate')); true")
light = js("(()=>{const e=Array.from(document.querySelectorAll('[class]')).find(x=>/theme-[a-z0-9-]+-(dark|light)/.test(x.className)); const m=e.className.match(/theme-([a-z0-9-]+)-(dark|light)/); localStorage.setItem('zwiper-web.theme', JSON.stringify({name:m[1], is_dark:false})); location.reload(); return m[0]})()")
def fill(id_, text): return [focus('#'+id_), js(f"document.getElementById('{id_}').select(); true"), typ(text)]
errs = js("Array.from(document.querySelectorAll('.web-form-panel *')).filter(e=>e.children.length===0 && e.offsetParent && /error|must|invalid|least|match/i.test(e.className+e.textContent)).map(e=>e.textContent.trim()).join(' | ')")
submit = js("(()=>{const b=Array.from(document.querySelectorAll('.web-form-actions button, .action-bar button')).filter(b=>b.offsetParent!==null).find(b=>/^(Create profile|Send reset link)$/.test(b.textContent.trim())); if(!b) return 'no submit'; b.click(); return 'submitted'})()")
if mode == 'reg-shots':
    steps = [wait(6), go('/register'), wait(3), shot('register')]
elif mode == 'reg-light':
    steps = [wait(6), light, wait(5), go('/register'), wait(3), shot('register-light')]
elif mode == 'reg-flow':
    name = 'webzr' + str(int(time.time()))[-5:]
    open('reg_user.txt','w').write(name)
    steps = [wait(6), observe, click('Create profile'), wait(3)] + fill('username','a') + fill('email','nope') + fill('password','short') + fill('confirm_password','other') + [submit, wait(1.5), errs, shot('register-errors'),
      js("(()=>{const b=Array.from(document.querySelectorAll('.web-form-panel button')).find(b=>b.textContent.trim()==='?'); if(!b) return 'no ?'; b.click(); return 'hint'})()"), wait(1.5), js("document.querySelector('[role=dialog], .dialog, .alert-dialog')?.innerText.slice(0,200)"), shot('register-hint'), gotit, wait(1)] \
      + fill('username', name) + fill('email', name + '@example.com') + fill('password','Zw!pe-Web-Test-42') + fill('confirm_password','Zw!pe-Web-Test-42') + [submit, wait(5), toasts, js("location.pathname + ' | identity=' + document.getElementById('identity')?.value"), shot('register-done')]
elif mode == 'fp-shots':
    steps = [wait(6), go('/forgot-password'), wait(3), shot('forgot')]
elif mode == 'fp-light':
    steps = [wait(6), light, wait(5), go('/forgot-password'), wait(3), shot('forgot-light')]
elif mode == 'fp-flow':
    steps = [wait(6), click('Forgot password'), wait(3)] + fill('email','bad') + [submit, wait(1.5), errs, shot('forgot-error')] + fill('email', 'nobody-webz@example.com') + [submit, wait(4), js("document.querySelector('.web-outcome-panel')?.innerText"), shot('forgot-sent'), click('Back to login'), wait(2), js('location.pathname')]
json.dump(steps, open(f's_{mode}.json','w'))
