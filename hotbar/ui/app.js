/* Hotbar overlay: glue between the shared layouts (skins.js) and the Rust core.
   skins.js expects these globals: S, cur, items(), stage, ov, $, $$, esc. */
'use strict';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => [...r.querySelectorAll(s)];
const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const reduceMotion = matchMedia('(prefers-reduced-motion: reduce)').matches;

const stage = document.body;
const ov = $('#ov');
const cur = { sel: 0, busy: false, q: '', shown: false, loading: false, pending: null };
const okColor = c => /^#(?:[0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(c || '');
const safeColor = c => okColor(c) ? c : '#8d8d8d';
let SNAP = null;   // { config, status[], configPath, elevated, update }
let S = {};        // flattened ui settings, the shape skins.js reads
let UPD = null;    // update status (update.rs Status), kept live by "update" events

/* ───────── state ───────── */
function mapSettings(cfg) {
  const u = cfg.ui || {};
  S = {
    ui: UIS[u.layout] ? u.layout : 'blockcraft',
    theme: THEMES[u.theme] ? u.theme : 'clay',
    alpha: u.opacity ?? .62,
    tex: TEX[u.texture] ? u.texture : 'holo',
    depth: DEPTHS[u.depth] ? u.depth : 'slab',
    texOp: u.textureStrength ?? .6,
    btnTilt: u.buttonTilt ?? true,
    tilt: u.panelTilt ?? true,
    openAnim: OPEN_ANIMS[u.openAnim] ? u.openAnim : 'drop',
    selAnim: SEL_ANIMS[u.selectAnim] ? u.selectAnim : 'press',
    sound: u.sound ?? true,
    motion: (u.animations ?? true) && !reduceMotion,
    other: cfg.other,
  };
}

async function reload() {
  SNAP = await invoke('snapshot');
  mapSettings(SNAP.config);
  setUpdate(SNAP.update);
  if (SNAP.error) toast(SNAP.error);
}

function ago(sec) {
  if (!sec) return '';
  const d = Date.now() / 1000 - sec;
  if (d < 90) return 'just now';
  if (d < 3600) return Math.round(d / 60) + 'm ago';
  if (d < 86400) return Math.round(d / 3600) + 'h ago';
  if (d < 172800) return 'yesterday';
  if (d < 86400 * 14) return Math.round(d / 86400) + 'd ago';
  return Math.round(d / 604800) + 'w ago';
}

function items() {
  const cfg = SNAP.config;
  return [
    ...cfg.projects.map((p, i) => {
      const st = SNAP.status[i] || {};
      const last = st.exists === false ? '⚠ folder missing'
        : (st.running ? '● running' : '') + (st.running && st.lastSession ? ' · ' : '') + (ago(st.lastSession) || (st.running ? '' : 'no sessions yet'));
      return { ...p, color: safeColor(p.color), idx: i, key: i < 9 ? String(i + 1) : '', last, git: st.branch || '—', missing: st.exists === false };
    }),
    { name: 'Other…', path: cfg.other.path, color: '#8d8d8d', other: true, idx: cfg.projects.length, key: '0', last: '', git: '' },
  ];
}

function visible() {
  const all = items();
  if (!cur.q || !UIS[S.ui].filter) return all;
  return all.map(it => ({ it, s: Math.max(fuzzy(cur.q, it.name), fuzzy(cur.q, it.path) - 1) }))
    .filter(x => x.s >= 0).sort((a, b) => b.s - a.s).map(x => x.it);
}

/* ───────── rendering ───────── */
function applyTheme() {
  const t = THEMES[S.theme], r = document.documentElement.style;
  const vars = {
    '--s-item': t.item, '--s-item-hi': t.itemHi, '--s-panel': t.panel, '--s-text': t.text, '--s-hi': t.hi, '--s-accent': t.accent,
    '--s-border': t.border, '--s-font': t.font, '--s-title': t.title, '--s-size': t.size, '--s-hint': t.hint, '--s-radius': t.radius,
    '--s-radius2': t.radius2, '--s-blur': '0px', '--s-tui': t.tui, '--s-icon': t.icon, '--s-tsize': t.tsize,
    '--s-shadow': t.shadow, '--s-bevel': t.bevel, '--s-bw': t.bw,
  };
  for (const k in vars) r.setProperty(k, vars[k]);
  ov.style.backgroundColor = hexA(t.bg, S.alpha);
  ov.style.backgroundImage = t.grad || 'none';
  skinVars(ov, S.tex, S.depth, S.btnTilt && S.motion ? 'tilt3d' : '');
}

