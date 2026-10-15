/* Generated from design-explorer.html by extract_ui.py: layouts, themes, skins, effects. */
const SANS = '"IBM Plex Sans",system-ui,sans-serif', DISP = '"Bricolage Grotesque",system-ui,sans-serif',
  PIXEL = '"Press Start 2P",monospace', VT = '"VT323",monospace', MONO = '"JetBrains Mono",monospace',
  BEVEL = 'inset 2px 2px 0 rgba(255,255,255,.22),inset -2px -3px 0 rgba(0,0,0,.35)',
  SOFT = 'inset 0 1px 0 rgba(255,255,255,.07),0 6px 18px rgba(0,0,0,.22)', DROP = '2px 2px 0 rgba(0,0,0,.5)';
const flat = o => Object.assign({font:SANS, title:DISP, icon:DISP, size:'16px', tsize:'20px', hint:'12.5px', radius:'10px', radius2:'14px', bw:'1px', shadow:'none', bevel:SOFT}, o);
const THEMES = {
  arcade:  flat({label:'Arcade (flat)', bg:'#1b1d22', item:'#6b6b6b', itemHi:'#7f89bd', panel:'rgba(36,37,42,.94)', text:'#f0f0f0', hi:'#ffffa0', accent:'#7dff7d', border:'#000', font:VT, title:PIXEL, icon:PIXEL, size:'25px', tsize:'15px', hint:'18px', radius:'0px', radius2:'2px', bw:'2px', shadow:DROP, bevel:BEVEL, tui:'#7dff7d'}),
  stone:   flat({label:'Slate', bg:'#15171c', item:'#2a2e37', itemHi:'#3b4252', panel:'rgba(28,31,38,.94)', text:'#e8eaf0', hi:'#ffffff', accent:'#8fa3ff', border:'#3a404c', tui:'#8fa3ff'}),
  clay:    flat({label:'Claude Clay', bg:'#1d1714', item:'#3a2c25', itemHi:'#c96442', panel:'rgba(34,26,22,.94)', text:'#f7ece4', hi:'#ffffff', accent:'#e07a52', border:'#4a372d', tui:'#e8916c'}),
  paper:   flat({label:'Paper (light)', bg:'#efe9df', item:'#fffdf9', itemHi:'#f6dccd', panel:'rgba(255,253,249,.96)', text:'#1e1b17', hi:'#a8401f', accent:'#c96442', border:'#d9cfc1', bevel:'0 1px 2px rgba(0,0,0,.08)', tui:'#e8916c'}),
  glass:   flat({label:'Fluent Glass', bg:'#1a2030', item:'rgba(255,255,255,.12)', itemHi:'rgba(255,255,255,.26)', panel:'rgba(30,36,50,.55)', text:'#ffffff', hi:'#ffffff', accent:'#4cc2ff', border:'rgba(255,255,255,.25)', font:'"Segoe UI Variable","Segoe UI","IBM Plex Sans",sans-serif', radius:'8px', tui:'#4cc2ff'}),
  aurora:  flat({label:'Aurora', bg:'#0a1020', item:'rgba(255,255,255,.08)', itemHi:'rgba(124,240,197,.22)', panel:'rgba(14,20,40,.72)', text:'#eaf2ff', hi:'#ffffff', accent:'#7cf0c5', border:'rgba(160,200,255,.22)', radius:'12px', radius2:'18px', tui:'#7cf0c5',
    grad:'radial-gradient(60% 55% at 18% 18%,rgba(124,240,197,.28),transparent),radial-gradient(55% 60% at 85% 75%,rgba(140,110,255,.32),transparent)'}),
  nether:  flat({label:'Ember', bg:'#1a0b08', item:'#3a1712', itemHi:'#c2410c', panel:'rgba(36,14,10,.94)', text:'#fde7dc', hi:'#fff4e6', accent:'#fb923c', border:'#5a2318', tui:'#fb923c'}),
  end:     flat({label:'Void', bg:'#0b0815', item:'#1c1630', itemHi:'#5b3fa8', panel:'rgba(20,15,36,.94)', text:'#ece6ff', hi:'#ffffff', accent:'#a78bfa', border:'#3a2d63', tui:'#c4b5fd'}),
  ocean:   flat({label:'Lagoon', bg:'#04191d', item:'#0f3036', itemHi:'#14857a', panel:'rgba(6,34,40,.92)', text:'#dcfbf6', hi:'#ffffff', accent:'#2dd4bf', border:'#1d4e55', tui:'#5eead4'}),
  neon:    flat({label:'Neon', bg:'#05040c', item:'#12111f', itemHi:'#1e1340', panel:'rgba(10,9,22,.92)', text:'#cfd6ff', hi:'#00f0ff', accent:'#ff2bd6', border:'#ff2bd6', font:MONO, icon:MONO, size:'15px', hint:'12px', radius:'4px', radius2:'8px', bevel:'0 0 14px rgba(255,43,214,.25)', shadow:'0 0 10px rgba(0,240,255,.45)', tui:'#00f0ff'}),
  phosphor:flat({label:'Phosphor', bg:'#000a02', item:'#062a10', itemHi:'#0d4a1d', panel:'rgba(0,14,4,.94)', text:'#39ff6a', hi:'#c6ffd4', accent:'#39ff6a', border:'#0f5a22', font:VT, title:VT, icon:VT, size:'26px', tsize:'28px', hint:'19px', radius:'0px', radius2:'0px', bevel:'none', shadow:'0 0 8px rgba(57,255,106,.5)', tui:'#39ff6a'}),
};

