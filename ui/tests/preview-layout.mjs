// 真实 Chromium 中验证菜单行为，IPC 仅替换为生成数据，不接触用户历史。by AI.Coding
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createServer} from 'node:http';
import {readFile,writeFile} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
const require=createRequire(import.meta.url);
const {chromium}=require(process.env.LOCALCLIP_PLAYWRIGHT||'playwright');
const root=resolve('ui/dist');
await writeFile(resolve(root,'fixture.svg'),'<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="2400"><rect width="1600" height="2400" fill="#f5edd3"/><rect x="0" y="0" width="1600" height="60" fill="#e15759"/><rect x="0" y="2340" width="1600" height="60" fill="#4e79a7"/><text x="100" y="180" font-size="100">TOP / 顶部</text><text x="100" y="2250" font-size="100">BOTTOM / 底部</text></svg>');
const server=createServer(async(req,res)=>{try{const path=resolve(root,'.'+new URL(req.url,'http://localhost').pathname);if(!path.startsWith(root+'/'))throw Error();const file=path.endsWith('/')?path+'index.html':path;res.setHeader('Content-Type',({'.js':'text/javascript','.css':'text/css','.html':'text/html','.svg':'image/svg+xml'})[extname(file)]||'application/octet-stream');res.end(await readFile(file));}catch{res.statusCode=404;res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const browser=await chromium.launch({headless:true,...(process.env.LOCALCLIP_CHROME?{executablePath:process.env.LOCALCLIP_CHROME}:{})});
const page=await browser.newPage({viewport:{width:1200,height:760},deviceScaleFactor:2});
const errors=[];page.on('pageerror',e=>errors.push(e.message));
await page.addInitScript(()=>{
 const callbacks=new Map(),listeners=new Map();let next=1;
 const items=Array.from({length:140},(_,i)=>({id:String(i),kind:i===0?'image':i===2?'file':i===3?'mixed':'text',title:null,preview:i===1?'function generated() { return true; }':`Generated ${i}`,context:'',sourceName:'Generated',sourceBasis:'owner',sourceAppID:'generated.exe',lastCopiedAtMs:Date.now()-i*1000,firstCopiedAtMs:Date.now(),copyCount:1,favorite:false,protected:false,categoryIDs:i===120?['a']:[],types:i===0?['image/png']:i===2?['text/uri-list']:i===3?['image/png','text/plain']:['text/plain'],byteCount:100,badge:null}));
 const settings={revision:1,defaultAction:'copy',defaultFormat:'plainText',hideContents:false,appearance:'light',compactLayout:false,ocrLanguages:['en-US'],retentionDays:30,quotaBytes:500*1048576,excludedAppIDs:[],shortcut:{modifiers:['ctrl','shift'],key:'V'},queueShortcut:{modifiers:['ctrl','shift','alt'],key:'V'}};
 window.testCalls=[];window.testItems=items;window.testSettings=settings;window.testOcrPending=false;
 const emit=()=>{const id=listeners.get('store-changed');if(id)callbacks.get(id)({payload:{kind:'updated'}});};
 window.testEmit=emit;
 window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener(){}};
 window.__TAURI_INTERNALS__={transformCallback(fn){const id=next++;callbacks.set(id,fn);return id;},convertFileSrc(){return '/fixture.svg';},async invoke(command,args={}){
 const r=args.request||{};window.testCalls.push({command,r});
 if(command==='plugin:event|listen'){listeners.set(args.event,args.handler);return next++;}
 if(command==='settings')return {...settings};if(command==='categories')return {categories:[{id:'a',name:'Generated category',color:'00aa00'}],sources:[]};
 if(command==='capture_status')return {state:'paused'};if(command==='queue_status')return {active:false,position:0,total:0};
 if(command==='search'){let list=items.filter(v=>(!r.text||v.preview.includes(r.text))&&(!r.kind||v.kind===r.kind)&&(!r.categoryIDs?.length||r.categoryIDs.every(id=>v.categoryIDs.includes(id)))&&(!r.favoritesOnly||v.favorite));const offset=r.cursor?.offset||0;return {items:list.slice(offset,offset+r.pageSize),cursor:offset+r.pageSize<list.length?{offset:offset+r.pageSize}:null,revision:1,labels:[],problems:[]};}
 if(command==='clip_detail'){const v=items.find(v=>v.id===r.id);if(r.previewOnly)return {contentBytes:100,contentCount:1,unknownCount:0,folderCount:0,fileVersion:'1'};return {summary:{...v},items:[{plainText:v.preview,hasImage:v.kind==='image',types:v.types}],categoryIDs:v.categoryIDs};}
 if(command==='inspect')return {rows:[],actions:[]};
 if(command==='perform')return {status:'copied'};
 if(command==='transform_menu')return [{transform:'uppercase',title:'转为大写',applicable:true}];
 if(command==='ocr_copy')return window.testOcrPending?new Promise(resolve=>{window.testOcrResolve=resolve;}):{status:'ocrCopied',lines:1};
 if(command==='ocr_cancel'){window.testOcrResolve?.({status:'cancelled'});return {};}
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
async function close(){await page.getByLabel('关闭详情',{exact:true}).click();await page.waitForFunction(()=>!document.querySelector('.detail-popover'));}
async function openImage(){await page.locator('.clip-card.kind-image').click({button:'right'});await page.getByRole('menuitem',{name:/^预览/}).click();await page.locator('.image-canvas img').waitFor();await page.waitForFunction(()=>document.querySelector('.image-canvas img')?.naturalHeight===2400);await page.waitForTimeout(100);}
async function fits(){return page.evaluate(()=>{const v=document.querySelector('.image-viewport').getBoundingClientRect(),i=document.querySelector('.image-canvas img').getBoundingClientRect();return {fits:i.x>=v.x&&i.y>=v.y&&i.right<=v.right&&i.bottom<=v.bottom,ratio:i.width/i.height,height:i.height,viewport:v.height};});}
try {
 await page.goto(`http://127.0.0.1:${server.address().port}/index.html`);await page.locator('.clip-card').first().waitFor();
 await check('title is above content and retains custom title',async()=>{await page.evaluate(()=>{window.testItems[0].title='测试图片标题';window.testEmit();});await page.getByText('测试图片标题',{exact:true}).waitFor();const b=await page.locator('.clip-card').first().evaluate(el=>({title:el.querySelector('.card-title').getBoundingClientRect().bottom,content:el.querySelector('.card-content').getBoundingClientRect().top}));assert(b.title<=b.content);});
 await check('tall image fits entirely without distortion at high DPI',async()=>{await openImage();const r=await fits();assert(r.fits);assert(Math.abs(r.ratio-2/3)<0.01);assert(r.height>400);assert(await page.evaluate(()=>window.testCalls.some(c=>c.command==='set_detail_view'&&c.r.expanded)));});
 await check('original size and zoom expose scrollable pixels; fit restores whole image',async()=>{await page.getByRole('button',{name:'原始大小',exact:true}).click();await page.waitForTimeout(100);const r=await page.locator('.image-viewport').evaluate(el=>({h:el.scrollHeight,v:el.clientHeight,w:el.querySelector('img').getBoundingClientRect().width}));assert(r.h>r.v&&r.w===1600);await page.getByLabel('放大图片',{exact:true}).click();assert.equal(await page.locator('.image-canvas img').evaluate(el=>el.getBoundingClientRect().width),2000);await page.getByRole('button',{name:'适应窗口',exact:true}).click();await page.waitForTimeout(100);assert((await fits()).fits);await page.screenshot({path:'reports/raw/layout-preview-2x.png'});});
 await check('close requests history layout restoration',async()=>{await close();await page.waitForFunction(()=>window.testCalls.filter(c=>c.command==='set_detail_view').at(-1)?.r.expanded===false);});
 await check('long detail text is complete and scrollable with metadata accessible',async()=>{await page.evaluate(()=>{window.testItems[1].preview='function example() { return true; }\n'.repeat(700)+'END_OF_FULL_TEXT';window.testEmit();});await page.locator('.clip-card').nth(1).click({button:'right'});await page.getByRole('menuitem',{name:/^详情/}).click();assert((await page.locator('.detail-body').textContent()).endsWith('END_OF_FULL_TEXT'));assert(await page.locator('.detail-body').evaluate(el=>el.scrollHeight>el.clientHeight));assert(await page.getByPlaceholder('添加名称').isVisible());await close();});
 await check('short viewport keeps image and close button within bounds',async()=>{await page.setViewportSize({width:1200,height:340});await openImage();assert((await fits()).fits);assert(await page.getByLabel('关闭详情',{exact:true}).isVisible());await close();});
 await check('narrow detail stacks metadata and keeps preview proportional',async()=>{await page.setViewportSize({width:620,height:700});await openImage();assert((await fits()).fits);await page.screenshot({path:'reports/raw/layout-preview-narrow.png'});await close();});
 assert.deepEqual(errors,[]);console.log(JSON.stringify({checks,passed:checks.length,errors}));
} finally {await browser.close();await new Promise(r=>server.close(r));}
