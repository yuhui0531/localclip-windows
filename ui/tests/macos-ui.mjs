// 真实 Chromium 中验证菜单行为，IPC 仅替换为生成数据，不接触用户历史。by AI.Coding
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
const require=createRequire(import.meta.url);
const {chromium}=require(process.env.LOCALCLIP_PLAYWRIGHT||'playwright');
const root=resolve('ui/dist');
const server=createServer(async(req,res)=>{try{const path=resolve(root,'.'+new URL(req.url,'http://localhost').pathname);if(!path.startsWith(root+'/'))throw Error();const file=path.endsWith('/')?path+'index.html':path;res.setHeader('Content-Type',({'.js':'text/javascript','.css':'text/css','.html':'text/html','.png':'image/png','.svg':'image/svg+xml'})[extname(file)]||'application/octet-stream');res.end(await readFile(file));}catch{res.statusCode=404;res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const browser=await chromium.launch({headless:true,...(process.env.LOCALCLIP_CHROME?{executablePath:process.env.LOCALCLIP_CHROME}:{})});
const page=await browser.newPage({viewport:{width:1440,height:340},deviceScaleFactor:2});
const errors=[];page.on('pageerror',e=>errors.push(e.message));
await page.addInitScript(()=>{
 const callbacks=new Map(),listeners=new Map();let next=1;
 const items=Array.from({length:140},(_,i)=>({id:String(i),kind:i===0?'image':i===2?'file':i===3?'mixed':'text',title:null,preview:i===1?'function generated() { return true; }':`Generated ${i}`,context:'',sourceName:'Generated',sourceBasis:'owner',sourceAppID:'generated.exe',lastCopiedAtMs:Date.now()-i*1000,firstCopiedAtMs:Date.now(),copyCount:1,favorite:false,protected:false,categoryIDs:i===120?['a']:[],types:i===0?['image/png']:i===2?['text/uri-list']:i===3?['image/png','text/plain']:['text/plain'],byteCount:100,badge:null}));
 const sample=[
 {kind:'text',title:'项目交付清单',preview:'设计评审、接口验证、验收记录。\n\n每一次复制，都能随时找回。',types:['text/plain']},
 {kind:'text',title:null,preview:'function previewCard(item) {\n  const title = item.title;\n  return { title, saved: true };\n}',types:['text/plain'],favorite:true},
 {kind:'image',title:null,preview:'',types:['image/png']},
 {kind:'file',title:'example.rs',preview:'C:\\Generated\\example.rs',body:'fn main() {\n    let message = "LocalClip";\n    println!("{}", message);\n}',types:['text/uri-list']},
 {kind:'url',title:null,preview:'https://example.com/design/localclip',types:['text/plain']},
 {kind:'mixed',title:'参考资料',preview:'图片与文本',types:['image/png','text/plain']}
 ];sample.forEach((v,i)=>Object.assign(items[i],v,{copyCount:i+1,sourceName:['记事本','Visual Studio Code','截图工具','文件资源管理器','浏览器','未知来源'][i]}));
 const settings={panelHeight:340,captureEnabled:false,smartCodeFilePaste:true,revision:1,defaultAction:'copy',defaultFormat:'plainText',hideContents:false,appearance:'light',compactLayout:false,ocrLanguages:['en-US'],retentionDays:30,quotaBytes:500*1048576,excludedAppIDs:[],shortcut:{modifiers:['ctrl','shift'],key:'V'},queueShortcut:{modifiers:['ctrl','shift','alt'],key:'V'}};
 window.testCalls=[];window.testItems=items;window.testSettings=settings;window.testOcrPending=false;
 const emit=()=>{const id=listeners.get('store-changed');if(id)callbacks.get(id)({payload:{kind:'updated'}});};
 window.testEmit=emit;window.testPanelShown=()=>{const id=listeners.get('panel-shown');if(id)callbacks.get(id)({payload:{}});};
 window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};
 window.__TAURI_INTERNALS__={transformCallback(fn){const id=next++;callbacks.set(id,fn);return id;},convertFileSrc(value){return '/'+value;},async invoke(command,args={}){
 const r=args.request||{};window.testCalls.push({command,r});
 if(command==='plugin:event|listen'){listeners.set(args.event,args.handler);return next++;}
 if(command==='settings')return {...settings};if(command==='categories')return {categories:[{id:'a',name:'常用提示词',color:'34c759'},{id:'b',name:'常用网址',color:'007aff'},{id:'c',name:'常用账户',color:'ff9500'}],sources:[{id:'generated.exe',name:'生成的测试应用'}]};
 if(command==='capture_status')return {state:'paused'};if(command==='queue_status')return {active:false,position:0,total:0};
 if(command==='search'){let list=items.filter(v=>(!r.text||v.preview.includes(r.text))&&(!r.kind||v.kind===r.kind)&&(!r.categoryIDs?.length||r.categoryIDs.every(id=>v.categoryIDs.includes(id)))&&(!r.favoritesOnly||v.favorite));const offset=r.cursor?.offset||0;return {items:list.slice(offset,offset+r.pageSize),cursor:offset+r.pageSize<list.length?{offset:offset+r.pageSize}:null,revision:1,labels:[],problems:[]};}
 if(command==='usage')return {totalBytes:20*1048576,protectedBytes:2*1048576,ordinaryBytes:18*1048576,dragImageBytes:r.clearDragImages?0:1048576,walBytes:4096,freeDiskBytes:50*1024*1048576};
 if(command==='clip_detail'){const v=items.find(v=>v.id===r.id);if(r.previewOnly)return {contentBytes:100,contentCount:1,unknownCount:0,folderCount:0,fileVersion:'1',previewText:v.body??v.preview,characterCount:['text','url'].includes(v.kind)?[...new Intl.Segmenter(undefined,{granularity:'grapheme'}).segment(v.body??v.preview)].length:undefined};return {summary:{...v},items:[{plainText:v.preview,hasImage:v.kind==='image',types:v.types}],categoryIDs:v.categoryIDs};}
 if(command==='inspect')return {rows:[],actions:[]};
 if(command==='perform')return {status:'copied'};
 if(command==='transform_menu')return [{transform:'uppercase',title:'转为大写',applicable:true}];
 if(command==='ocr_copy')return window.testOcrPending?new Promise(resolve=>{window.testOcrResolve=resolve;}):{status:'ocrCopied',lines:1};
 if(command==='ocr_cancel'){window.testOcrResolve?.({status:'cancelled'});return {};}
 // 编辑模拟仅用于浏览器事件验证；存储事务另由 Rust 集成测试覆盖。
 if(command==='edit_text'){
  const v=items.find(v=>v.id===r.id);
  if(r.text===undefined)return {text:v.fullText??v.preview,fingerprint:'generated-version',richText:Boolean(v.richText)};
  if(window.testEditError)throw window.testEditError;if(window.testEditPending)await new Promise(resolve=>window.testResolveEdit=resolve);
  v.fullText=r.text;v.preview=r.text.slice(0,180);emit();return {saved:true};
 }
 if(command==='set_title'){items.find(v=>v.id===r.id).title=r.value;emit();return {};}
 if(command==='set_favorite'){items.find(v=>v.id===r.id).favorite=r.value;emit();return {};}
 if(command==='assign_categories')return {token:'assign',needsConfirmation:false};
 if(command==='apply_change'){items[1].categoryIDs=['a'];emit();return {};}
 if(command==='open_item'){if(r.index===undefined)return [{index:0,label:'C:/generated.txt',file:true}];return {opened:true};}
 if(command==='save_as')return {saved:true};
 if(command==='delete'){items.splice(items.findIndex(v=>v.id===r.ids[0]),1);emit();return {undoAvailable:true};}
 if(command==='update_check')return {downloaded:false,installationEnabled:false};
 if(command==='update_settings'){Object.assign(settings,r);return {...settings};}
 return {};
 }};
});
const checks=[];
async function check(name,fn){await fn();checks.push(name);console.log('PASS '+name);}
async function screenshot(name){await page.screenshot({path:`reports/raw/macos-ui-${name}.png`});}
async function more(){await page.getByRole('button',{name:'更多',exact:true}).click();}
async function settings(){await more();await page.getByRole('menuitem',{name:'设置…',exact:true}).click();await page.getByLabel('设置',{exact:true}).waitFor();}
async function theme(value){await page.evaluate(value=>{window.testSettings.appearance=value;window.testEmit();},value);await page.waitForFunction(value=>document.documentElement.dataset.theme===value,value);}
const image='<svg xmlns="http://www.w3.org/2000/svg" width="900" height="600"><rect width="900" height="600" fill="#f3eee7"/><rect x="80" y="80" width="740" height="440" rx="20" fill="#fff"/><rect x="120" y="130" width="250" height="24" rx="5" fill="#6fae9a"/><rect x="120" y="184" width="530" height="13" rx="5" fill="#dfe5e1"/><rect x="120" y="216" width="460" height="13" rx="5" fill="#dfe5e1"/><circle cx="660" cy="390" r="72" fill="#80b6a2"/><rect x="555" y="335" width="88" height="110" rx="20" fill="#efc087"/></svg>';
try{
 await page.route('**/thumb/*',route=>/\/thumb\/(2|5)(?:\?|$)/.test(route.request().url())?route.fulfill({contentType:'image/svg+xml',body:image}):route.fulfill({status:404,body:''}));
 await page.goto(`http://127.0.0.1:${server.address().port}/index.html`);await page.locator('.clip-card').first().waitFor();await page.waitForTimeout(150);
 await check('one toolbar and square cards follow macOS geometry',async()=>{const v=await page.evaluate(()=>{const main=document.querySelector('.app'),toolbar=document.querySelector('.toolbar').getBoundingClientRect(),card=document.querySelector('.clip-card').getBoundingClientRect(),nav=document.querySelector('.filter-bar').getBoundingClientRect(),search=document.querySelector('.search').getBoundingClientRect();return {w:card.width,h:card.height,toolbar:toolbar.height,sameRow:nav.top>=toolbar.top&&nav.bottom<=toolbar.bottom&&search.bottom<=toolbar.bottom,rootFits:main.scrollWidth<=innerWidth};});assert(Math.abs(v.w-v.h)<1);assert.equal(v.toolbar,48);assert(v.sameRow&&v.rootFits);await page.locator('.clip-card').nth(1).locator('.hljs-keyword').first().waitFor();await screenshot('light');});
 await check('title time source and copy metadata match card hierarchy',async()=>{const card=page.locator('.clip-card').first();assert.equal(await card.locator('.card-title').textContent(),'项目交付清单');assert(await card.locator('.card-header time').isVisible());assert((await card.locator('.card-footer .card-size').textContent()).endsWith('个字符'));assert((await page.locator('.clip-card').nth(2).locator('.card-size').textContent()).startsWith('复制3次')); assert(await card.locator('.source-badge').getAttribute('aria-label'));assert.equal(await page.locator('.clip-card').nth(1).locator('.card-title').textContent(),'JavaScript');await card.locator('.card-header').dblclick();await page.getByRole('dialog',{name:'重命名'}).waitFor();await page.keyboard.press('Escape');assert.equal(await page.evaluate(()=>window.testCalls.filter(v=>v.command==='perform').length),0);});
 await check('source/date filters remain reachable and clearable in popover',async()=>{await page.getByRole('button',{name:'筛选',exact:true}).click();await page.getByLabel('筛选来源应用').selectOption('generated.exe');await page.getByLabel('开始日期',{exact:true}).fill('2026-10-01');await page.waitForFunction(()=>window.testCalls.some(v=>v.command==='search'&&v.r.sourceAppID==='generated.exe'&&v.r.from));await page.keyboard.press('Escape');assert(await page.locator('.prefix-line').isVisible());await page.locator('.prefix-line').getByRole('button',{name:'清除筛选'}).click();await page.waitForFunction(()=>!document.querySelector('.prefix-line'));});
 await check('toolbar more keeps monitoring and privacy actions',async()=>{await more();await page.getByRole('menuitem',{name:'开始监控',exact:true}).click();assert(await page.evaluate(()=>window.testCalls.some(v=>v.command==='resume')));await more();await page.getByRole('menuitem',{name:'隐藏内容',exact:true}).click();await page.locator('.hidden-content').first().waitFor();assert.equal(await page.locator('.code-preview').count(),0);await more();await page.getByRole('menuitem',{name:'显示内容',exact:true}).click();await page.waitForFunction(()=>!document.querySelector('.hidden-content'));});
 await check('footer actions keep copy format and content editing shortcuts',async()=>{await page.locator('.clip-card').first().click();await page.getByRole('button',{name:'分类',exact:true}).click();assert.equal(await page.getByRole('menuitemcheckbox').count(),3);assert.equal(await page.getByRole('menuitem',{name:'原格式复制',exact:true}).count(),0);await page.keyboard.press('Escape');await page.getByRole('button',{name:'纯文本复制',exact:true}).click();assert.equal(await page.evaluate(()=>window.testCalls.filter(v=>v.command==='perform').at(-1).r.format),'plainText');await page.getByRole('button',{name:'编辑所选内容',exact:true}).focus();await page.keyboard.press('Space');await page.getByRole('dialog',{name:'编辑内容'}).waitFor();assert.equal(await page.getByLabel('条目预览').count(),0);await page.keyboard.press('Escape');await page.locator('.clip-card').first().focus();await page.keyboard.press('Control+e');await page.getByLabel('文本内容').fill('Generated changed content');await page.keyboard.press('Control+s');await page.getByRole('dialog',{name:'编辑内容'}).waitFor({state:'detached'});assert.equal(await page.evaluate(()=>window.testItems[0].fullText),'Generated changed content');});
 await check('settings uses top tabs and retains cache cleanup confirmation',async()=>{await page.setViewportSize({width:1000,height:760});await settings();const rect=await page.locator('.settings>aside').boundingBox();assert(rect.width>900&&rect.height<90);await page.locator('.settings>aside').getByRole('button',{name:'存储',exact:true}).click();await page.getByText('拖出图片文件',{exact:true}).waitFor();await page.getByRole('button',{name:'清理…',exact:true}).click();await page.getByRole('alertdialog').waitFor();await page.getByRole('button',{name:'取消',exact:true}).click();assert(!await page.evaluate(()=>window.testCalls.some(v=>v.command==='usage'&&v.r.clearDragImages)));await page.getByRole('status').filter({hasText:'内容已保存'}).waitFor({state:'detached'});await screenshot('settings');await page.getByLabel('关闭设置').click();});
 await check('dark appearance has readable syntax colors and neutral surfaces',async()=>{await page.setViewportSize({width:1440,height:340});await theme('dark');const c=await page.locator('.clip-card').nth(1).locator('.hljs-keyword').first().evaluate(el=>({token:getComputedStyle(el).color,body:getComputedStyle(el.closest('pre')).color}));assert.notEqual(c.token,c.body);await screenshot('dark');await page.locator('.clip-card').nth(2).click({button:'right'});await page.getByRole('menu').waitFor();await screenshot('menu-dark');await page.keyboard.press('Escape');});
 await check('compact toolbar retains types categories search and all panel controls',async()=>{await page.setViewportSize({width:960,height:340});await page.getByLabel('类型',{exact:true}).waitFor({state:'visible'});await page.getByLabel('类型',{exact:true}).selectOption('image');await page.waitForFunction(()=>window.testCalls.some(v=>v.command==='search'&&v.r.kind==='image'));await page.getByRole('button',{name:'全部历史',exact:true}).click();const r=await page.locator('.toolbar').evaluate(el=>({width:el.scrollWidth,viewport:innerWidth}));assert(r.width<=r.viewport);for(const label of ['筛选','更多','置顶面板','关闭面板'])assert(await page.getByRole('button',{name:label,exact:true}).isVisible());await screenshot('compact');});
 await check('medium-height cards retain their time and quick-key metadata',async()=>{await page.setViewportSize({width:960,height:280});await page.waitForFunction(()=>document.querySelector('.app.dense')&&!document.querySelector('.app.tiny-cards'));assert(await page.locator('.clip-card').first().locator('.card-header time').isVisible());assert(await page.locator('.clip-card').first().locator('.card-number').isVisible());});
 await check('short panel keeps complete square card boundaries and footer',async()=>{await theme('light');await page.setViewportSize({width:960,height:220});await page.waitForFunction(()=>document.querySelector('.app.tiny-cards'));const r=await page.locator('.clip-card').first().boundingBox();const footer=await page.locator('.status-bar').boundingBox();assert(Math.abs(r.width-r.height)<1&&r.y+r.height<=footer.y);assert(footer.y+footer.height<=220);await screenshot('short');});
 await check('narrow panel has no toolbar overflow and filters remain usable',async()=>{await page.setViewportSize({width:620,height:340});const r=await page.locator('.toolbar').evaluate(el=>({width:el.scrollWidth,viewport:innerWidth}));assert(r.width<=r.viewport);await page.getByRole('button',{name:'筛选',exact:true}).click();const f=await page.getByRole('dialog',{name:'筛选历史'}).boundingBox();assert(f.x>=0&&f.x+f.width<=620);await page.keyboard.press('Escape');await screenshot('narrow');});
 await check('narrow multi-selection keeps merge queue and primary actions in view',async()=>{await page.locator('.clip-card').first().click();await page.locator('.clip-card').nth(1).click({modifiers:['Meta']});await page.getByLabel('合并方式').waitFor();const r=await page.locator('.status-bar').evaluate(el=>({scroll:el.scrollWidth,width:el.clientWidth}));assert(r.scroll<=r.width);assert(await page.getByRole('button',{name:'加入粘贴队列',exact:true}).isVisible());await page.locator('.clip-card').first().click();});
 await check('forced compact preference and taller panel retain proportional card geometry',async()=>{await page.setViewportSize({width:1440,height:520});await page.evaluate(()=>{window.testSettings.compactLayout=true;window.testSettings.panelHeight=520;window.testEmit();});await page.waitForFunction(()=>document.querySelector('.clip-card')?.getBoundingClientRect().width>390);assert(await page.getByLabel('类型',{exact:true}).isVisible());const r=await page.locator('.clip-card').first().boundingBox();assert(Math.abs(r.width-r.height)<1&&r.width>390);await screenshot('tall-compact');});
 assert.deepEqual(errors,[]);console.log(JSON.stringify({checks,passed:checks.length,errors}));
}finally{await browser.close();await new Promise(r=>server.close(r));}