function renderStage(anim = true) {
  applyTheme();
  const L = visible();
  if (cur.sel >= L.length) cur.sel = Math.max(0, L.length - 1);
  ov.innerHTML = UIS[S.ui].render(L, cur.sel);
  ov.classList.remove('zooming');
  $$('#ov [data-k]').forEach(el => { if (L[+el.dataset.k]?.missing) el.classList.add('missing'); });
  if (anim && S.motion) ov.firstElementChild.classList.add('oa-' + S.openAnim);
  updSel(false);
}

function updSel(sound = true) {
  const L = visible(), ui = UIS[S.ui];
  $$('#ov [data-k]').forEach(el => el.classList.toggle('sel', +el.dataset.k === cur.sel));
  if (ui.update) ui.update(L, cur.sel);
  const el = $(`#ov [data-k="${cur.sel}"]`), list = el && el.closest('.list');
  if (el && list) {
    if (list.scrollHeight > list.clientHeight) {
      if (el.offsetTop < list.scrollTop) list.scrollTop = el.offsetTop - 6;
      else if (el.offsetTop + el.offsetHeight > list.scrollTop + list.clientHeight) list.scrollTop = el.offsetTop + el.offsetHeight - list.clientHeight + 6;
    }
    if (list.scrollWidth > list.clientWidth) list.scrollLeft = el.offsetLeft - list.clientWidth / 2 + el.offsetWidth / 2;
  }
  if (sound) blip(520 + cur.sel * 40, .035);
}
function setSel(k) { const n = visible().length; if (!n) return; k = (k + n) % n; if (k !== cur.sel) { cur.sel = k; updSel(); } }
function move(d) { setSel(cur.sel + d); }

/* ───────── launching ───────── */
function modeFor(e, it) {
  if (e.shiftKey && e.ctrlKey) return { mode: it.enter || 'claude', forceNew: true };
  if (e.shiftKey) return { mode: 'continue', forceNew: false };
  if (e.ctrlKey) return { mode: 'shell', forceNew: false };
  if (e.altKey) return { mode: 'resume', forceNew: false };
  return { mode: it.enter || 'claude', forceNew: false };
}

async function launch(k, e = {}) {
  const L = visible(), it = L[k];
  if (!it || cur.busy) return;
  if (it.missing) {
    toast(`Folder not found: ${it.path}`);
    const el = $(`#ov [data-k="${k}"]`);
    if (el) { el.classList.remove('sa-shake'); void el.offsetWidth; el.classList.add('sa-shake'); }
    return;
  }
  const { mode, forceNew } = modeFor(e, it);
  cur.sel = k; updSel(false); cur.busy = true;
  await new Promise(done => playSel($(`#ov [data-k="${k}"]`), it.color, done));
  if (!cur.shown) { cur.busy = false; return; } // closed during the animation
  try {
    await invoke('open_project', { index: it.idx, mode, forceNew });
  } catch (err) {
    toast(String(err));
  } finally {
    cur.busy = false;
  }
}

function launchDigit(d, e = {}) {
  const all = items(), target = d === 0 ? all[all.length - 1] : all[d - 1];
  if (!target || (target.other && d !== 0)) return;
  const k = visible().findIndex(x => x.idx === target.idx);
  if (k >= 0) launch(k, e);
}

/* ───────── input ───────── */
// Ctrl ×2 while the menu is open closes it. Windows doesn't run the low-level hook for key-downs
// aimed at Hotbar's own window, so the page detects this double-tap itself (same rules as hook.rs).
const ctrlTap = { down: false, dirty: false, at: 0, last: 0 };
function trigCode() { return SNAP?.config.trigger.key === 'rightCtrl' ? 'ControlRight' : 'ControlLeft'; }
document.addEventListener('keydown', e => {
  if (e.code !== trigCode()) { ctrlTap.dirty = true; ctrlTap.last = 0; }
  else if (!e.repeat && !ctrlTap.down) Object.assign(ctrlTap, { down: true, dirty: false, at: e.timeStamp });
}, true);
document.addEventListener('keyup', e => {
  if (!SNAP || e.code !== trigCode() || !ctrlTap.down) return;
  ctrlTap.down = false;
  const t = SNAP.config.trigger, tap = !ctrlTap.dirty && e.timeStamp - ctrlTap.at <= 250;
  if (tap && ctrlTap.last && e.timeStamp - ctrlTap.last <= (t.tapWindowMs || 350)) {
    ctrlTap.last = 0;
    if (t.doubleTap !== false && $('#settings').hidden) invoke('hide');
  } else ctrlTap.last = tap ? e.timeStamp : 0;
}, true);

