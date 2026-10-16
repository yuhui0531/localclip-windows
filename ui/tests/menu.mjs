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
 window.testCalls=[];window.testItems=items;window.testSettings=settings;window.testOcrPending=false;
 const emit=()=>{const id=listeners.get('store-changed');if(id)callbacks.get(id)({payload:{kind:'updated'}});};
 window.testEmit=emit;window.testPanelShown=()=>{const id=listeners.get('panel-shown');if(id)callbacks.get(id)({payload:{}});};
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
 // 保留隐藏历史的布局，避免晚页记录在编辑窗口缩小时跳回开头。by AI.Coding
 await check('closing compact editor retains late history selection and scroll',async()=>{
  const selected=page.locator('.clip-card.selected');const before=await selected.textContent();const left=await page.locator('.history').evaluate(el=>el.scrollLeft);assert(left>1000);
  await selected.click({button:'right'});await choose('编辑内容');await page.getByLabel('文本内容').waitFor();await page.setViewportSize({width:560,height:390});await page.waitForTimeout(100);await page.getByRole('button',{name:'取消',exact:true}).click();await page.setViewportSize({width:1200,height:340});await page.waitForTimeout(100);
  assert.equal(await page.locator('.clip-card.selected').textContent(),before);assert(Math.abs(await page.locator('.history').evaluate(el=>el.scrollLeft)-left)<1);
 });
 await check('locate category clears previous filters',async()=>{await page.locator('.clip-card.selected').click({button:'right'});await menu.getByRole('menuitem',{name:'在分类中定位'}).click();await menu.getByRole('menuitem',{name:'Generated category',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.filter-bar .category-filter.active')&&document.querySelector('.clip-card.selected')?.textContent.includes('Generated 120'));});
 await check('Space preview toggle and Ctrl+O open',async()=>{await page.locator('.clip-card.selected').click();await page.keyboard.press('Space');await page.getByLabel('条目预览').waitFor();await page.keyboard.press('Escape');await page.locator('.clip-card.selected').focus();await page.keyboard.press('Control+o');await page.waitForFunction(()=>window.testCalls.some(v=>v.command==='open_item'&&v.r.index===0));});
 await check('save action wired',async()=>{await page.locator('.clip-card.selected').click({button:'right'});await choose('存储为文件…');await page.waitForFunction(()=>window.testCalls.some(v=>v.command==='save_as'));});
 await check('mixed filter and category plus destination',async()=>{await page.getByRole('button',{name:'全部历史',exact:true}).click();await page.getByRole('button',{name:'混合',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.clip-card')?.classList.contains('kind-mixed'));await page.getByLabel('管理分类').click();await page.locator('.settings h2').filter({hasText:'分类'}).waitFor();await page.getByLabel('关闭设置').click();});
 await check('file menu omits unsupported Save As and category creation remains available',async()=>{await page.getByRole('button',{name:'全部历史',exact:true}).click();await page.locator('.clip-card.kind-file').click({button:'right'});assert.equal(await menu.getByRole('menuitem',{name:'存储为文件…'}).count(),0);await menu.getByRole('menuitem',{name:'加入分类'}).click();await menu.getByRole('button',{name:'新建分类…'}).click();await page.locator('.settings h2').filter({hasText:'分类'}).waitFor();await page.getByLabel('关闭设置').click();});
 await check('OCR progress cancellation and hidden-content guard',async()=>{await page.getByRole('button',{name:'全部历史',exact:true}).click();await page.evaluate(()=>{window.testOcrPending=true;});await page.locator('.clip-card.kind-image').click({button:'right'});await choose('复制图中文字');await page.locator('.ocr-progress').waitFor();await page.locator('.ocr-progress').getByRole('button',{name:'取消',exact:true}).click();await page.waitForFunction(()=>!document.querySelector('.ocr-progress'));assert(await page.evaluate(()=>window.testCalls.some(v=>v.command==='ocr_cancel')));await page.evaluate(()=>{window.testOcrPending=false;window.testSettings.hideContents=true;window.testEmit();});await page.locator('.hidden-content').first().waitFor();await page.locator('.clip-card.kind-image').click({button:'right'});assert(await menu.getByRole('menuitem',{name:'复制图中文字',exact:true}).isDisabled());await page.keyboard.press('Escape');await page.evaluate(()=>{window.testSettings.hideContents=false;window.testEmit();});await page.waitForFunction(()=>!document.querySelector('.hidden-content'));});
 await check('edit entry excludes images/files and opens full text from code card',async()=>{
  await page.getByRole('button',{name:'全部历史',exact:true}).click();await card(0).click({button:'right'});assert.equal(await menu.getByRole('menuitem',{name:'编辑内容',exact:true}).count(),0);await page.keyboard.press('Escape');
  await page.evaluate(()=>{window.testItems[1].fullText='// generated line\n'.repeat(100)+'const answer = 42;';window.testItems[1].richText=true;});
  await card(1).click({button:'right'});await choose('编辑内容');await page.getByRole('dialog',{name:'编辑内容'}).waitFor();
  assert((await page.getByLabel('文本内容').inputValue()).endsWith('const answer = 42;'));assert(await page.getByText('保存修改后将使用纯文本，原有富文本格式会被替换。').isVisible());
  await page.setViewportSize({width:1200,height:800});const rect=await page.getByRole('dialog',{name:'编辑内容'}).boundingBox();assert(rect.height>=350&&rect.height<=430&&rect.y+rect.height<=800);assert((await page.getByLabel('文本内容').boundingBox()).height>=200);await page.screenshot({path:'reports/raw/edit018-browser-editor.png'});
 });
 // 编辑只显示自己的表面；标题拖动交给原生窗口，正文和按钮不得启动拖动。by AI.Coding
 await check('editor hides underlying history and requests compact native layout',async()=>{
  assert(await page.getByRole('dialog',{name:'编辑内容'}).isVisible());assert(!(await page.locator('.toolbar').isVisible()));assert(!(await page.locator('.history').isVisible()));assert(!(await page.locator('.status-bar').isVisible()));
  assert(await page.evaluate(()=>{const r=window.testCalls.filter(v=>v.command==='set_detail_view').at(-1).r;return r.expanded&&r.editor;}));
  await page.setViewportSize({width:560,height:390});await page.waitForTimeout(100);const rect=await page.getByRole('dialog',{name:'编辑内容'}).boundingBox();assert(rect.x>=0&&rect.y>=0&&rect.x+rect.width<=560&&rect.y+rect.height<=390);assert(await page.getByRole('button',{name:'保存',exact:true}).isVisible());await page.screenshot({path:'reports/raw/editor0111-browser.png'});
  await page.setViewportSize({width:1200,height:800});
 });
 await check('only primary title-bar dragging starts native window movement',async()=>{
  const count=()=>page.evaluate(()=>window.testCalls.filter(v=>v.command==='start_window_drag').length);
  const before=await count();const title=await page.locator('.editor-titlebar h2').boundingBox();await page.mouse.move(title.x+10,title.y+10);await page.mouse.down();await page.mouse.move(title.x+60,title.y+30);await page.mouse.up();assert.equal(await count(),before+1);
  await page.getByLabel('文本内容').click();await page.getByLabel('取消编辑',{exact:true}).dispatchEvent('pointerdown',{button:0,isPrimary:true,bubbles:true});await page.locator('.editor-titlebar').dispatchEvent('pointerdown',{button:2,isPrimary:true,bubbles:true});assert.equal(await count(),before+1);
 });
 await check('editing keyboard and IME cannot trigger history actions; cancel preserves content',async()=>{
  const before=await page.evaluate(()=>window.testCalls.filter(v=>['perform','delete'].includes(v.command)).length);
  await page.getByLabel('文本内容').fill('cancelled draft');await page.evaluate(()=>window.testPanelShown());await page.waitForFunction(()=>document.activeElement?.getAttribute('aria-label')==='文本内容');assert.equal(await page.getByLabel('文本内容').inputValue(),'cancelled draft');assert(await page.evaluate(()=>window.testCalls.filter(v=>v.command==='set_detail_view').at(-1).r.expanded));await page.keyboard.press('Enter');await page.keyboard.press('Delete');await page.keyboard.press('Control+f');assert(await page.getByLabel('文本内容').evaluate(el=>el===document.activeElement));
  await page.getByLabel('文本内容').dispatchEvent('keydown',{key:'Escape',isComposing:true,bubbles:true});assert(await page.getByRole('dialog',{name:'编辑内容'}).isVisible());
  await page.getByLabel('文本内容').dispatchEvent('keydown',{key:'Enter',ctrlKey:true,isComposing:true,bubbles:true});assert.equal(await page.evaluate(()=>window.testCalls.filter(v=>v.command==='edit_text'&&v.r.text!==undefined).length),0);
  await page.keyboard.press('Tab');await page.keyboard.press('Tab');await page.keyboard.press('Tab');assert(await page.getByRole('dialog',{name:'编辑内容'}).evaluate(el=>el.contains(document.activeElement)));
  await page.getByRole('button',{name:'取消',exact:true}).click();assert((await page.evaluate(()=>window.testItems[1].fullText)).endsWith('const answer = 42;'));assert.equal(await page.evaluate(()=>window.testCalls.filter(v=>['perform','delete'].includes(v.command)).length),before);
  await page.waitForFunction(()=>{const r=window.testCalls.filter(v=>v.command==='set_detail_view').at(-1).r;return !r.editor&&!r.expanded;});assert(await page.locator('.toolbar').isVisible());assert(await page.locator('.history').isVisible());
 });
 await check('failed save retains draft; successful save preserves metadata and refreshes highlighting',async()=>{
  await card(1).click({button:'right'});await choose('编辑内容');await page.getByLabel('文本内容').fill('');assert(await page.getByRole('button',{name:'保存',exact:true}).isDisabled());
  await page.getByLabel('文本内容').fill('function edited() { return 42; }');await page.evaluate(()=>{window.testEditError='DuplicateContent';});await page.keyboard.press('Control+Enter');await page.getByRole('alert').filter({hasText:'历史中已存在相同内容'}).waitFor();assert.equal(await page.getByLabel('文本内容').inputValue(),'function edited() { return 42; }');
  await page.evaluate(()=>{window.testEditError=null;});await page.keyboard.press('Control+Enter');await page.getByRole('dialog',{name:'编辑内容'}).waitFor({state:'detached'});await card(1).locator('.hljs-keyword').first().waitFor();
  assert.deepEqual(await page.evaluate(()=>{const v=window.testItems[1];return [v.title,v.favorite,v.categoryIDs,v.fullText];}),['Renamed generated code',true,['a'],'function edited() { return 42; }']);
 });
 await check('details edit entry and hiding contents clears unsaved draft',async()=>{
  await card(1).click({button:'right'});await menu.getByRole('menuitem',{name:/^详情/}).click();await page.getByLabel('条目详情').getByRole('button',{name:'编辑内容',exact:true}).click();await page.getByLabel('文本内容').fill('private draft');
  await page.evaluate(()=>{window.testSettings.hideContents=true;window.testEmit();});await page.getByRole('dialog',{name:'编辑内容'}).waitFor({state:'detached'});await page.locator('.hidden-content').first().waitFor();await card(1).click({button:'right'});assert(await menu.getByRole('menuitem',{name:'编辑内容',exact:true}).isDisabled());await page.keyboard.press('Escape');
  await page.evaluate(()=>{window.testSettings.hideContents=false;window.testEmit();});await page.waitForFunction(()=>!document.querySelector('.hidden-content'));assert.equal(await page.evaluate(()=>window.testItems[1].fullText),'function edited() { return 42; }');await page.setViewportSize({width:1200,height:340});
 });
 await check('save in flight keeps keyboard inside editor',async()=>{await card(1).click({button:'right'});await choose('编辑内容');await page.getByLabel('文本内容').fill('function pending() { return 42; }');await page.evaluate(()=>window.testEditPending=true);await page.getByRole('button',{name:'保存',exact:true}).click();await page.keyboard.press('Tab');assert(await page.getByRole('dialog',{name:'编辑内容'}).evaluate(el=>el.contains(document.activeElement)));await page.evaluate(()=>window.testResolveEdit());await page.getByRole('dialog',{name:'编辑内容'}).waitFor({state:'detached'});});
 await check('delete wired and code highlight retained',async()=>{await page.getByRole('button',{name:'全部历史',exact:true}).click();await page.locator('.code-preview .hljs-keyword').first().waitFor();await card(3).click({button:'right'});await menu.getByRole('menuitem',{name:/^删除/}).click();await page.waitForFunction(()=>window.testCalls.some(v=>v.command==='delete'));});
 assert.deepEqual(errors,[]);console.log(JSON.stringify({checks,passed:checks.length,errors}));
}finally{await browser.close();await new Promise(r=>server.close(r));}
