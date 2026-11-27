"""sRGB <-> OKLab/OKLCH, WCAG contrast, CVD simulation (Machado 2009, severity 1.0)."""
import math
def hex2rgb(h):
    h=h.lstrip('#'); return tuple(int(h[i:i+2],16)/255 for i in (0,2,4))
def rgb2hex(r,g,b):
    c=lambda x: max(0,min(255,round(x*255))); return '#%02x%02x%02x'%(c(r),c(g),c(b))
def lin(c): return c/12.92 if c<=0.04045 else ((c+0.055)/1.055)**2.4
def unlin(c): return 12.92*c if c<=0.0031308 else 1.055*c**(1/2.4)-0.055
def rgb2oklab(rgb):
    r,g,b=[lin(x) for x in rgb]
    l=0.4122214708*r+0.5363325363*g+0.0514459929*b
    m=0.2119034982*r+0.6806995451*g+0.1073969566*b
    s=0.0883024619*r+0.2817188376*g+0.6299787005*b
    l,m,s=l**(1/3),m**(1/3),s**(1/3)
    return (0.2104542553*l+0.7936177850*m-0.0040720468*s, 1.9779984951*l-2.4285922050*m+0.4505937099*s, 0.0259040371*l+0.7827717662*m-0.8086757660*s)
def oklab2rgb(L,a,b):
    l=L+0.3963377774*a+0.2158037573*b; m=L-0.1055613458*a-0.0638541728*b; s=L-0.0894841775*a-1.2914855480*b
    l,m,s=l**3,m**3,s**3
    r=4.0767416621*l-3.3077115913*m+0.2309699292*s; g=-1.2684380046*l+2.6097574011*m-0.3413193965*s; bb=-0.0041960863*l-0.7034186147*m+1.7076147010*s
    return tuple(unlin(max(0,min(1,x))) for x in (r,g,bb))
def lch(rgb):
    L,a,b=rgb2oklab(rgb); return L, math.hypot(a,b), math.degrees(math.atan2(b,a))%360
def from_lch(L,C,h):
    return oklab2rgb(L, C*math.cos(math.radians(h)), C*math.sin(math.radians(h)))
def dist(h1,h2):
    a=rgb2oklab(hex2rgb(h1)); b=rgb2oklab(hex2rgb(h2)); return 100*math.sqrt(sum((x-y)**2 for x,y in zip(a,b)))
def luminance(h):
    r,g,b=[lin(x) for x in hex2rgb(h)]; return 0.2126*r+0.7152*g+0.0722*b
def contrast(h1,h2):
    a,b=luminance(h1),luminance(h2); a,b=max(a,b),min(a,b); return (a+0.05)/(b+0.05)
# Machado, Oliveira, Fernandes 2009 severity 1.0 matrices (linear RGB)
CVD={'protan':[[0.152286,1.052583,-0.204868],[0.114503,0.786281,0.099216],[-0.003882,-0.048116,1.051998]],
     'deutan':[[0.367322,0.860646,-0.227968],[0.280085,0.672501,0.047413],[-0.011820,0.042940,0.968881]],
     'tritan':[[1.255528,-0.076749,-0.178779],[-0.078411,0.930809,0.147602],[0.004733,0.691367,0.303900]]}
def simulate(h,kind):
    r,g,b=[lin(x) for x in hex2rgb(h)]; M=CVD[kind]
    out=[M[i][0]*r+M[i][1]*g+M[i][2]*b for i in range(3)]
    return rgb2hex(*[unlin(max(0,min(1,x))) for x in out])
def relight(h, bg, floor=3.0, dark_bg=True):
    """Keep hue and chroma, move lightness until contrast vs bg clears the floor."""
    L,C,H=lch(hex2rgb(h))
    step=0.02 if dark_bg else -0.02
    for _ in range(40):
        cand=rgb2hex(*from_lch(L,C,H))
        if contrast(cand,bg)>=floor: return cand
        L+=step
        if not 0<L<1: break
    return rgb2hex(*from_lch(max(0,min(1,L)),C,H))
