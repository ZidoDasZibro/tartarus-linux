//! Config UI — 5×4 Tartarus grid, chords, thumb + wheel (no layers)

use crate::config::{self, UiPayload};
use crate::state;
use tiny_http::{Header, Method, Response, Server, StatusCode};

const PORT: u16 = 8787;

const HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>Tartarus Linux</title>
<style>
:root{--bg:#0d0d0f;--card:#16161a;--line:#2a2a32;--txt:#e8e8ec;--mut:#8b8b98;--acc:#3dd68c;--warn:#f5a524;--blue:#5b9cff}
*{box-sizing:border-box}
body{font-family:ui-sans-serif,system-ui,sans-serif;margin:0;background:var(--bg);color:var(--txt);line-height:1.35}
header{padding:1rem 1.25rem;border-bottom:1px solid var(--line);display:flex;align-items:center;gap:1rem}
header h1{margin:0;font-size:1.15rem}
main{max-width:1000px;margin:0 auto;padding:1rem 1.25rem 3rem;display:grid;gap:1rem}
.card{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:1rem}
.card h2{margin:0 0 .75rem;font-size:.8rem;font-weight:600;color:var(--mut);text-transform:uppercase;letter-spacing:.04em}
.row{display:flex;flex-wrap:wrap;gap:.6rem 1rem;align-items:center}
label{font-size:.75rem;color:var(--mut);display:flex;flex-direction:column;gap:.2rem}
label.inline{flex-direction:row;align-items:center;gap:.35rem}
input,select{background:#0f0f12;color:var(--txt);border:1px solid var(--line);border-radius:6px;padding:.3rem .45rem;font-size:.8rem}
input[type=number]{width:4rem}
input[type=checkbox]{width:auto}
select[multiple]{min-height:4.5rem;font-size:.7rem}
button{background:var(--acc);color:#062;border:0;padding:.5rem 1rem;border-radius:8px;font-weight:600;cursor:pointer}
button.ghost{background:transparent;color:var(--mut);border:1px solid var(--line)}
#msg{font-size:.85rem;color:var(--acc)}
.viz{display:grid;grid-template-columns:1fr 1fr;gap:1rem}
@media(max-width:800px){.viz{grid-template-columns:1fr}}
canvas{width:100%;background:#0a0a0c;border-radius:8px;border:1px solid var(--line)}
.legend{display:flex;flex-wrap:wrap;gap:.5rem 1rem;font-size:.72rem;color:var(--mut);margin-top:.4rem}
.depths{display:grid;grid-template-columns:repeat(5,1fr);gap:.35rem}
.dcell{display:flex;align-items:center;gap:.35rem;font-size:.7rem;color:var(--mut)}
.dbar{flex:1;height:8px;background:#222;border-radius:4px;overflow:hidden}
.dbar>i{display:block;height:100%;width:0%;background:var(--acc)}
.dbar.on>i{background:var(--warn)}
/* Force 5 rows × 4 columns */
#keygrid{
  display:grid;
  grid-template-columns:repeat(5,minmax(0,1fr));
  grid-template-rows:repeat(4,auto);
  gap:.5rem;
  width:100%;
}
.kcard{background:#101014;border:1px solid var(--line);border-radius:8px;padding:.5rem;font-size:.75rem}
.kcard .num{font-weight:700;color:var(--blue);font-size:.72rem;margin-bottom:.35rem;display:flex;justify-content:space-between}
.kcard .fields{display:grid;grid-template-columns:1fr 1fr;gap:.3rem}
.kcard label{font-size:.65rem}
.kcard select,.kcard input{width:100%;font-size:.72rem}
.kcard .modline{margin-top:.35rem;display:grid;grid-template-columns:1fr 1fr;gap:.3rem;align-items:start}
.kcard.axis{border-color:#2a4a3a}
.thumb-table{width:100%;border-collapse:collapse;font-size:.8rem}
.thumb-table th,.thumb-table td{padding:.35rem;border-bottom:1px solid var(--line);text-align:left;vertical-align:top}
.thumb-table select{min-width:7rem}
.hint{font-size:.75rem;color:var(--mut)}
</style>
</head>
<body>
<header><h1>Tartarus Pro</h1></header>
<main>
<section class="card">
  <h2>Response curve</h2>
  <div class="viz">
    <div>
      <canvas id="curveCanvas" width="480" height="260"></canvas>
      <div class="legend"><span>green = output · grey = 5% deadzone · orange = t_on · red = t_full · blue = live</span></div>
    </div>
    <div class="row">
      <label>t_on<input type="number" id="t_on" min="1" max="255"></label>
      <label>t_off<input type="number" id="t_off" min="0" max="254"></label>
      <label>t_full<input type="number" id="t_full" min="1" max="255"></label>
      <label>curve<select id="curve"><option>linear</option><option>expo</option><option>scurve</option></select></label>
      <label>gamma<input type="number" id="gamma" min="0.2" max="5" step="0.1"></label>
    </div>
    <ul class="hint" style="margin:.6rem 0 0;padding-left:1.1rem;line-height:1.45">
      <li><b>t_on</b> — depth where a key starts firing (press threshold).</li>
      <li><b>t_off</b> — depth where it releases (must be below t_on; hysteresis).</li>
      <li><b>t_full</b> — depth for dual-bind “full” action (top of travel).</li>
      <li><b>curve</b> — default for <em>all</em> axis binds unless a key overrides it.
        <b>linear</b> = proportional; <b>expo</b> = soft start (gamma); <b>scurve</b> = smooth S.</li>
      <li><b>gamma</b> — only for expo: &gt;1 softer start, &lt;1 snappier. Ignored for linear/scurve.</li>
      <li>Axes also use a fixed <b>5% deadzone</b> at each end of the depth range.</li>
    </ul>
  </div>
</section>

<section class="card">
  <h2>Live depth (4×5)</h2>
  <div class="depths" id="depths"></div>
</section>

<section class="card">
  <h2>Thumb / wheel</h2>
  <table class="thumb-table">
    <thead><tr><th>Control</th><th>Bind</th><th>Mod 1</th><th>Mod 2</th></tr></thead>
    <tbody>
      <tr><td>Pad up</td><td><select id="thumb_up"></select></td><td><select id="thumb_up_m1"></select></td><td><select id="thumb_up_m2"></select></td></tr>
      <tr><td>Pad down</td><td><select id="thumb_down"></select></td><td><select id="thumb_down_m1"></select></td><td><select id="thumb_down_m2"></select></td></tr>
      <tr><td>Pad left</td><td><select id="thumb_left"></select></td><td><select id="thumb_left_m1"></select></td><td><select id="thumb_left_m2"></select></td></tr>
      <tr><td>Pad right</td><td><select id="thumb_right"></select></td><td><select id="thumb_right_m1"></select></td><td><select id="thumb_right_m2"></select></td></tr>
      <tr><td>Thumb button</td><td><select id="thumb_button"></select></td><td><select id="thumb_button_m1"></select></td><td><select id="thumb_button_m2"></select></td></tr>
      <tr><td>Wheel up</td><td><select id="wheel_up"></select></td><td><select id="wheel_up_m1"></select></td><td><select id="wheel_up_m2"></select></td></tr>
      <tr><td>Wheel down</td><td><select id="wheel_down"></select></td><td><select id="wheel_down_m1"></select></td><td><select id="wheel_down_m2"></select></td></tr>
      <tr><td>Wheel click</td><td><select id="wheel_click"></select></td><td><select id="wheel_click_m1"></select></td><td><select id="wheel_click_m2"></select></td></tr>
    </tbody>
  </table>
</section>

<section class="card">
  <h2>Keys — 4 rows × 5 cols</h2>
  <div id="keygrid"></div>
</section>

<div class="row">
  <button id="save">Save</button>
  <button type="button" class="ghost" id="reload">Reload</button>
  <button type="button" class="ghost" id="export">Export</button>
  <button type="button" class="ghost" id="import">Import</button>
  <input type="file" id="importFile" accept=".json,application/json" style="display:none"/>
  <span id="msg"></span>
</div>
</main>
<script>
const KEYS=["","0","1","2","3","4","5","6","7","8","9","A","B","C","D","E","F","G","H","I","J","K","L","M","N","O","P","Q","R","S","T","U","V","W","X","Y","Z","LCTRL","RCTRL","LSHIFT","RSHIFT","LALT","RALT","LWIN","RWIN","F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12","F13","F14","F15","F16","F17","F18","F19","F20","F21","F22","F23","F24","UP","DOWN","LEFT","RIGHT","HOME","END","PAGEUP","PAGEDOWN","INSERT","DELETE","MEDIA_NEXT","MEDIA_PLAY_PAUSE","MEDIA_PREV","MEDIA_STOP","VOLUME_DOWN","VOLUME_MUTE","VOLUME_UP","APOSTROPHE","BACKSLASH","BACKSPACE","CAPSLOCK","COMMA","DOT","ENTER","EQUAL","ESC","GRAVE","KP0","KP1","KP2","KP3","KP4","KP5","KP6","KP7","KP8","KP9","KPASTERISK","KPDOT","KPENTER","KPMINUS","KPPLUS","KPSLASH","LEFTBRACE","MINUS","NUMLOCK","PAUSE","PRINT","RIGHTBRACE","SCROLLLOCK","SEMICOLON","SLASH","SPACE","TAB"];
const MODS=["","LCTRL","RCTRL","LSHIFT","RSHIFT","LALT","RALT","LWIN","RWIN"];
const AXES=["LX","LY","RX","RY","LT","RT","GAS","BRAKE","HAT_X","HAT_Y"];
const CURVES=[["","(global default)"],["linear","linear"],["expo","expo"],["scurve","scurve"]];
let cfg={keys:[]};
let liveDepths=new Array(20).fill(0);

function $(id){return document.getElementById(id)}
function opts(list,sel){return list.map(k=>{
  const v=Array.isArray(k)?k[0]:k, lab=Array.isArray(k)?k[1]:(k||'—');
  return `<option value="${v}" ${String(v)===(sel??'')?'selected':''}>${lab||'—'}</option>`;
}).join('')}
function modOpts(selected){
  const sel=selected||'';
  return MODS.map(m=>`<option value="${m}" ${m===sel?'selected':''}>${m||'—'}</option>`).join('');
}
function fill(id,v){const el=$(id); if(el) el.innerHTML=opts(KEYS,v)}
function fillMods(id,arr){const el=$(id); if(el) el.innerHTML=modOpts(arr)}
function selectedMods(id){return [...($(id)?.selectedOptions||[])].map(o=>o.value)}
function modsFromFlags(k){
  const a=[];
  if(k.lctrl)a.push('LCTRL'); if(k.rctrl)a.push('RCTRL');
  if(k.lshift)a.push('LSHIFT'); if(k.rshift)a.push('RSHIFT');
  if(k.lalt)a.push('LALT'); if(k.ralt)a.push('RALT');
  if(k.lwin)a.push('LWIN'); if(k.rwin)a.push('RWIN');
  return a;
}
function flagsFromMods(arr, onPartial, onFull){
  const s=new Set(arr||[]);
  const base={
    lctrl:s.has('LCTRL'),rctrl:s.has('RCTRL'),lshift:s.has('LSHIFT'),rshift:s.has('RSHIFT'),
    lalt:s.has('LALT'),ralt:s.has('RALT'),lwin:s.has('LWIN'),rwin:s.has('RWIN')
  };
  // UI stores one mod set; checkboxes decide if applied to partial and/or full
  return {mods: base, mods_partial: !!onPartial, mods_full_flag: !!onFull};
}

function shaped(depth,curve,gamma){
  const LO=0.05,HI=0.95; let x=depth/255,t=x<=LO?0:x>=HI?1:(x-LO)/(HI-LO);
  if(curve==='expo')t=Math.pow(t,gamma||2); else if(curve==='scurve')t=t*t*(3-2*t);
  return t;
}
function drawCurve(){
  const c=$('curveCanvas'),ctx=c.getContext('2d'),W=c.width,H=c.height,pad=32,pw=W-pad*2,ph=H-pad*2;
  ctx.clearRect(0,0,W,H);
  const ton=+$('t_on').value||100,tfull=+$('t_full').value||230,curve=$('curve').value,gamma=+$('gamma').value||2;
  ctx.fillStyle='#1a1a22'; ctx.fillRect(pad,pad,pw*0.05,ph); ctx.fillRect(pad+pw*0.95,pad,pw*0.05,ph);
  ctx.strokeStyle='#222'; for(let i=0;i<=4;i++){const x=pad+pw*i/4,y=pad+ph*i/4;ctx.beginPath();ctx.moveTo(x,pad);ctx.lineTo(x,pad+ph);ctx.stroke();ctx.beginPath();ctx.moveTo(pad,y);ctx.lineTo(pad+pw,y);ctx.stroke();}
  ctx.strokeStyle='#3dd68c';ctx.lineWidth=2;ctx.beginPath();
  for(let i=0;i<=255;i++){const x=pad+(i/255)*pw,y=pad+ph*(1-shaped(i,curve,gamma)); i?ctx.lineTo(x,y):ctx.moveTo(x,y);} ctx.stroke();
  function vl(d,col){const x=pad+(d/255)*pw;ctx.strokeStyle=col;ctx.setLineDash([4,3]);ctx.beginPath();ctx.moveTo(x,pad);ctx.lineTo(x,pad+ph);ctx.stroke();ctx.setLineDash([]);}
  vl(ton,'#f5a524'); vl(tfull,'#ff6b6b');
  const ld=Math.max(...liveDepths,0),lx=pad+(ld/255)*pw,ly=pad+ph*(1-shaped(ld,curve,gamma));
  ctx.fillStyle='#5b9cff';ctx.beginPath();ctx.arc(lx,ly,5,0,Math.PI*2);ctx.fill();
}

function keyCard(i,k){
  k=k||{};
  const mods=modsFromFlags(k);
  return `<div class="kcard ${k.mode==='axis'?'axis':''}" data-i="${i}">
    <div class="num"><span>KEY ${String(i+1).padStart(2,'0')}</span><span id="kl${i}">0</span></div>
    <div class="fields">
      <label>mode<select class="mode"><option value="key" ${k.mode!=='axis'?'selected':''}>key</option><option value="axis" ${k.mode==='axis'?'selected':''}>axis</option></select></label>
      <label>bind<select class="bind">${opts(KEYS,k.bind)}</select></label>
      <label>full<select class="bind_full">${opts(KEYS,k.bind_full||'')}</select></label>
      <label>axis<select class="axis">${opts(AXES,k.axis||'LX')}</select></label>
      <label>sign<select class="sign"><option value="1" ${(k.sign??1)>=0?'selected':''}>+1</option><option value="-1" ${k.sign<0?'selected':''}>-1</option></select></label>
      <label>curve<select class="curve">${opts(CURVES,k.curve||'')}</select></label>
    </div>
    <div class="modline">
      <label>mod 1<select class="mod1">${modOpts(mods[0]||'')}</select></label>
      <label>mod 2<select class="mod2">${modOpts(mods[1]||'')}</select></label>
      <label class="inline"><input type="checkbox" class="mods_on" ${(k.lctrl||k.rctrl||k.lshift||k.rshift||k.lalt||k.ralt||k.lwin||k.rwin)?'checked':''}> on normal</label>
      <label class="inline"><input type="checkbox" class="mods_on_full" ${k.mods_on_full?'checked':''}> on full</label>
    </div>
  </div>`;
}

function renderKeys(){
  const arr=cfg.keys||[];
  let h='';
  for(let row=0;row<4;row++) for(let col=0;col<5;col++){ const i=row*5+col; h+=keyCard(i,arr[i]); }
  $('keygrid').innerHTML=h;
}

function collectKeys(){
  return [...document.querySelectorAll('.kcard')].map(card=>{
    const mods=[card.querySelector('.mod1').value, card.querySelector('.mod2').value].filter(Boolean);
    const on=card.querySelector('.mods_on').checked;
    const onFull=card.querySelector('.mods_on_full').checked;
    const s=new Set(on?mods:[]);
    return {
      mode:card.querySelector('.mode').value,
      bind:card.querySelector('.bind').value,
      bind_full:card.querySelector('.bind_full').value,
      axis:card.querySelector('.axis').value,
      sign:+card.querySelector('.sign').value,
      t_on:null,t_off:null,t_full:null,
      curve:card.querySelector('.curve').value,
      gamma:+$('gamma').value||2,
      lctrl:s.has('LCTRL'),rctrl:s.has('RCTRL'),lshift:s.has('LSHIFT'),rshift:s.has('RSHIFT'),
      lalt:s.has('LALT'),ralt:s.has('RALT'),lwin:s.has('LWIN'),rwin:s.has('RWIN'),
      mods_on_full: onFull,
      // full uses same mod list when onFull
      _fullmods: onFull?mods:[]
    };
  }).map(k=>{
    if(k.mods_on_full){
      const s=new Set(k._fullmods);
      // encode full mods into same flags for now (driver uses mods for both when set)
      k.lctrl=k.lctrl||s.has('LCTRL'); k.rctrl=k.rctrl||s.has('RCTRL');
      k.lshift=k.lshift||s.has('LSHIFT'); k.rshift=k.rshift||s.has('RSHIFT');
      k.lalt=k.lalt||s.has('LALT'); k.ralt=k.ralt||s.has('RALT');
      k.lwin=k.lwin||s.has('LWIN'); k.rwin=k.rwin||s.has('RWIN');
    }
    delete k._fullmods;
    return k;
  });
}

function thumbModsPayload(id){
  const s=new Set(selectedMods(id));
  return {lctrl:s.has('LCTRL'),rctrl:s.has('RCTRL'),lshift:s.has('LSHIFT'),rshift:s.has('RSHIFT'),
    lalt:s.has('LALT'),ralt:s.has('RALT'),lwin:s.has('LWIN'),rwin:s.has('RWIN')};
}

async function load(){
  const j=await (await fetch('/api/config')).json();
  cfg=j;
  $('t_on').value=j.t_on; $('t_off').value=j.t_off; $('t_full').value=j.t_full||230;
  $('curve').value=j.curve||'linear'; $('gamma').value=j.gamma||2;
  const thumbs=[
    ['thumb_up','thumb_up_m1','thumb_up_m2',j.thumb_up,j.thumb_up_mods],
    ['thumb_down','thumb_down_m1','thumb_down_m2',j.thumb_down,j.thumb_down_mods],
    ['thumb_left','thumb_left_m1','thumb_left_m2',j.thumb_left,j.thumb_left_mods],
    ['thumb_right','thumb_right_m1','thumb_right_m2',j.thumb_right,j.thumb_right_mods],
    ['thumb_button','thumb_button_m1','thumb_button_m2',j.thumb_button||'F24',j.thumb_button_mods],
    ['wheel_up','wheel_up_m1','wheel_up_m2',j.wheel_up||'VOLUME_UP',j.wheel_up_mods],
    ['wheel_down','wheel_down_m1','wheel_down_m2',j.wheel_down||'VOLUME_DOWN',j.wheel_down_mods],
    ['wheel_click','wheel_click_m1','wheel_click_m2',j.wheel_click||'MENU',j.wheel_click_mods]];
  for(const [bid,m1,m2,v,m] of thumbs){
    fill(bid, v);
    const arr=m||[];
    $(m1).innerHTML=modOpts(arr[0]||'');
    $(m2).innerHTML=modOpts(arr[1]||'');
  }
  renderKeys(); drawCurve();
}

async function save(){
  const body={
    keys:collectKeys(), layer1: cfg.layer1||[],
    t_on:+$('t_on').value, t_off:+$('t_off').value, t_full:+$('t_full').value,
    curve:$('curve').value, gamma:+$('gamma').value,
    thumb_up:$('thumb_up').value, thumb_down:$('thumb_down').value,
    thumb_left:$('thumb_left').value, thumb_right:$('thumb_right').value,
    thumb_button:$('thumb_button').value,
    button_switches_layer:false,
    wheel_up:$('wheel_up').value, wheel_down:$('wheel_down').value, wheel_click:$('wheel_click').value||'MENU',
    thumb_up_mods:[$('thumb_up_m1').value,$('thumb_up_m2').value].filter(Boolean),
    thumb_down_mods:[$('thumb_down_m1').value,$('thumb_down_m2').value].filter(Boolean),
    thumb_left_mods:[$('thumb_left_m1').value,$('thumb_left_m2').value].filter(Boolean),
    thumb_right_mods:[$('thumb_right_m1').value,$('thumb_right_m2').value].filter(Boolean),
    thumb_button_mods:[$('thumb_button_m1').value,$('thumb_button_m2').value].filter(Boolean),
    wheel_up_mods:[$('wheel_up_m1').value,$('wheel_up_m2').value].filter(Boolean),
    wheel_down_mods:[$('wheel_down_m1').value,$('wheel_down_m2').value].filter(Boolean),
    wheel_click_mods:[$('wheel_click_m1').value,$('wheel_click_m2').value].filter(Boolean),
  };
  const r=await fetch('/api/config',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});
  $('msg').textContent=r.ok?'Saved':'Error: '+await r.text();
  if(r.ok) cfg.keys=body.keys;
}

function renderDepths(){
  if(!$('depths').children.length){
    let h=''; for(let i=0;i<20;i++) h+=`<div class="dcell"><span>${String(i+1).padStart(2,'0')}</span><div class="dbar" id="bar${i}"><i></i></div></div>`;
    $('depths').innerHTML=h;
  }
  const ton=+$('t_on').value||100;
  for(let i=0;i<20;i++){
    const d=liveDepths[i]||0, bar=$('bar'+i);
    bar.querySelector('i').style.width=(d/255*100)+'%';
    bar.classList.toggle('on',d>ton);
    const kl=$('kl'+i); if(kl) kl.textContent=d;
  }
}

function exportProfile(){
  const body={
    keys:collectKeys(), layer1: cfg.layer1||[],
    t_on:+$('t_on').value, t_off:+$('t_off').value, t_full:+$('t_full').value,
    curve:$('curve').value, gamma:+$('gamma').value,
    thumb_up:$('thumb_up').value, thumb_down:$('thumb_down').value,
    thumb_left:$('thumb_left').value, thumb_right:$('thumb_right').value,
    thumb_button:$('thumb_button').value,
    button_switches_layer:false,
    wheel_up:$('wheel_up').value, wheel_down:$('wheel_down').value, wheel_click:$('wheel_click').value||'MENU',
    thumb_up_mods:[$('thumb_up_m1').value,$('thumb_up_m2').value].filter(Boolean),
    thumb_down_mods:[$('thumb_down_m1').value,$('thumb_down_m2').value].filter(Boolean),
    thumb_left_mods:[$('thumb_left_m1').value,$('thumb_left_m2').value].filter(Boolean),
    thumb_right_mods:[$('thumb_right_m1').value,$('thumb_right_m2').value].filter(Boolean),
    thumb_button_mods:[$('thumb_button_m1').value,$('thumb_button_m2').value].filter(Boolean),
    wheel_up_mods:[$('wheel_up_m1').value,$('wheel_up_m2').value].filter(Boolean),
    wheel_down_mods:[$('wheel_down_m1').value,$('wheel_down_m2').value].filter(Boolean),
    wheel_click_mods:[$('wheel_click_m1').value,$('wheel_click_m2').value].filter(Boolean),
  };
  const blob=new Blob([JSON.stringify(body,null,2)],{type:'application/json'});
  const a=document.createElement('a');
  a.href=URL.createObjectURL(blob);
  a.download='tartarus-profile.json';
  a.click();
  URL.revokeObjectURL(a.href);
  $('msg').textContent='Exported';
}
function importProfile(){ $('importFile').click(); }
$('importFile').onchange=async e=>{
  const f=e.target.files[0]; if(!f) return;
  try{
    const j=JSON.parse(await f.text());
    cfg=j;
    $('t_on').value=j.t_on; $('t_off').value=j.t_off; $('t_full').value=j.t_full||230;
    $('curve').value=j.curve||'linear'; $('gamma').value=j.gamma||2;
    const thumbs=[
      ['thumb_up','thumb_up_m1','thumb_up_m2',j.thumb_up,j.thumb_up_mods],
      ['thumb_down','thumb_down_m1','thumb_down_m2',j.thumb_down,j.thumb_down_mods],
      ['thumb_left','thumb_left_m1','thumb_left_m2',j.thumb_left,j.thumb_left_mods],
      ['thumb_right','thumb_right_m1','thumb_right_m2',j.thumb_right,j.thumb_right_mods],
      ['thumb_button','thumb_button_m1','thumb_button_m2',j.thumb_button||'F24',j.thumb_button_mods],
      ['wheel_up','wheel_up_m1','wheel_up_m2',j.wheel_up||'VOLUME_UP',j.wheel_up_mods],
      ['wheel_down','wheel_down_m1','wheel_down_m2',j.wheel_down||'VOLUME_DOWN',j.wheel_down_mods],
      ['wheel_click','wheel_click_m1','wheel_click_m2',j.wheel_click||'MENU',j.wheel_click_mods]];
    for(const [bid,m1,m2,v,m] of thumbs){
      fill(bid, v);
      const arr=m||[];
      $(m1).innerHTML=modOpts(arr[0]||'');
      $(m2).innerHTML=modOpts(arr[1]||'');
    }
    renderKeys(); drawCurve();
    $('msg').textContent='Imported (Save to apply)';
  }catch(err){ $('msg').textContent='Import error: '+err; }
  e.target.value='';
};
$('save').onclick=save; $('reload').onclick=load;
$('export').onclick=exportProfile; $('import').onclick=importProfile;
['t_on','t_off','t_full','curve','gamma'].forEach(id=>{ $(id).addEventListener('input',drawCurve); });
async function poll(){ try{ const j=await(await fetch('/api/live')).json(); liveDepths=j.depths||liveDepths; renderDepths(); drawCurve(); }catch(e){} }
load(); setInterval(poll,50);
</script>
</body>
</html>
"#;

pub fn run() {
    let server = match Server::http(format!("127.0.0.1:{PORT}")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[webui] bind failed: {e}");
            return;
        }
    };
    println!("[webui] http://127.0.0.1:{PORT}/");
    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().clone();
        if method == Method::Get && (url == "/" || url.starts_with("/index")) {
            let mut resp = Response::from_string(HTML);
            resp.add_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }
        if method == Method::Get && url == "/api/config" {
            let body = serde_json::to_string(&UiPayload::from_cfg(&config::cfg()))
                .unwrap_or_else(|_| "{}".into());
            let mut resp = Response::from_string(body);
            resp.add_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }
        if method == Method::Get && url == "/api/live" {
            let body = serde_json::json!({
                "depths": state::get_depths().as_slice(),
                "layer": 0,
            })
            .to_string();
            let mut resp = Response::from_string(body);
            resp.add_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }
        if method == Method::Post && url == "/api/config" {
            let mut data = Vec::new();
            {
                let mut r = request.as_reader();
                std::io::Read::read_to_end(&mut r, &mut data).ok();
            }
            match serde_json::from_slice::<UiPayload>(&data) {
                Ok(p) => match p.save_toml() {
                    Ok(()) => {
                        config::set_cfg(p.to_cfg());
                        let _ = request.respond(Response::from_string("ok"));
                    }
                    Err(e) => {
                        let _ = request.respond(
                            Response::from_string(e.to_string()).with_status_code(StatusCode(500)),
                        );
                    }
                },
                Err(e) => {
                    let _ = request.respond(
                        Response::from_string(e.to_string()).with_status_code(StatusCode(400)),
                    );
                }
            }
            continue;
        }
        let _ = request.respond(Response::from_string("not found").with_status_code(StatusCode(404)));
    }
}
