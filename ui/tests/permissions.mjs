// 原生注册、构建清单和主窗口 ACL 必须一致；浏览器模拟 IPC 无法发现缺失权限。by AI.Coding
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
/** 从仓库相对路径读取声明，不依赖运行脚本时的工作目录。 */
const read=path=>readFile(new URL('../../'+path,import.meta.url),'utf8');
const [desktop,build,mainText,feedbackText]=await Promise.all(['app/src/desktop.rs','app/build.rs','app/capabilities/main.json','app/capabilities/feedback.json'].map(read));
const handlerBlock=desktop.match(/generate_handler!\[([\s\S]*?)\]/)?.[1];
assert(handlerBlock,'native invoke handler missing');
const registered=[...handlerBlock.matchAll(/crate::(?:commands|update)::([a-z_]+)/g)].map(v=>v[1]).sort();
const commandBlock=build.match(/\.commands\(&\[([\s\S]*?)\]\)/)?.[1];
assert(commandBlock,'native command manifest missing');
const declared=[...commandBlock.matchAll(/"([a-z_]+)"/g)].map(v=>v[1]).sort();
assert(registered.length>0,'registered command list missing');
assert.deepEqual(declared,registered,'registered commands must be included in the build manifest');
const main=JSON.parse(mainText),feedback=JSON.parse(feedbackText);
assert.deepEqual(main.permissions.filter(p=>p.startsWith('allow-')).sort(),registered.map(p=>'allow-'+p.replaceAll('_','-')).sort(),'main window must explicitly allow every registered application command');
assert(!feedback.permissions.some(p=>!p.startsWith('core:event:')),'feedback window must remain read-only event access');
console.log(`PASS native command manifest and window ACL consistency (${registered.length} commands)`);
