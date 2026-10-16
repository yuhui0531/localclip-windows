// 完整正文编辑；草稿仅存内存，显式保存才更新原记录。by AI.Coding
import {useEffect, useRef, useState} from 'react';
import {call, Clip, message} from './api';
interface Draft {text:string;fingerprint:string;richText:boolean}
/** 文件引用和图片不显示编辑入口；后端进一步校验单项文本。 */
export function canEdit(clip:Clip){return (clip.kind==='text'||clip.kind==='url')&&clip.types.some(t=>['text/plain','text/uri','text/html','text/rtf'].includes(t));}
/** 多行编辑支持 IME，防止快捷键落到历史卡片并保留失败草稿。 */
export function TextEditor({id,close,saved}:{id:string;close:()=>void;saved:()=>void}){
 const [draft,setDraft]=useState<Draft>();const [text,setText]=useState('');const [error,setError]=useState('');const [busy,setBusy]=useState(false);
 const dialog=useRef<HTMLElement>(null);const input=useRef<HTMLTextAreaElement>(null);const saving=useRef(false);const alive=useRef(true);
 useEffect(()=>{alive.current=true;let active=true;void call<Draft>('edit_text',{id}).then(d=>{if(active){setDraft(d);setText(d.text);}}).catch(e=>{if(active)setError(message(e));});return()=>{active=false;alive.current=false;};},[id]);
 useEffect(()=>{if(draft)input.current?.focus();},[draft]);
 // 锁住焦点和 Tab；保存中正文只读但保持可聚焦，避免焦点落到历史操作。
 useEffect(()=>{function focus(e:FocusEvent){if(!dialog.current?.contains(e.target as Node)){(input.current??dialog.current)?.focus();}}document.addEventListener('focusin',focus);dialog.current?.focus();return()=>document.removeEventListener('focusin',focus);},[]);
 /** 乐观锁失败时保留草稿，避免覆盖另一处编辑。 */
 async function save(){if(!draft||saving.current)return;saving.current=true;setBusy(true);setError('');try{await call('edit_text',{id,fingerprint:draft.fingerprint,text});if(alive.current)saved();}catch(e){if(alive.current)setError(message(e));}finally{saving.current=false;if(alive.current)setBusy(false);}}
 return <div className="modal-shade edit-shade"><section ref={dialog} tabIndex={-1} className="text-editor" role="dialog" aria-modal="true" aria-label="编辑内容" onKeyDown={e=>{e.stopPropagation();if(e.nativeEvent.isComposing)return;if(e.key==='Escape'){e.preventDefault();if(!busy)close();}else if(e.ctrlKey&&e.key==='Enter'){e.preventDefault();if(text.length)void save();}else if(e.key==='Tab'){const controls=Array.from(dialog.current?.querySelectorAll<HTMLElement>('textarea:not(:disabled),button:not(:disabled)')??[]);const first=controls[0],last=controls.at(-1);if(e.shiftKey&&(document.activeElement===first||document.activeElement===dialog.current)){e.preventDefault();last?.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first?.focus();}}}}>
 <header><h2>编辑内容</h2><button disabled={busy} aria-label="取消编辑" onClick={close}>×</button></header>
 <p>{draft?.richText?'保存修改后将使用纯文本，原有富文本格式会被替换。':'保存后更新当前记录，保留标题、收藏和分类。'}</p>
 {draft?<textarea ref={input} aria-label="文本内容" spellCheck={false} value={text} readOnly={busy} onChange={e=>setText(e.target.value)}/>:<p>{error?'无法读取内容':'正在读取完整内容…'}</p>}
 {error&&<p className="edit-error" role="alert">{error}</p>}
 <footer><span>{text.length.toLocaleString()} 个字符 · Ctrl+Enter 保存</span><button disabled={busy} onClick={close}>取消</button><button className="primary" disabled={!draft||!text.length||busy} onClick={()=>void save()}>{busy?'正在保存…':'保存'}</button></footer>
 </section></div>;
}
