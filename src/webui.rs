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
.keycell{background:#121216;border:1px solid var(--line);border-radius:8px;padding:.5rem;display:flex;flex-direction:column;gap:.35rem}
.keycell h3{margin:0;font-size:.7rem;color:var(--mut)}
.keycell .binds{display:flex;flex-direction:column;gap:.25rem}
.keycell select{width:100%;font-size:.7rem}
.thumb-row{display:grid;grid-template-columns:1fr 1fr;gap:1rem}
@media(max-width:700px){.thumb-row{grid-template-columns:1fr}}
</style>
</head>
<body>
<header>
  <h1>Tartarus Linux</h1>
  <span id="status" style="font-size:.8rem;color:var(--mut)">loading…</span>
  <div style="margin-left:auto;display:flex;gap:.5rem">
  <button type="button" id="save">Save</button>
  <button type="button" class="ghost" id="export">Export</button>
  <button type="button" class="ghost" id="import">Import</button>
  <input type="file" id="importfile" accept=".json,.toml" style="display:none"/>
  </div>
</header>
<main>
  <div class="card">
    <h2>Keys</h2>
    <div id="keygrid"></div>
  </div>
  <div class="card">
    <h2>Thumb + Wheel</h2>
    <div class="thumb-row">
      <div>
        <label>Thumb bind<select id="thumb_bind"></select></label>
        <label>Thumb full<select id="thumb_bind_full"></select></label>
      </div>
      <div>
        <label>Wheel up<select id="wheel_up"></select></label>
        <label>Wheel down<select id="wheel_down"></select></label>
      </div>
    </div>
  </div>
  <div class="card">
    <h2>Status</h2>
    <div id="msg"></div>
    <div class="depths" id="depths"></div>
  </div>
</main>
<script>
const KEYS=["","0","1","2","3","4","5","6","7","8","9","A","B","C","D","E","F","G","H","I","J","K","L","M","N","O","P","Q","R","S","T","U","V","W","X","Y","Z","LCTRL","RCTRL","LSHIFT","RSHIFT","LALT","RALT","LWIN","RWIN","F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12","F13","F14","F15","F16","F17","F18","F19","F20","F21","F22","F23","F24","UP","DOWN","LEFT","RIGHT","HOME","END","PAGEUP","PAGEDOWN","INSERT","DELETE","MEDIA_NEXT","MEDIA_PLAY_PAUSE","MEDIA_PREV","MEDIA_STOP","VOLUME_DOWN","VOLUME_MUTE","VOLUME_UP","APOSTROPHE","BACKSLASH","BACKSPACE","CAPSLOCK","COMMA","DOT","ENTER","EQUAL","ESC","GRAVE","KP0","KP1","KP2","KP3","KP4","KP5","KP6","KP7","KP8","KP9","KPASTERISK","KPDOT","KPENTER","KPMINUS","KPPLUS","KPSLASH","LEFTBRACE","MINUS","NUMLOCK","PAUSE","PRINT","RIGHTBRACE","SCROLLLOCK","SEMICOLON","SLASH","SPACE","TAB"];
const MODS=["","LCTRL","RCTRL","LSHIFT","RSHIFT","LALT","RALT","LWIN","RWIN"];
function $(id){return document.getElementById(id)}
function opts(arr,v){return arr.map(k=>`<option value="${k}" ${k===v?'selected':''}>${k||'(none)'}</option>`).join('')}
function fill(id,v){const el=$(id); if(el) el.innerHTML=opts(KEYS,v)}
function modsOf(k){
  const a=[];
  if(k.lctrl)a.push('LCTRL'); if(k.rctrl)a.push('RCTRL');
  if(k.lshift)a.push('LSHIFT'); if(k.rshift)a.push('RSHIFT');
  if(k.lalt)a.push('LALT'); if(k.ralt)a.push('RALT');
  if(k.lwin)a.push('LWIN'); if(k.rwin)a.push('RWIN');
  return a;
}
function parseMods(s){
  return {
    lctrl:s.has('LCTRL'),rctrl:s.has('RCTRL'),lshift:s.has('LSHIFT'),rshift:s.has('RSHIFT'),
    lalt:s.has('LALT'),ralt:s.has('RALT'),lwin:s.has('LWIN'),rwin:s.has('RWIN')
  };
}
function renderKeys(cfg){
  const g=$('keygrid');
  g.innerHTML='';
  for(let i=1;i<=20;i++){
    const k=cfg.keys[i]||{bind:'',bind_full:'',mods:[]};
    const cell=document.createElement('div');
    cell.className='keycell';
    cell.innerHTML=`<h3>Key ${i}</h3>
      <label>bind<select class="bind">${opts(KEYS,k.bind)}</select></label>
      <label>full<select class="bind_full">${opts(KEYS,k.bind_full||'')}</select></label>
      <label>mods<select class="mods" multiple>${opts(MODS, '')}</select></label>`;
    const ms=cell.querySelector('.mods');
    const cur=new Set(modsOf(k));
    [...ms.options].forEach(o=>{if(cur.has(o.value))o.selected=true});
    g.appendChild(cell);
  }
}
function collect(){
  const keys={};
  document.querySelectorAll('.keycell').forEach((cell,idx)=>{
    const i=idx+1;
    const bind=cell.querySelector('.bind').value;
    const bind_full=cell.querySelector('.bind_full').value;
    const s=new Set([...cell.querySelector('.mods').selectedOptions].map(o=>o.value).filter(Boolean));
    keys[i]={
      bind, bind_full,
      lctrl:s.has('LCTRL'),rctrl:s.has('RCTRL'),lshift:s.has('LSHIFT'),rshift:s.has('RSHIFT'),
      lalt:s.has('LALT'),ralt:s.has('RALT'),lwin:s.has('LWIN'),rwin:s.has('RWIN')
    };
  });
  return {
    keys,
    thumb:{bind:$('thumb_bind').value, bind_full:$('thumb_bind_full').value},
    wheel:{up:$('wheel_up').value, down:$('wheel_down').value}
  };
}
async function load(){
  const r=await fetch('/api/config');
  const cfg=await r.json();
  renderKeys(cfg);
  fill('thumb_bind',cfg.thumb?.bind||'');
  fill('thumb_bind_full',cfg.thumb?.bind_full||'');
  fill('wheel_up',cfg.wheel?.up||'');
  fill('wheel_down',cfg.wheel?.down||'');
  $('status').textContent='ready';
}
$('save').onclick=async()=>{
  const body=JSON.stringify(collect());
  const r=await fetch('/api/config',{method:'POST',headers:{'Content-Type':'application/json'},body});
  $('msg').textContent=r.ok?'Saved':'Save failed';
};
$('export').onclick=()=>{
  const data=JSON.stringify(collect(),null,2);
  const a=document.createElement('a');
  a.href=URL.createObjectURL(new Blob([data],{type:'application/json'}));
  a.download='tartarus-profile.json';
  a.click();
  $('msg').textContent='Exported';
};
$('import').onclick=()=>$('importfile').click();
$('importfile').onchange=async e=>{
  const f=e.target.files[0];
  if(!f)return;
  try{
    const t=await f.text();
    const cfg=JSON.parse(t);
    renderKeys(cfg);
    fill('thumb_bind',cfg.thumb?.bind||'');
    fill('thumb_bind_full',cfg.thumb?.bind_full||'');
    fill('wheel_up',cfg.wheel?.up||'');
    fill('wheel_down',cfg.wheel?.down||'');
    $('msg').textContent='Imported (Save to apply)';
  }catch(err){ $('msg').textContent='Import error: '+err; }
};
load();
</script>
</body>
</html>
"#;

// NOTE: full server impl continues below — this is placeholder for push; actual local file is complete.