document.addEventListener('keydown', e => {
  if (!$('#settings').hidden) {
    if (e.key === 'Escape') { e.preventDefault(); requestCloseSettings(); }
    return;
  }
  // with NumLock off the numpad sends PageUp/Insert/…, which must not launch anything
  const dm = /^\d$/.test(e.key) && /^(?:Digit|Numpad)(\d)$/.exec(e.code);
  if (cur.loading) {
    // the menu is still loading its data: remember a digit for when it's ready
    if (dm && !e.ctrlKey) cur.pending = { d: +dm[1], shiftKey: e.shiftKey, altKey: e.altKey };
    e.preventDefault();
    return;
  }
  if (e.ctrlKey && e.key === ',') { e.preventDefault(); if (SNAP) openSettings(); return; }
  if (!SNAP || cur.busy) { e.preventDefault(); return; }
  const ui = UIS[S.ui], L = visible(), cols = ui.cols || 1;
  let handled = true;
  if (e.key === 'ArrowDown') move(ui.axis === 'grid' ? cols : 1);
  else if (e.key === 'ArrowUp') move(ui.axis === 'grid' ? -cols : -1);
  else if (e.key === 'ArrowRight') move(1);
  else if (e.key === 'ArrowLeft') move(-1);
  else if (e.key === 'Home') setSel(0);
  else if (e.key === 'End') setSel(L.length - 1);
  else if (e.key === 'Enter') launch(cur.sel, e);
  else if (e.key === 'Escape') { if (cur.q) { cur.q = ''; cur.sel = 0; renderStage(false); } else invoke('hide'); }
  else if (e.key === 'Backspace' && ui.filter && cur.q) { cur.q = cur.q.slice(0, -1); cur.sel = 0; renderStage(false); }
  else if (dm && !(ui.filter && cur.q) && !e.ctrlKey) launchDigit(+dm[1], e);
  else if (e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey && e.key !== ' ') {
    if (ui.filter) { cur.q += e.key; cur.sel = 0; renderStage(false); blip(700, .02); }
    else {
      const n = L.length, ch = e.key.toLowerCase();
      for (let s = 1; s <= n; s++) { const k = (cur.sel + s) % n; if (L[k].name.toLowerCase().startsWith(ch)) { setSel(k); break; } }
    }
  }
  else handled = false;
  if (handled) e.preventDefault();
});

ov.addEventListener('click', e => {
  const it = e.target.closest('[data-k]');
  if (it) {
    const k = +it.dataset.k;
    if (UIS[S.ui].noHover && k !== cur.sel) { setSel(k); return; }
    launch(k, e);
  } else if (e.target === ov || e.target.classList.contains('ui')) {
    invoke('hide');
  }
});
ov.addEventListener('mouseover', e => {
  if (S.ui === 'radial' || UIS[S.ui].noHover || cur.busy) return;
  const it = e.target.closest('[data-k]');
  if (it && +it.dataset.k !== cur.sel) { cur.sel = +it.dataset.k; updSel(); }
});
let wheelAt = 0;
ov.addEventListener('wheel', e => {
  e.preventDefault();
  const now = performance.now();
  if (now - wheelAt < 70 || cur.busy) return;
  wheelAt = now;
  move((e.deltaY || e.deltaX) > 0 ? 1 : -1);
}, { passive: false });

