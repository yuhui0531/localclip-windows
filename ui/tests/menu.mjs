// 真实 Chromium 中验证菜单行为，IPC 仅替换为生成数据，不接触用户历史。by AI.Coding
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
const require=createRequire(import.meta.url);
const {chromium}=require(process.env.LOCALCLIP_PLAYWRIGHT||'playwright');
const root=resolve('ui/dist');
const server=createServer(async(req,res)=>{try{const path=resolve(root,'.'+new URL(req.url,'http://localhost').pathname);if(!path.startsWith(root+'/'))throw Error();const file=path.endsWith('/')?path+'index.html':path;res.setHeader('Content-Type',({'.js':'text/javascript','.css':'text/css','.html':'text/html'})[extname(file)]||'application/octet-stream');res.end(await readFile(file));}catch{res.statusCode=404;res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const browser=await chromium.launch({headless:true,...(process.env.LOCALCLIP_CHROME?{executablePath:process.env.LOCALCLIP_CHROME}:{})});
const page=await browser.newPage({viewport:{width:1200,height:340}});
const errors=[];page.on('pageerror',e=>errors.push(e.message));
await page.addInitScript(()=>{
 const callbacks=new Map(),listeners=new Map();let next=1;
 const items=Array.from({length:140},(_,i)=>({id:String(i),kind:i===0?'image':i===2?'file':i===3?'mixed':'text',title:null,preview:i===1?'function generated() { return true; }':`Generated ${i}`,context:'',sourceName:'Generated',sourceBasis:'owner',sourceAppID:'generated.exe',lastCopiedAtMs:Date.now()-i*1000,firstCopiedAtMs:Date.now(),copyCount:1,favorite:false,protected:false,categoryIDs:i===120?['a']:[],types:i===0?['image/png']:i===2?['text/uri-list']:i===3?['image/png','text/plain']:['text/plain'],byteCount:100,badge:null}));
 const settings={revision:1,defaultAction:'copy',defaultFormat:'plainText',hideContents:false,appearance:'light',compactLayout:false,ocrLanguages:['en-US'],retentionDays:30,quotaBytes:500*1048576,excludedAppIDs:[],shortcut:{modifiers:['ctrl','shift'],key:'V'},queueShortcut:{modifiers:['ctrl','shift','alt'],key:'V'}};
 window.testCalls=[];window.testItems=items;window.testSettings=settings;
 const emit=()=>{const id=listeners.get('store-changed');if(id)callbacks.get(id)({payload:{kind:'updated'}});};
 window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};
 window.__TAURI_INTERNALS__={transformCallback(fn){const id=next++;callbacks.set(id,fn);return id;},convertFileSrc(){return '/missing.png';},async invoke(command,args={}){
 const r=args.request||{};window.testCalls.push({command,r});
 if(command==='plugin:event|listen'){listeners.set(args.event,args.handler);return next++;}
 if(command==='settings')return {...settings};if(command==='categories')return {categories:[{id:'a',name:'Generated category',color:'00aa00'}],sources:[]};
 if(command==='capture_status')return {state:'paused'};if(command==='queue_status')return {active:false,position:0,total:0};
 if(command==='search'){let list=items.filter(v=>(!r.text||v.preview.includes(r.text))&&(!r.kind||v.kind===r.kind)&&(!r.categoryIDs?.length||r.categoryIDs.every(id=>v.categoryIDs.includes(id)))&&(!r.favoritesOnly||v.favorite));const offset=r.cursor?.offset||0;return {items:list.slice(offset,offset+r.pageSize),cursor:offset+r.pageSize<list.length?{offset:offset+r.pageSize}:null,revision:1,labels:[],problems:[]};}
 if(command==='clip_detail'){const v=items.find(v=>v.id===r.id);if(r.previewOnly)return {contentBytes:100,contentCount:1,unknownCount:0,folderCount:0,fileVersion:'1'};return {summary:{...v},items:[{plainText:v.preview,hasImage:v.kind==='image',types:v.types}],categoryIDs:v.categoryIDs};}
 if(command==='inspect')return {rows:[],actions:[]};
 if(command==='perform')return {status:'copied'};
 if(command==='transform_menu')return [{transform:'uppercase',title:'转为大写',applicable:true}];
 if(command==='ocr_copy')return {status:'ocrCopied',lines:1};
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
const menu=page.getByRole('menu');
const card=i=>page.locator('.clip-card').nth(i);
const choose=label=>menu.getByRole('menuitem',{name:label,exact:true}).click();
const search=page.getByLabel('搜索剪贴板历史');
try{
 await page.goto(`http://127.0.0.1:${server.address().port}/index.html`);await card(0).waitFor();
 await check('right-click menu anchored and bounded; screenshot actions present',async()=>{await card(0).click({button:'right'});await menu.waitFor();const r=await menu.boundingBox();assert(r.x<400&&r.x>=0&&r.y>=0&&r.y+r.height<=340);for(const label of ['原格式复制','复制图中文字','重命名','收藏','加入分类','在历史中定位','在分类中定位','详情','预览','打开','存储为文件…','删除'])assert(await menu.locator('button').filter({hasText:label}).count());});
 await check('original copy overrides plain-text default',async()=>{await choose('原格式复制');assert.deepEqual(await page.evaluate(()=>window.testCalls.filter(v=>v.command==='perform').at(-1).r.format),'original');});
 await check('OCR reachable from card menu and detail',async()=>{await card(0).click({button:'right'});await choose('复制图中文字');await page.getByRole('status').filter({hasText:'已复制图中文字'}).waitFor();await card(0).click({button:'right'});await menu.getByRole('menuitem',{name:/^详情/}).click();await page.getByLabel('条目详情').getByRole('button',{name:'复制图中文字'}).click();assert.equal(await page.evaluate(()=>window.testCalls.filter(v=>v.command==='ocr_copy').length),2);await page.keyboard.press('Escape');});
 await check('rename saves selected card',async()=>{await card(1).click({button:'right'});await choose('重命名');await page.getByLabel('卡片名称').fill('Renamed generated code');await page.getByRole('button',{name:'保存',exact:true}).click();await page.getByText('Renamed generated code',{exact:true}).waitFor();});
 await check('favorite and category assignment',async()=>{await card(1).click({button:'right'});await choose('收藏');await page.waitForFunction(()=>window.testItems[1].favorite);await menu.getByRole('menuitem',{name:'加入分类'}).click();await menu.getByRole('menuitemcheckbox').click();await page.waitForFunction(()=>window.testItems[1].categoryIDs.includes('a'));await page.keyboard.press('Escape');});
 await check('transform explicitly pastes even with copy default',async()=>{await card(1).click({button:'right'});await menu.getByRole('menuitem',{name:/转换后粘贴/}).click();await menu.getByRole('menuitem',{name:'转为大写'}).click();assert.equal(await page.evaluate(()=>window.testCalls.filter(v=>v.command==='perform').at(-1).r.action),'directPaste');});
 await check('locate history paginates beyond first page and retains selected ID',async()=>{await search.fill('Generated 120');await page.waitForFunction(()=>document.querySelectorAll('.clip-card').length===1);await card(0).click({button:'right'});await menu.getByRole('menuitem',{name:/在历史中定位/}).click();await page.waitForFunction(()=>document.querySelector('.clip-card.selected')?.textContent.includes('Generated 120'));assert((await page.locator('.history').evaluate(el=>el.scrollLeft))>1000);assert.equal(await search.inputValue(),'');});
 await check('locate category clears previous filters',async()=>{await page.locator('.clip-card.selected').click({button:'right'});await menu.getByRole('menuitem',{name:'在分类中定位'}).click();await menu.getByRole('menuitem',{name:'Generated category',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.filter-bar .category-filter.active')&&document.querySelector('.clip-card.selected')?.textContent.includes('Generated 120'));});
 await check('Space preview toggle and Ctrl+O open',async()=>{await page.locator('.clip-card.selected').click();await page.keyboard.press('Space');await page.getByLabel('条目预览').waitFor();await page.keyboard.press('Escape');await page.locator('.clip-card.selected').focus();await page.keyboard.press('Control+o');await page.waitForFunction(()=>window.testCalls.some(v=>v.command==='open_item'&&v.r.index===0));});
 await check('save action wired',async()=>{await page.locator('.clip-card.selected').click({button:'right'});await choose('存储为文件…');await page.waitForFunction(()=>window.testCalls.some(v=>v.command==='save_as'));});
 await check('mixed filter and category plus destination',async()=>{await page.getByRole('button',{name:'全部历史',exact:true}).click();await page.getByRole('button',{name:'混合',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.clip-card')?.classList.contains('kind-mixed'));await page.getByLabel('管理分类').click();await page.locator('.settings h2').filter({hasText:'分类'}).waitFor();await page.getByLabel('关闭设置').click();});
 await check('file menu omits unsupported Save As and category creation remains available',async()=>{await page.getByRole('button',{name:'全部历史',exact:true}).click();await page.locator('.clip-card.kind-file').click({button:'right'});assert.equal(await menu.getByRole('menuitem',{name:'存储为文件…'}).count(),0);await menu.getByRole('menuitem',{name:'加入分类'}).click();await menu.getByRole('button',{name:'新建分类…'}).click();await page.locator('.settings h2').filter({hasText:'分类'}).waitFor();await page.getByLabel('关闭设置').click();});
 await check('delete wired and code highlight retained',async()=>{await page.getByRole('button',{name:'全部历史',exact:true}).click();await page.locator('.code-preview .hljs-keyword').first().waitFor();await card(3).click({button:'right'});await menu.getByRole('menuitem',{name:/^删除/}).click();await page.waitForFunction(()=>window.testCalls.some(v=>v.command==='delete'));});
 assert.deepEqual(errors,[]);console.log(JSON.stringify({checks,passed:checks.length,errors}));
}finally{await browser.close();await new Promise(r=>server.close(r));}