const GRAIN = (f, oct, a) => `url("data:image/svg+xml,${encodeURIComponent(`<svg xmlns='http://www.w3.org/2000/svg' width='180' height='180'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='${f}' numOctaves='${oct}' stitchTiles='stitch'/><feColorMatrix values='.33 .33 .33 0 0 .33 .33 .33 0 0 .33 .33 .33 0 0 0 0 0 0 ${a}'/></filter><rect width='100%' height='100%' filter='url(#n)'/></svg>`)}")`;
const TEX = {
  none:     {label:'Flat', d:'Solid colour, nothing on top.', css:'none'},
  gloss:    {label:'Glossy', d:'Candy shine on the top half, like an app icon.', css:'linear-gradient(180deg,rgba(255,255,255,.36),rgba(255,255,255,.08) 48%,rgba(0,0,0,.05) 52%,rgba(0,0,0,.2))'},
  satin:    {label:'Satin sheen', d:'A soft diagonal highlight; subtle and premium.', css:'linear-gradient(115deg,rgba(255,255,255,0) 18%,rgba(255,255,255,.24) 45%,rgba(255,255,255,0) 72%),linear-gradient(180deg,rgba(255,255,255,.08),rgba(0,0,0,.14))'},
  brushed:  {label:'Brushed metal', d:'Fine vertical hairlines with a steel gradient.', css:'repeating-linear-gradient(90deg,rgba(255,255,255,.08) 0 1px,rgba(0,0,0,.06) 1px 2px,transparent 2px 4px),linear-gradient(180deg,rgba(255,255,255,.24),rgba(0,0,0,.22))'},
  carbon:   {label:'Carbon fibre', d:'Woven diagonal weave; sporty and dark.', css:'repeating-linear-gradient(45deg,rgba(0,0,0,.3) 0 3px,rgba(255,255,255,.05) 3px 6px),repeating-linear-gradient(-45deg,rgba(0,0,0,.22) 0 3px,rgba(255,255,255,.04) 3px 6px)'},
  frost:    {label:'Frosted glass', d:'Milky light falloff with fine frost grain.', css:`linear-gradient(135deg,rgba(255,255,255,.32),rgba(255,255,255,.05) 55%,rgba(255,255,255,.14)),${GRAIN(.9, 2, .22)}`},
  grain:    {label:'Film grain', d:'Smooth photographic grain (not pixels).', css:GRAIN(.8, 3, .7), blend:'overlay'},
  stone:    {label:'Soft stone', d:'Large mottled clouds, like polished stone.', css:GRAIN(.045, 4, .85), blend:'overlay'},
  velvet:   {label:'Velvet', d:'Deep, soft light pooling from one corner.', css:'radial-gradient(120% 90% at 30% 0%,rgba(255,255,255,.24),transparent 60%),radial-gradient(100% 100% at 100% 100%,rgba(0,0,0,.38),transparent 60%)'},
  woven:    {label:'Woven fabric', d:'Fine cross-hatch weave, like canvas.', css:'repeating-linear-gradient(0deg,rgba(255,255,255,.07) 0 2px,transparent 2px 4px),repeating-linear-gradient(90deg,rgba(0,0,0,.14) 0 2px,transparent 2px 4px)'},
  halftone: {label:'Halftone', d:'Comic-print dots.', css:'radial-gradient(circle,rgba(0,0,0,.32) 1.1px,transparent 1.6px) 0 0/7px 7px'},
  blueprint:{label:'Blueprint grid', d:'Engineering grid lines.', css:'linear-gradient(rgba(255,255,255,.14) 1px,transparent 1px) 0 0/10px 10px,linear-gradient(90deg,rgba(255,255,255,.14) 1px,transparent 1px) 0 0/10px 10px'},
  holo:     {label:'Holographic', d:'Animated rainbow foil that drifts slowly.', css:'linear-gradient(125deg,rgba(255,0,170,.5),rgba(0,220,255,.5) 25%,rgba(255,240,0,.45) 50%,rgba(140,80,255,.5) 75%,rgba(255,0,170,.5))', blend:'overlay'},
};
const DEPTHS = {
  flat:   {label:'Flat', d:'Uses the theme’s own edge only.'},
  raised: {label:'Raised', d:'Lifted off the surface with a soft shadow; rises when selected.'},
  slab:   {label:'Slab (pressable)', d:'A solid 3D slab with a visible side; pushes down when chosen.'},
  soft:   {label:'Soft UI', d:'Neumorphic: moulded from the surface; sinks in when chosen.'},
  chisel: {label:'Chiseled', d:'Hard bevelled edges that flip when pressed.'},
  engrave:{label:'Engraved', d:'Carved into the surface; pops out when chosen.'},
  float:  {label:'Floating', d:'Hovers above a long shadow; floats higher when chosen.'},
};
function skinVars(el, tex, depth, extra = '') {
  const T = TEX[tex] || TEX.none;
  el.style.setProperty('--tx', T.css);
  el.style.setProperty('--tx-blend', T.blend || 'normal');
  el.style.setProperty('--tx-op', S.texOp);
  el.classList.remove(...[...el.classList].filter(c => /^(d-|tx-holo$|tilt3d$)/.test(c)));
  el.classList.add('sk', 'd-' + depth, ...(tex === 'holo' ? ['tx-holo'] : []), ...(extra ? [extra] : []));
}