let tiltEl = null;
function untilt() { if (tiltEl) { tiltEl.style.rotate = ''; tiltEl.style.setProperty('--glare', '0'); tiltEl = null; } }
document.addEventListener('mousemove', e => {
  const nx = e.clientX / innerWidth - .5, ny = e.clientY / innerHeight - .5;
  const r = document.documentElement.style;
  r.setProperty('--rx', S.tilt && S.motion ? (-ny * 7).toFixed(2) + 'deg' : '0deg');
  r.setProperty('--ry', S.tilt && S.motion ? (nx * 9).toFixed(2) + 'deg' : '0deg');
  if (S.ui === 'radial' && !cur.busy) {
    const p = $('#ov .radial');
    if (p) {
      const pr = p.getBoundingClientRect(), dx = e.clientX - (pr.left + pr.width / 2), dy = e.clientY - (pr.top + pr.height / 2);
      if (Math.hypot(dx, dy) >= 40) {
        const n = visible().length, a = Math.atan2(dy, dx) + Math.PI / 2;
        setSel(Math.round(((a + Math.PI * 2) % (Math.PI * 2)) / (Math.PI * 2 / n)) % n);
      }
    }
  }
  if (!S.btnTilt || !S.motion || cur.busy) return untilt();
  const el = e.target.closest && e.target.closest('#ov [data-k]');
  if (el !== tiltEl) untilt();
  if (!el) return;
  const b = el.getBoundingClientRect(), bx = (e.clientX - b.left) / b.width - .5, by = (e.clientY - b.top) / b.height - .5;
  const mag = Math.min(18, 1800 / Math.max(b.width, b.height)) * Math.min(1, Math.hypot(bx, by) * 1.6);
  el.style.rotate = `${(-by).toFixed(3)} ${bx.toFixed(3)} 0 ${mag.toFixed(1)}deg`;
  el.style.setProperty('--mx', ((bx + .5) * 100).toFixed(0) + '%');
  el.style.setProperty('--my', ((by + .5) * 100).toFixed(0) + '%');
  el.style.setProperty('--glare', '.3');
  tiltEl = el;
});
$('#gear').addEventListener('click', () => openSettings());
$('#vtag').addEventListener('click', () => { if (SNAP) openSettings('updates'); });

/* ───────── events from Rust ───────── */
// Blind-chord digits are handled entirely in Rust (lib.rs chord_digit), without the page.
let summonSeq = 0;
listen('summon', async e => {
  const { view, error } = e.payload || {};
  const seq = ++summonSeq;
  Object.assign(cur, { q: '', sel: 0, busy: false, shown: true, loading: true, pending: null });
  Object.assign(ctrlTap, { down: false, last: 0 });
  try {
    await reload();
  } catch (err) {
    toast(`Couldn't load projects: ${err}`);
  }
  if (seq !== summonSeq) return; // a newer summon took over
  cur.loading = false;
  if (!SNAP) return;
  if (error) toast(error);
  const editing = !$('#settings').hidden;
  if (editing) invoke('set_pinned', { pinned: true }).catch(() => {}); // keep an open editor and its unsaved changes
  if (view === 'settings' || view === 'updates') {
    renderStage(false);
    if (!editing) openSettings(view);
    if (view === 'updates' && UPD?.updatedFrom) toast(`Hotbar updated to v${UPD.current}`);
    return;
  }
  renderStage(!editing);
  if (!editing) blip(440, .06, 'triangle', .05);
  if (cur.pending && !editing) { const p = cur.pending; cur.pending = null; launchDigit(p.d, p); }
});
listen('hidden', () => { cur.shown = false; cur.pending = null; untilt(); ov.innerHTML = ''; });
listen('toast', e => toast(String(e.payload)));
listen('update', e => setUpdate(e.payload));

/* ───────── updates: the version tag and Settings → Updates ───────── */
function setUpdate(u) {
  if (!u) return;
  UPD = u;
  renderTag();
  const box = $('#upd');
  if (box) box.innerHTML = updateHtml();
}

const updPct = () => UPD.total ? Math.min(100, Math.floor(UPD.downloaded * 100 / UPD.total)) : null;
const mb = n => (n / 1048576).toFixed(1) + ' MB';
const busyUpd = () => UPD.phase !== 'idle';

function renderTag() {
  const t = $('#vtag'), a = UPD.available, pct = updPct();
  t.hidden = false;
  t.classList.toggle('new', !!a && !busyUpd());
  t.textContent = UPD.phase === 'downloading' ? `Updating ${pct ?? '…'}${pct == null ? '' : '%'}`
    : UPD.phase === 'installing' ? 'Restarting…'
    : a ? `v${a.version} available` : `v${UPD.current}`;
  t.title = a ? `Hotbar v${UPD.current}: v${a.version} is available. Click for Settings → Updates.` : `Hotbar v${UPD.current}. Click for Settings → Updates.`;
}

function updLine() {
  const u = UPD, a = u.available;
  if (u.phase === 'checking') return 'Checking for updates…';
  if (u.phase === 'downloading') return `Downloading v${a?.version ?? ''}…`;
  if (u.phase === 'installing') return 'Installing: Hotbar closes and comes back in a few seconds.';
  if (a) return `v${a.version} is available` + (a.date ? ` · released ${new Date(a.date).toLocaleDateString()}` : '');
  if (u.updatedFrom) return `Updated from v${u.updatedFrom} to v${u.current}`;
  if (u.checkedAt) return `Up to date · checked ${ago(u.checkedAt)}`;
  return 'Not checked yet';
}

