// 完整正文编辑；草稿仅存内存，显式保存才更新原记录。by AI.Coding
import {useEffect, useRef, useState} from 'react';
import {call, Clip, message} from './api';
import {WindowDialog} from './WindowDialog';
interface Draft {text:string;fingerprint:string;richText:boolean}
/** 文件引用和图片不显示编辑入口；后端进一步校验单项文本。 */
export function canEdit(clip:Clip){return (clip.kind==='text'||clip.kind==='url')&&clip.types.some(t=>['text/plain','text/uri','text/html','text/rtf'].includes(t));}
/** 多行编辑支持 IME，防止快捷键落到历史卡片并保留失败草稿。 */
export function TextEditor({id,close,saved}:{id:string;close:()=>void;saved:()=>void}){
 const [draft,setDraft]=useState<Draft>();const [text,setText]=useState('');const [error,setError]=useState('');const [busy,setBusy]=useState(false);
 const dialog=useRef<HTMLElement>(null);const input=useRef<HTMLTextAreaElement>(null);const saving=useRef(false);const alive=useRef(true);
 useEffect(()=>{alive.current=true;let active=true;void call<Draft>('edit_text',{id}).then(d=>{if(active){setDraft(d);setText(d.text);}}).catch(e=>{if(active)setError(message(e));});return()=>{active=false;alive.current=false;};},[id]);
 useEffect(()=>{if(draft)input.current?.focus();},[draft]);
 /** 乐观锁失败时保留草稿，避免覆盖另一处编辑。 */
 async function save(){if(!draft||saving.current)return;saving.current=true;setBusy(true);setError('');try{await call('edit_text',{id,fingerprint:draft.fingerprint,text});if(alive.current)saved();}catch(e){if(alive.current)setError(message(e));}finally{saving.current=false;if(alive.current)setBusy(false);}}
 // 弹窗壳负责系统拖动、缩放和焦点环绕；这里只处理编辑专属快捷键。
 return <div className="modal-shade edit-shade dialog-layer"><WindowDialog dialogRef={dialog} className="text-editor" label="编辑内容" title="编辑内容" closeLabel="取消编辑" close={close} busy={busy} onError={setError} onKeyDown={e=>{e.stopPropagation();if(e.nativeEvent.isComposing)return;if(e.ctrlKey&&(e.key==='Enter'||e.key.toLowerCase()==='s')){e.preventDefault();if(text.length)void save();}}}>
 <p>{draft?.richText?'保存修改后将使用纯文本，原有富文本格式会被替换。':'保存后更新当前记录，保留标题、收藏和分类。'}</p>
 {draft?<textarea ref={input} aria-label="文本内容" spellCheck={false} value={text} readOnly={busy} onChange={e=>setText(e.target.value)}/>:<p>{error?'无法读取内容':'正在读取完整内容…'}</p>}
 {error&&<p className="edit-error" role="alert">{error}</p>}
 <footer><span>{text.length.toLocaleString()} 个字符 · Ctrl+S 保存 · Esc 取消</span><button disabled={busy} onClick={close}>取消</button><button className="primary" disabled={!draft||!text.length||busy} onClick={()=>void save()}>{busy?'正在保存…':'保存'}</button></footer>
 </WindowDialog></div>;
}