const OPEN_ANIMS = {fade:'Fade', drop:'Drop in', flip:'3D flip', slide:'Slide', rise:'Rise + blur', depth:'Fly from depth', spring:'Spring pop', crt:'CRT on', portal:'Portal', pixel:'Pixel resolve'};
const SEL_ANIMS  = {press:'Press', burst:'Particle burst', ripple:'Ripple rings', break:'Shatter', glow:'Glow', zoom:'Zoom through', fling:'Fling away', shake:'Shake'};
const SEL_DUR = {press:280, burst:520, ripple:520, break:620, glow:480, zoom:440, fling:420, shake:380};

function hexA(hex, a) {
  if (!hex.startsWith('#')) return hex;
  const n = parseInt(hex.slice(1), 16);
  return `rgba(${n >> 16 & 255},${n >> 8 & 255},${n & 255},${a})`;
}
function initials(name) {
  const caps = (name.match(/[A-Z0-9]/g) || []);
  if (caps.length >= 2) return caps.slice(0, 2).join('');
  const w = name.trim().split(/\s+/);
  return (w.length > 1 ? w[0][0] + w[1][0] : name.slice(0, 2)).toUpperCase();
}


function fuzzy(q, s) {
  q = q.toLowerCase(); s = s.toLowerCase();
  let j = 0, score = 0, last = -2;
  for (let i = 0; i < s.length && j < q.length; i++) {
    if (s[i] === q[j]) { score += (i === last + 1) ? 3 : 1; if (i === 0 || ' -_\\'.includes(s[i - 1])) score += 2; last = i; j++; }
  }
  return j === q.length ? score : -1;
}