function updateHtml() {
  const u = UPD, a = u.available, busy = busyUpd(), pct = updPct();
  const moving = u.phase === 'downloading' || u.phase === 'installing';
  return `<div class="upd-head">
      <div class="upd-ver"><span>Installed</span><b>v${esc(u.current)}</b></div>
      <div class="upd-state ${a && !busy ? 'new' : ''}" role="status">${esc(updLine())}${u.error && !busy ? `<span class="warn">${esc(u.error)}</span>` : ''}</div>
      <button data-act="upd-check" ${busy ? 'disabled' : ''}>Check for updates</button>
      <button class="pri" data-act="upd-install" ${a && !busy ? '' : 'disabled'}>${a ? `Install v${esc(a.version)}` : 'Install update'}</button>
    </div>
    ${moving ? `<div class="upd-bar ${pct == null ? 'indet' : ''}" role="progressbar" aria-valuemin="0" aria-valuemax="100" ${pct == null ? '' : `aria-valuenow="${pct}"`}><i style="width:${pct ?? 0}%"></i></div>
      <div class="upd-sub">${u.phase === 'installing' ? 'Verified · starting the installer' : u.total ? `${mb(u.downloaded)} of ${mb(u.total)} · ${pct}%` : mb(u.downloaded)}</div>` : ''}
    ${a?.notes ? `<div class="upd-notes"><span>What's new in v${esc(a.version)}</span>${esc(a.notes)}</div>` : ''}`;
}

function toast(msg) {
  const t = $('#toast');
  t.textContent = msg;
  t.classList.add('show');
  clearTimeout(t._h);
  t._h = setTimeout(() => t.classList.remove('show'), 3200);
}

/* ───────── settings ───────── */
let draft = null, draftBase = '', escArmedAt = 0;
// <input type=color> only takes #rrggbb: expand #rgb, drop alpha, fall back to grey
function colorInput(c) {
  if (!okColor(c)) return '#888888';
  const h = c.slice(1);
  return '#' + (h.length <= 4 ? [...h.slice(0, 3)].map(x => x + x).join('') : h.slice(0, 6));
}
const opts = (obj, val) => Object.entries(obj).map(([k, v]) => `<option value="${k}" ${k === val ? 'selected' : ''}>${esc(typeof v === 'string' ? v : v.label)}</option>`).join('');

function openSettings(section) {
  draft = JSON.parse(JSON.stringify(SNAP.config));
  // Fill look settings missing from a hand-edited config with what the menu actually uses,
  // so the form shows (and a save keeps) the current look instead of each list's first option.
  // The list-backed keys always take the validated values from mapSettings (S).
  draft.ui = {
    backdrop: 'blur', textureStrength: S.texOp, opacity: S.alpha, buttonTilt: S.btnTilt, panelTilt: S.tilt,
    sound: S.sound, animations: true,
    ...(draft.ui || {}),
    layout: S.ui, theme: S.theme, texture: S.tex, depth: S.depth, openAnim: S.openAnim, selectAnim: S.selAnim,
  };
  for (const p of draft.projects) if (!['claude', 'continue', 'shell'].includes(p.enter)) p.enter = 'claude';
  invoke('set_pinned', { pinned: true }).catch(() => {});
  $('#settings').hidden = false;
  renderSettings();
  readSettings(); // normalise the draft to what the form shows, so "unchanged" compares equal
  draftBase = JSON.stringify(draft);
  if (section === 'updates') $('#updSec').scrollIntoView({ block: 'start' });
}
function closeSettings(back = true) {
  $('#settings').hidden = true;
  draft = null;
  invoke('set_pinned', { pinned: false }).catch(() => {});
  if (back) renderStage(false);
}
function settingsDirty() {
  if (!draft) return false;
  readSettings();
  return JSON.stringify(draft) !== draftBase;
}
// Esc closes the editor; with unsaved changes it takes a second Esc, so nothing is lost by accident.
function requestCloseSettings() {
  if (!settingsDirty() || performance.now() - escArmedAt < 2500) { escArmedAt = 0; return closeSettings(); }
  escArmedAt = performance.now();
  toast('Unsaved changes: press Esc again to discard them, or Save.');
}