/* ───────── interfaces ───────── */
const HINT_V = '↑↓ choose · 1–9 jump · ↵ Claude · ⇧↵ continue · Ctrl↵ shell · Esc';
const UIS = {
  blockcraft:{label:'Game Menu', sub:'stacked buttons · your idea', axis:'v', filter:true, best:'Your concept; daily driver', sc:[5,4,3,4,4,5], render(L, sel) {
    return `<div class="ui"><div class="panel">
      <div class="bc-title">${cur.q ? 'Search: ' + esc(cur.q) + '<span class="caret"></span>' : 'Select Project'}</div>
      <div class="bc-list list">${L.map((it, k) => `${it.other && k ? '<div class="bc-sep"></div>' : ''}<button class="mcb tx" data-k="${k}" style="--i:${k};--pc:${it.color}">
        <span class="mcb-key">${it.key || '·'}</span><span class="mcb-name">${esc(it.name)}</span>
        <span class="mcb-meta">${it.other ? (S.other.action === 'pick' ? 'pick folder' : 'shell in ' + esc(it.path)) : esc(it.last)}</span></button>`).join('')}</div>
      <div class="hint">${HINT_V}<br>type to filter</div></div></div>`;
  }},
  hotbar:{label:'Hotbar', sub:'slots 1–9 · scroll wheel', axis:'h', filter:false, best:'Fastest muscle memory, ≤ 9 projects', sc:[5,2,4,4,4,5], render(L) {
    return `<div class="ui ui-hot"><div class="panel" style="display:flex;flex-direction:column;align-items:center;max-width:100%">
      <div class="hot-tip" id="hotTip"></div><div class="hot-path" id="hotPath"></div>
      <div class="hotbar list">${L.map((it, k) => `<div class="slot" data-k="${k}" style="--i:${k};--pc:${it.color}"><span class="slot-ico tx">${it.other ? '…' : esc(initials(it.name))}</span><span class="slot-n">${it.key}</span></div>`).join('')}</div>
      <div class="hint">1–9 · ← → or wheel · ↵ open · ⇧↵ continue</div></div></div>`;
  }, update(L, sel) {
    const it = L[sel]; if (!it) return;
    const t = $('#hotTip'), p = $('#hotPath');
    t.textContent = it.name; p.textContent = it.other ? (S.other.action === 'pick' ? 'pick a folder…' : 'shell in ' + it.path) : it.path;
    [t, p].forEach(e => { e.classList.remove('flash'); void e.offsetWidth; e.classList.add('flash'); });
  }},
  spotlight:{label:'Spotlight', sub:'type to filter · Raycast-like', axis:'v', filter:true, best:'Many projects, keyboard purists', sc:[5,5,3,3,4,1], render(L, sel) {
    return `<div class="ui"><div class="panel spot">
      <div class="spot-in"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg>
        <span class="spot-q">${cur.q ? esc(cur.q) : '<span class="ph">Search projects…</span>'}<span class="caret"></span></span><kbd>Ctrl \\</kbd></div>
      <div class="spot-list list">${L.length ? L.map((it, k) => `<div class="srow" data-k="${k}" style="--i:${k};--pc:${it.color}">
        <span class="sdot tx">${it.other ? '…' : esc(initials(it.name))}</span>
        <span class="stext"><b>${esc(it.name)}</b><small>${esc(it.other ? (S.other.action === 'pick' ? 'folder picker → claude' : it.path + '  · shell only') : it.path)}</small></span>
        <span class="smeta">${esc(it.last)}</span><kbd>${it.key}</kbd></div>`).join('') : '<div class="empty">No match · Esc clears</div>'}</div>
      <div class="spot-foot"><span><kbd>↵</kbd> Claude</span><span><kbd>⇧↵</kbd> Continue</span><span><kbd>Ctrl↵</kbd> Shell</span><span><kbd>Esc</kbd> ${cur.q ? 'clear' : 'close'}</span></div>
    </div></div>`;
  }},
  radial:{label:'Radial', sub:'point in a direction & click', axis:'both', filter:false, best:'Mouse-first, gesture lovers', sc:[4,2,5,4,3,2], render(L) {
    const n = L.length, R = 142;
    return `<div class="ui"><div class="panel radial"><div class="rring"></div><div class="rpointer" id="rPtr"></div>
      ${L.map((it, k) => { const a = -Math.PI / 2 + k * 2 * Math.PI / n;
        return `<div class="rnode tx" data-k="${k}" data-a="${a}" style="--i:${k};--pc:${it.color};left:calc(50% + ${(Math.cos(a) * R).toFixed(1)}px);top:calc(50% + ${(Math.sin(a) * R).toFixed(1)}px)">${it.other ? '…' : esc(initials(it.name))}<em>${it.key}</em></div>`; }).join('')}
      <div class="rcenter"><b id="rName"></b><small id="rPath"></small></div></div>
      <div class="hint">point the mouse · arrows · 1–9 · ↵ / click</div></div>`;
  }, update(L, sel) {
    const it = L[sel]; if (!it) return;
    $('#rName').textContent = it.name; $('#rPath').textContent = it.path;
    const node = $(`#ov [data-k="${sel}"]`);
    if (node) $('#rPtr').style.rotate = (parseFloat(node.dataset.a) * 180 / Math.PI - 90) + 'deg';
  }},
  carousel:{label:'Carousel 3D', sub:'cover-flow cards', axis:'h', filter:false, noHover:true, best:'Show-off mode, big previews', sc:[3,3,4,5,3,2], render(L) {
    return `<div class="ui"><div class="panel car"><div class="car-track">${L.map((it, k) => `<div class="card3" data-k="${k}" style="--i:${k};--pc:${it.color}">
      <span class="card-key">${it.key}</span><div class="card-art tx">${it.other ? '…' : esc(initials(it.name))}</div><b>${esc(it.name)}</b><small>${esc(it.path)}</small></div>`).join('')}</div></div>
      <div class="hint">← → · wheel · 1–9 · ↵ open</div></div>`;
  }, update(L, sel) {
    $$('#ov .card3').forEach((c, k) => {
      const o = k - sel, a = Math.abs(o);
      c.style.transform = `translateX(${o * 150}px) translateZ(${-a * 150}px) rotateY(${o === 0 ? 0 : (o < 0 ? 42 : -42)}deg)`;
      c.style.zIndex = 100 - a; c.style.opacity = a > 3 ? 0 : 1 - a * .14;
    });
  }},
  glass:{label:'Glass Grid', sub:'Windows 11 tiles', axis:'grid', cols:3, filter:false, best:'Feels native to Windows', sc:[4,3,4,3,4,1], render(L) {
    return `<div class="ui"><div class="panel glass">${L.map((it, k) => `<div class="tile tx" data-k="${k}" style="--i:${k};--pc:${it.color}">
      <span class="tk">${it.key}</span><span class="ic">${it.other ? '…' : esc(initials(it.name))}</span>
      <span><b>${esc(it.name)}</b><small>${esc(it.path)}</small></span></div>`).join('')}</div>
      <div class="hint">arrows move in 2D · 1–9 · ↵ open</div></div>`;
  }},
  tui:{label:'Terminal TUI', sub:'a `cc` command inside PowerShell', axis:'v', filter:true, best:'Zero-install fallback', sc:[4,4,1,2,5,2], render(L) {
    const w = Math.max(...items().map(i => i.name.length)) + 2;
    return `<div class="ui"><div class="panel tui"><div class="tui-bar">Windows PowerShell</div><div class="tui-body">
      <div class="tui-head">PS C:\\Users\\you&gt; cc</div>
      <div class="list" style="max-height:290px;overflow:auto">${L.map((it, k) => `<div class="trow" data-k="${k}" style="--i:${k}">[${it.key}] ${esc(it.name.padEnd(w))}<span class="tp">${esc(it.path)}</span></div>`).join('') || '<div class="trow">  no match</div>'}</div>
      <div class="tui-prompt">${L.length}/${items().length} ❯ ${esc(cur.q)}<span class="caret"></span></div></div></div>
      <div class="hint">a PowerShell function + PSReadLine, no app needed</div></div>`;
  }},
  switcher:{label:'Alt-Tab Strip', sub:'big icons, like Alt+Tab', axis:'h', filter:false, best:'Feels familiar on day one', sc:[5,3,4,3,5,2], render(L) {
    return `<div class="ui"><div class="panel as-strip list">${L.map((it, k) => `<div class="asc" data-k="${k}" style="--i:${k};--pc:${it.color}">
      <span class="k">${it.key}</span><span class="big tx">${ic(it)}</span><b>${esc(it.name)}</b></div>`).join('')}</div>
      <div class="as-cap" id="asCap"></div><div class="hint">← → · wheel · 1–9 · ↵ open</div></div>`;
  }, update(L, sel) {
    const it = L[sel]; if (!it) return;
    $('#asCap').innerHTML = `${esc(it.path)} <span style="opacity:.6">· ${esc(it.other ? 'plain shell' : it.last + ' · ⎇ ' + it.git)}</span>`;
  }},
  bento:{label:'Bento', sub:'pinned get bigger tiles', axis:'both', filter:false, best:'At-a-glance status board', sc:[4,3,4,4,3,1], render(L) {
    return `<div class="ui"><div class="panel bento">${L.map((it, k) => `<div class="bt tx ${k === 0 ? 'big' : k === 1 ? 'wide' : ''}" data-k="${k}" style="--i:${k};--pc:${it.color}">
      <span class="k">${it.key}</span><span><b>${esc(it.name)}</b><small>${esc(it.path)}</small></span>
      <span class="meta">${it.other ? '<span>plain shell</span>' : `<span><i class="dot"></i>${esc(it.last)}</span><span>⎇ ${esc(it.git)}</span>`}</span></div>`).join('')}</div>
      <div class="hint">1st = most used, 2nd = runner-up · arrows · 1–9 · ↵</div></div>`;
  }},
  drawer:{label:'Edge Drawer', sub:'slides in from the screen edge', axis:'v', filter:true, best:'Out of the way; hot-edge trigger', sc:[4,5,4,3,4,1], render(L) {
    const rows = L.map((it, k) => `${!cur.q && k === 0 ? '<div class="drw-sec">Pinned</div>' : ''}${!cur.q && k === 3 && !it.other ? '<div class="drw-sec">Projects</div>' : ''}${it.other ? '<div class="drw-sec">More</div>' : ''}<div class="drow" data-k="${k}" style="--i:${k};--pc:${it.color}">
      <span><b>${esc(it.name)}</b><small>${esc(it.other ? it.path + ' · shell' : it.path + ' · ' + it.last)}</small></span><kbd>${it.key}</kbd></div>`).join('');
    return `<div class="ui ui-drawer"><div class="drw-wrap"><div class="panel flat drw"><div class="drw-h">Projects</div>
      <div class="drw-q">${cur.q ? esc(cur.q) : '<span style="opacity:.5">Type to filter…</span>'}<span class="caret"></span></div>
      <div class="drw-list list">${rows || '<div class="empty">No match</div>'}</div>
      <div class="drw-f">↑↓ · 1–9 · ↵ Claude · ⇧↵ continue · Esc</div></div></div></div>`;
  }},
  quake:{label:'Drop-down Bar', sub:'Quake-style, slides from the top', axis:'h', filter:true, best:'Type-first terminal people', sc:[5,4,3,3,4,1], render(L) {
    return `<div class="ui ui-quake"><div class="qk-wrap"><div class="panel flat qk">
      <div class="qk-line"><span class="pr">claude ❯</span> ${esc(cur.q)}<span class="caret"></span> <span style="opacity:.45">${cur.q ? '' : 'type a project name, or press 1–9'}</span></div>
      <div class="qk-chips list">${L.map((it, k) => `<div class="qkc tx" data-k="${k}" style="--i:${k};--pc:${it.color}"><i></i>${esc(it.name)}<kbd>${it.key}</kbd></div>`).join('') || '<span style="opacity:.6">no match</span>'}</div>
      <div class="qk-prev" id="qkPrev"></div></div></div></div>`;
  }, update(L, sel) {
    const it = L[sel], p = $('#qkPrev');
    if (p) p.textContent = it ? `→ ${it.path}${it.other ? '   (plain shell)' : '   ·  last session ' + it.last + '  ·  ⎇ ' + it.git}` : '';
  }},
  keymap:{label:'Keymap', sub:'your number row, labelled', axis:'h', filter:false, best:'Learning the numbers fast', sc:[5,2,3,4,4,3], render(L) {
    const byKey = {}; L.forEach((it, k) => { if (it.key) byKey[it.key] = k; });
    const key = d => { const k = byKey[d]; if (k === undefined) return `<div class="kc ghost"><span class="kn">${d}</span></div>`;
      const it = L[k]; return `<div class="kc tx" data-k="${k}" style="--i:${k};--pc:${it.color}"><span class="kn">${d}</span><span class="kl">${esc(it.name)}</span></div>`; };
    const ghost = s => [...s].map(c => `<div class="kc ghost"><span class="kn">${c}</span></div>`).join('');
    return `<div class="ui"><div class="panel km"><div class="km-row">${[...'1234567890'].map(key).join('')}</div>
      <div class="km-row" style="margin-left:32px">${ghost('QWERTYUIOP')}</div><div class="km-row" style="margin-left:50px">${ghost('ASDFGHJKL')}</div></div>
      <div class="km-cap" id="kmCap"></div><div class="hint">press the key you see · ← → · ↵</div></div>`;
  }, update(L, sel) {
    const it = L[sel]; if (!it) return;
    $('#kmCap').innerHTML = `<b>${esc(it.name)}</b> <span style="opacity:.7;font-family:'JetBrains Mono';font-size:.75em">${esc(it.path)}</span>`;
  }},
  orbit:{label:'Orbit 3D', sub:'projects orbit; the front one is chosen', axis:'h', filter:false, noHover:true, best:'Showpiece with depth', sc:[3,2,4,5,3,2], render(L) {
    return `<div class="ui"><div class="panel orb"><div class="orb-ring">${L.map((it, k) => `<div class="ob tx" data-k="${k}" style="--i:${k};--pc:${it.color}">${ic(it)}</div>`).join('')}</div></div>
      <div class="orb-c" id="orbC"></div><div class="hint">← → spin · wheel · 1–9 · ↵</div></div>`;
  }, update(L, sel) {
    const n = L.length;
    $$('#ov .ob').forEach((el, k) => {
      let o = ((k - sel) % n + n) % n; if (o > n / 2) o -= n;
      const a = o / n * Math.PI * 2, c = Math.cos(a);
      el.style.transform = `translate3d(${(Math.sin(a) * 200).toFixed(1)}px,${((1 - c) * -36).toFixed(1)}px,${((c - 1) * 150).toFixed(1)}px)`;
      el.style.zIndex = Math.round(c * 100) + 100;
      el.style.opacity = (.3 + .7 * (c + 1) / 2).toFixed(2);
      el.style.filter = c < .3 ? `blur(${((1 - c) * 1.3).toFixed(1)}px)` : 'none';
    });
    const it = L[sel]; $('#orbC').innerHTML = it ? `<b>${esc(it.name)}</b><small>${esc(it.path)}</small>` : '';
  }},
  fan:{label:'Card Fan', sub:'a hand of cards; chosen one lifts', axis:'h', filter:false, noHover:true, best:'Playful and tactile', sc:[4,2,4,5,4,3], render(L) {
    return `<div class="ui ui-fan"><div class="fan-cap" id="fanCap"></div><div class="panel fan">${L.map((it, k) => `<div class="fc" data-k="${k}" style="--i:${k};--pc:${it.color}">
      <span class="k">${it.key}</span><div class="art tx">${ic(it)}</div><b>${esc(it.name)}</b></div>`).join('')}</div>
      <div class="hint">← → · wheel · click a card · 1–9 · ↵</div></div>`;
  }, update(L, sel) {
    const n = L.length;
    $$('#ov .fc').forEach((el, k) => { el.style.transform = `rotate(${((k - (n - 1) / 2) * 8).toFixed(1)}deg) translateY(${k === sel ? -48 : 0}px)`; el.style.zIndex = k === sel ? 50 : k; });
    const it = L[sel]; $('#fanCap').innerHTML = it ? `<b>${esc(it.name)}</b><small>${esc(it.path)}</small>` : '';
  }},
  flyout:{label:'Tray Flyout', sub:'Win 11 flyout above the tray', axis:'v', filter:true, best:'Low-key, native, mouse-friendly', sc:[4,4,4,2,5,1], render(L) {
    return `<div class="ui ui-fly"><div class="panel flat fly"><div class="fly-h"><span>Open project</span><span style="opacity:.6;font-size:.82em;font-weight:400">${cur.q ? '“' + esc(cur.q) + '”' : 'type to filter'}</span></div>
      <div class="fly-list list">${L.map((it, k) => `<div class="frow" data-k="${k}" style="--i:${k};--pc:${it.color}"><span class="ic tx">${ic(it)}</span>
        <span><b>${esc(it.name)}</b><small>${esc(it.other ? it.path + ' · shell' : it.last + ' · ⎇ ' + it.git)}</small></span><kbd>${it.key}</kbd></div>`).join('') || '<div class="empty">No match</div>'}</div>
      <div class="fly-f"><span>⚙ Settings</span><span>↵ Claude · ⇧↵ continue</span></div></div>
      <div class="taskbar"><i></i><i></i><i class="me"></i><i></i><i></i><span class="tray">ENG&nbsp;&nbsp;12:04</span></div></div>`;
  }},
};
function ic(it) { return it.other ? '…' : esc(initials(it.name)); }


/* ───────── sound ───────── */
let AC;
function blip(f = 600, d = .04, type = 'square', vol = .035) {
  if (!S.sound) return;
  try {
    AC = AC || new (window.AudioContext || window.webkitAudioContext)();
    const o = AC.createOscillator(), g = AC.createGain();
    o.type = type; o.frequency.value = f; g.gain.setValueAtTime(vol, AC.currentTime);
    g.gain.exponentialRampToValueAtTime(.0001, AC.currentTime + d);
    o.connect(g).connect(AC.destination); o.start(); o.stop(AC.currentTime + d + .02);
  } catch (e) {}
}

/* ───────── select effects ───────── */
function relRect(el) { const a = el.getBoundingClientRect(), b = stage.getBoundingClientRect(); return {x:a.left - b.left, y:a.top - b.top, w:a.width, h:a.height}; }
function particles(el, color) {
  const r = relRect(el), cx = r.x + r.w / 2, cy = r.y + r.h / 2;
  for (let i = 0; i < 26; i++) {
    const p = document.createElement('div'), ang = Math.random() * Math.PI * 2, dist = 60 + Math.random() * 130;
    p.className = 'pt';
    p.style.cssText = `left:${cx}px;top:${cy}px;--c:${i % 3 ? color : '#fff'};--dx:${Math.cos(ang) * dist}px;--dy:${Math.sin(ang) * dist}px;--r:${Math.random() * 360}deg`;
    stage.appendChild(p); setTimeout(() => p.remove(), 700);
  }
}
function shards(el, color) {
  const r = relRect(el), cs = getComputedStyle(el), c = cs.backgroundColor !== 'rgba(0, 0, 0, 0)' ? cs.backgroundColor : color;
  const cols = 4, rows = 2, w = r.w / cols, h = r.h / rows;
  for (let i = 0; i < cols; i++) for (let j = 0; j < rows; j++) {
    const s = document.createElement('div'); s.className = 'shard';
    s.style.cssText = `left:${r.x + i * w}px;top:${r.y + j * h}px;width:${w}px;height:${h}px;--c:${c};--dx:${(i - cols / 2 + .5) * 40 + (Math.random() * 30 - 15)}px;--r:${Math.random() * 120 - 60}deg;animation-delay:${Math.random() * 80}ms`;
    stage.appendChild(s); setTimeout(() => s.remove(), 900);
  }
}
function ripple(el, color) {
  const r = relRect(el);
  [0, 140].forEach(d => { const g = document.createElement('div'); g.className = 'ring';
    g.style.cssText = `left:${r.x + r.w / 2}px;top:${r.y + r.h / 2}px;--c:${color};animation-delay:${d}ms`;
    stage.appendChild(g); setTimeout(() => g.remove(), 800 + d); });
}
function playSel(el, color, done) {
  if (!S.motion || !el) { done && done(); return; }
  const a = S.selAnim;
  el.classList.remove('sa-' + a); void el.offsetWidth; el.classList.add('sa-' + a);
  if (a === 'burst') particles(el, color);
  if (a === 'break') shards(el, color);
  if (a === 'ripple') ripple(el, color);
  if (a === 'zoom') ov.classList.add('zooming');
  blip(a === 'break' ? 180 : 880, .09, a === 'break' ? 'sawtooth' : 'square', .05);
  setTimeout(() => { done ? done() : (el.classList.remove('sa-' + a), ov.classList.remove('zooming')); }, SEL_DUR[a] || 300);
}