function renderSettings() {
  const scrolled = $('#settings .body')?.scrollTop ?? 0; // keep the place when a row moves
  renderSettingsHtml();
  const body = $('#settings .body');
  if (body) body.scrollTop = scrolled;
}

function renderSettingsHtml() {
  const c = draft, u = c.ui, t = c.trigger, l = c.launch;
  const status = SNAP.status;
  $('#settings').innerHTML = `<div class="sp" role="dialog" aria-label="Hotbar settings">
    <header><img src="hotbar-icon.png" alt=""><h1>Hotbar settings</h1><span class="hintline" style="margin:0">${SNAP.elevated ? 'Running as admin: Claude sessions still open unelevated.' : 'Not elevated: the hotkey won’t fire while an admin window is focused.'}</span></header>
    <div class="body">
      <h2>Projects</h2>
      <p class="hintline">Order sets the number keys (1–9). Drag the number to reorder.</p>
      <div class="projects" id="pList">${c.projects.map((p, i) => `<div class="prow" data-i="${i}">
        <span class="num" draggable="true" title="Drag to reorder">${i < 9 ? i + 1 : '·'}</span>
        <input type="color" data-f="color" value="${esc(colorInput(p.color))}" title="Colour">
        <input type="text" data-f="name" value="${esc(p.name)}" placeholder="Shown name">
        <input type="text" class="mono" data-f="path" value="${esc(p.path)}" placeholder="D:\\Folder">
        <button data-act="browse">Browse…</button>
        <select data-f="enter" title="What Enter does"><option value="claude">↵ claude</option><option value="continue">↵ claude --continue</option><option value="shell">↵ shell only</option></select>
        <span class="acts"><button data-act="up" title="Move up">↑</button><button data-act="down" title="Move down">↓</button><button data-act="del" title="Remove">✕</button></span>
        <span class="more"><input type="text" class="mono" data-f="pre" value="${esc(p.pre)}" placeholder="Run first (optional), e.g. .venv\\Scripts\\Activate.ps1"><input type="text" class="mono" data-f="flags" value="${esc(p.flags)}" placeholder="Extra claude flags (optional), e.g. --model opus"></span>
        ${status[i] && status[i].exists === false && p.path === SNAP.config.projects[i]?.path ? `<span class="warn">⚠ This folder doesn't exist right now.</span>` : ''}
      </div>`).join('')}</div>
      <div style="margin-top:8px"><button data-act="add">+ Add project</button></div>

      <h2>Other… (key 0)</h2>
      <div class="other">
        <label>Folder<input type="text" class="mono" id="oPath" value="${esc(c.other.path)}"></label>
        <button data-act="browse-other">Browse…</button>
        <label>Action<select id="oAction"><option value="shell" ${c.other.action === 'shell' ? 'selected' : ''}>Plain PowerShell there</option><option value="pick" ${c.other.action === 'pick' ? 'selected' : ''}>Folder picker, then Claude</option></select></label>
      </div>

      <h2>Look</h2>
      <div class="grid">
        <label>Layout<select data-ui="layout">${opts(Object.fromEntries(Object.entries(UIS).map(([k, v]) => [k, v.label])), u.layout)}</select></label>
        <label>Theme<select data-ui="theme">${opts(THEMES, u.theme)}</select></label>
        <label>Backdrop<select data-ui="backdrop">${opts({ blur: 'Blur', acrylic: 'Acrylic', none: 'None' }, u.backdrop || 'blur')}</select></label>
        <label>Button texture<select data-ui="texture">${opts(TEX, u.texture)}</select></label>
        <label>3D depth<select data-ui="depth">${opts(DEPTHS, u.depth)}</select></label>
        <label>Texture strength <span class="val">${Math.round((u.textureStrength ?? .6) * 100)}%</span><input type="range" min="0.2" max="1" step="0.05" data-ui="textureStrength" value="${esc(u.textureStrength ?? .6)}"></label>
        <label>Overlay opacity <span class="val">${Math.round((u.opacity ?? .62) * 100)}%</span><input type="range" min="0.15" max="0.95" step="0.01" data-ui="opacity" value="${esc(u.opacity ?? .62)}"></label>
        <label class="check"><input type="checkbox" data-ui="buttonTilt" ${u.buttonTilt !== false ? 'checked' : ''}> Per-button 3D tilt</label>
        <label class="check"><input type="checkbox" data-ui="panelTilt" ${u.panelTilt !== false ? 'checked' : ''}> Panel follows the mouse</label>
      </div>

      <h2>Motion &amp; sound</h2>
      <div class="grid">
        <label>Open gesture<select data-ui="openAnim">${opts(OPEN_ANIMS, u.openAnim)}</select></label>
        <label>Select gesture<select data-ui="selectAnim">${opts(SEL_ANIMS, u.selectAnim)}</select></label>
        <span></span>
        <label class="check"><input type="checkbox" data-ui="animations" ${u.animations !== false ? 'checked' : ''}> Animations</label>
        <label class="check"><input type="checkbox" data-ui="sound" ${u.sound !== false ? 'checked' : ''}> Sound effects</label>
      </div>

      <h2>Trigger</h2>
      <div class="grid">
        <label class="check"><input type="checkbox" id="tDouble" ${(t.doubleTap ?? t.doubleTapRightCtrl) !== false ? 'checked' : ''}> Double-tap Ctrl opens the menu</label>
        <label>Which Ctrl<select id="tKey">${opts({ leftCtrl: 'Left Ctrl', rightCtrl: 'Right Ctrl' }, t.key || 'leftCtrl')}</select></label>
        <label>Max gap between taps (ms)<input type="number" id="tTap" min="150" max="1000" step="10" value="${esc(t.tapWindowMs)}"></label>
        <label class="check"><input type="checkbox" id="tChord" ${t.blindChord ? 'checked' : ''}> Blind chord (Ctrl ×2, then a digit)</label>
        <label>Chord window (ms): the menu waits this long<input type="number" id="tChordMs" min="80" max="1000" step="10" value="${esc(t.chordMs)}"></label>
      </div>

      <h2>Launch</h2>
      <div class="grid">
        <label>Terminal window<select id="lWin">${opts({ maximized: 'Maximized', fullscreen: 'Full screen (F11)', normal: 'Normal' }, l.window)}</select></label>
        <label>Shell<select id="lShell">${opts({ powershell: 'Windows PowerShell 5.1', pwsh: 'PowerShell 7 (pwsh)' }, l.shell)}</select></label>
        <label>Sessions open in<select id="lTabs">${opts({ shared: 'One window: all projects as tabs', project: 'A window per project', new: 'A new window every time' }, l.oneWindow ? 'shared' : l.sameWindow !== false ? 'project' : 'new')}</select></label>
        <label class="check"><input type="checkbox" id="lFocus" ${l.focusExisting ? 'checked' : ''}> Focus an open session instead of opening a duplicate</label>
      </div>
      <p class="hintline" style="margin-top:12px">In the menu: <kbd>↵</kbd> claude · <kbd>⇧↵</kbd> continue · <kbd>Alt↵</kbd> resume picker · <kbd>Ctrl↵</kbd> shell · <kbd>Ctrl⇧↵</kbd> force a new session · <kbd>Ctrl ,</kbd> settings</p>

      <h2 id="updSec">Updates</h2>
      <div class="upd" id="upd">${updateHtml()}</div>
      <div class="grid" style="margin-top:12px">
        <label class="check" style="grid-column: 1 / -1"><input type="checkbox" id="uAuto" ${c.updates?.autoCheck !== false ? 'checked' : ''}> Check automatically (shortly after start, then daily)</label>
        <label class="check" style="grid-column: 1 / -1"><input type="checkbox" id="uReopen" ${c.updates?.openAfterUpdate !== false ? 'checked' : ''}> Open Hotbar after an update (here, showing the result) instead of restarting silently in the tray</label>
      </div>
      <p class="hintline" style="margin-top:8px">Updates come from Hotbar's GitHub releases and are verified against its signing key before they run. They install silently in the background: Hotbar closes for a few seconds and comes back on its own.</p>
    </div>
    <footer><span class="path" id="cfgPath" title="Open in Notepad">${esc(SNAP.configPath)}</span><button data-act="cancel">Cancel</button><button class="pri" data-act="save">Save</button></footer>
  </div>`;
  $$('#pList select[data-f="enter"]').forEach((s, i) => s.value = draft.projects[i].enter || 'claude');
}

function readSettings() {
  const c = draft;
  $$('#pList .prow').forEach(row => {
    const p = c.projects[+row.dataset.i];
    $$('[data-f]', row).forEach(inp => p[inp.dataset.f] = inp.value);
  });
  c.other.path = $('#oPath').value;
  c.other.action = $('#oAction').value;
  $$('#settings [data-ui]').forEach(inp => {
    c.ui[inp.dataset.ui] = inp.type === 'checkbox' ? inp.checked : inp.type === 'range' ? +inp.value : inp.value;
  });
  c.trigger.doubleTap = $('#tDouble').checked;
  c.trigger.key = $('#tKey').value;
  delete c.trigger.doubleTapRightCtrl;
  c.trigger.tapWindowMs = +$('#tTap').value || 350;
  c.trigger.blindChord = $('#tChord').checked;
  c.trigger.chordMs = +$('#tChordMs').value || 200;
  c.launch.window = $('#lWin').value;
  c.launch.shell = $('#lShell').value;
  c.launch.focusExisting = $('#lFocus').checked;
  c.launch.oneWindow = $('#lTabs').value === 'shared';
  c.launch.sameWindow = $('#lTabs').value !== 'new';
  c.updates = { ...c.updates, autoCheck: $('#uAuto').checked, openAfterUpdate: $('#uReopen').checked };
}

const settingsEl = $('#settings');
settingsEl.addEventListener('input', e => {
  if (e.target.type === 'range') {
    const v = e.target.closest('label').querySelector('.val');
    if (v) v.textContent = Math.round(+e.target.value * 100) + '%';
  }
});
settingsEl.addEventListener('click', async e => {
  if (e.target === settingsEl) return; // a stray click on the backdrop must not throw away edits
  if (e.target.id === 'cfgPath') return invoke('open_config_file').catch(() => {});
  const b = e.target.closest('[data-act]');
  if (!b) return;
  const act = b.dataset.act, row = b.closest('.prow'), i = row ? +row.dataset.i : -1, P = draft.projects;
  // update buttons act right away and leave the form (and its unsaved edits) alone
  if (act === 'upd-check') return invoke('check_update').catch(err => toast(String(err)));
  if (act === 'upd-install') {
    if (!UPD?.available) return;
    if (settingsDirty()) return toast('Save or cancel your changes first: installing restarts Hotbar.');
    return invoke('install_update', { version: UPD.available.version }).catch(err => toast(String(err)));
  }
  if (act === 'save') {
    readSettings();
    try {
      await invoke('save_config', { config: draft });
      await reload();
      cur.q = ''; cur.sel = 0; // the layout may have changed under a typed filter
      closeSettings();
      toast('Saved');
    } catch (err) { toast(String(err)); }
    return;
  }
  if (act === 'cancel') return closeSettings();
  readSettings();
  if (act === 'browse' || act === 'browse-other') {
    const start = act === 'browse' ? P[i].path : draft.other.path;
    const picked = await invoke('pick_folder', { start }).catch(() => null);
    if (!draft) return; // the editor was closed while the picker was open
    if (picked) {
      if (act === 'browse') {
        P[i].path = picked;
        if (!P[i].name || P[i].name === 'New project') P[i].name = picked.split(/[\\/]/).filter(Boolean).pop() || picked;
      } else draft.other.path = picked;
    }
  }
  if (act === 'add') P.push({ name: 'New project', path: '', color: ['#f2a541', '#2ec4b6', '#ff6b9a', '#4d96ff'][P.length % 4], enter: 'claude', pre: '', flags: '' });
  if (act === 'del') P.splice(i, 1);
  if (act === 'up' && i > 0) [P[i - 1], P[i]] = [P[i], P[i - 1]];
  if (act === 'down' && i < P.length - 1) [P[i + 1], P[i]] = [P[i], P[i + 1]];
  renderSettings();
});
let dragI = null;
settingsEl.addEventListener('dragend', () => { dragI = null; $$('.prow.drag-over').forEach(x => x.classList.remove('drag-over')); });
settingsEl.addEventListener('dragstart', e => { const r = e.target.closest('.prow'); if (r) { readSettings(); dragI = +r.dataset.i; } });
settingsEl.addEventListener('dragover', e => { const r = e.target.closest('.prow'); if (!r) return; e.preventDefault(); $$('.prow').forEach(x => x.classList.toggle('drag-over', x === r)); });
settingsEl.addEventListener('drop', e => {
  const r = e.target.closest('.prow'); if (!r || dragI === null) return;
  e.preventDefault();
  const [m] = draft.projects.splice(dragI, 1);
  draft.projects.splice(+r.dataset.i, 0, m);
  dragI = null;
  renderSettings();
});

/* first paint happens on the first "summon" event; preload so it's instant */
reload().catch(err => toast(String(err)));
