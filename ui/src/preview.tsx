// 代码、文件与富文本预览；派生内容只保存在内存。by AI.Coding
import { useEffect, useRef, useState } from 'react';
import hljs from 'highlight.js/lib/common';
import swift from 'highlight.js/lib/languages/swift';
import { bytes, call, Clip, thumb } from './api';
hljs.registerLanguage('swift',swift);
const code = new Map<string,{html:string;language?:string}>();
// 有界检测样本排除路径、链接和普通中文正文，真实代码仍使用原文着色。by AI.Coding
export function highlighted(text:string):{html:string;language?:string} {
    const cached=code.get(text);if(cached)return cached;
    const prefix=text.slice(0,16_384);const sample=prefix.slice(0,2048);
    const plain={html:prefix.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')};
    const lines=sample.trim().split(/\r?\n/);
    const path=(line:string)=>/^(?:[A-Za-z]:[\\/]|\\\\|\/|~\/|file:\/\/)/.test(line.trim());
    const prose=(sample.match(/[\u3400-\u9fff]/g)?.length??0)>sample.length/3;
    let value:{html:string;language?:string}=plain;
    if(sample.trim()&&!lines.every(path)&&!(lines.length===1&&/^(?:https?:|mailto:)/i.test(sample.trim()))&&!(prose&&!/[{};=<>]/.test(sample))) {
      const detection=hljs.highlightAuto(lines.filter(line=>!path(line)).join('\n'), ['json','java','python','javascript','typescript','yaml','xml','css','bash','swift','sql','cpp','csharp','go','ruby','php','rust','ini','markdown']);
      if(detection.language&&detection.relevance>=3)value={html:hljs.highlight(prefix,{language:detection.language,ignoreIllegals:true}).value,language:detection.language};
    }
    if(code.size>=160)code.delete(code.keys().next().value!);code.set(text,value);return value;
}
export interface FileInfo {contentBytes:number;contentCount:number;unknownCount:number;folderCount:number;fileVersion:string;previewText?:string}
// 只为实际挂载的可见卡片读取元数据；取消后不发布内容。
export function useFileInfo(clip:Clip,revision:number,hidden=false) {
 const [info,setInfo]=useState<FileInfo>();
 useEffect(()=>{let live=true;setInfo(undefined);if(hidden||!clip.types.some(t=>t==='text/uri-list'||t.startsWith('image/')))return;
  void call<FileInfo>('clip_detail',{id:clip.id,previewOnly:true}).then(v=>{if(live)setInfo(v);}).catch(()=>{});return()=>{live=false;};
 },[clip.id,revision,hidden]);return info;
}
export function sizeLabel(info?:FileInfo) {if(!info||!info.contentCount)return '';const partial=info.unknownCount+info.folderCount;return partial?(partial===info.contentCount?'大小未知':`已知 ${bytes(info.contentBytes)} · 部分未知`):`${info.contentCount>1?info.contentCount+' 项 · ':''}${bytes(info.contentBytes)}`;}
/** 预览请求共用取消和 URL 回收；详情单独请求原始像素，不放大小缩略图。 */
function usePreviewSource(clip:Clip,revision:number,info:FileInfo|undefined,full=false) {
 const [src,setSrc]=useState<string>();const [failed,setFailed]=useState(false);
 useEffect(()=>{let live=true;let url:string|undefined;const abort=new AbortController();setFailed(false);setSrc(undefined);
  fetch(thumb(clip.id,revision)+(full?'&view=full':''),{signal:abort.signal}).then(r=>{if(!r.ok)throw new Error('unavailable');return r.blob();}).then(b=>{if(!live)return;url=URL.createObjectURL(b);setSrc(url);}).catch(()=>{if(live)setFailed(true);});
  return()=>{live=false;abort.abort();if(url)URL.revokeObjectURL(url);};
 },[clip.id,revision,info?.fileVersion,full]);return {src,failed};
}
/** 卡片保留轻量缩略图和代码文件回退预览。 */
export function Thumbnail({clip,revision,fallback,info}:{clip:Clip;revision:number;fallback?:string;info?:FileInfo}) {
 const {src,failed}=usePreviewSource(clip,revision,info);
 const text=info?.previewText||fallback;
 return src?<img className="thumbnail" src={src} alt="内容预览"/>:failed&&text?<TextPreview text={text}/>:<div className="image-placeholder">{failed?'预览不可用':'内容预览'}<span>{failed?'原始内容仍保留':'正在生成预览'}</span></div>;
}
/** 图片查看器默认完整适配，原始比例与手动缩放可通过滚动或拖动画布检查细节。by AI.Coding */
export function ImagePreview({clip,revision,fallback,info}:{clip:Clip;revision:number;fallback?:string;info?:FileInfo}) {
 const {src,failed}=usePreviewSource(clip,revision,info,true);
 const viewport=useRef<HTMLDivElement>(null);
 const pan=useRef<{x:number;y:number;left:number;top:number}|undefined>(undefined);
 const [natural,setNatural]=useState({width:0,height:0});
 const [area,setArea]=useState({width:0,height:0});
 const [zoom,setZoom]=useState<number|null>(null);
 useEffect(()=>{setNatural({width:0,height:0});setZoom(null);},[src]);
 useEffect(()=>{const el=viewport.current;if(!el)return;const observer=new ResizeObserver(()=>setArea({width:el.clientWidth,height:el.clientHeight}));observer.observe(el);return()=>observer.disconnect();},[src]);
 const fit=natural.width?Math.min(1,Math.max(1,area.width-32)/natural.width,Math.max(1,area.height-32)/natural.height):1;
 const scale=zoom??fit;
 // 缩放后回到中心，避免长图停留在旧滚动位置而看不到图像。
 useEffect(()=>{const el=viewport.current;if(el){el.scrollLeft=(el.scrollWidth-el.clientWidth)/2;el.scrollTop=(el.scrollHeight-el.clientHeight)/2;}},[scale]);
 const text=info?.previewText||fallback;
 if(!src)return failed&&text?<TextPreview text={text}/>:<div className="image-placeholder">{failed?'预览不可用':'正在加载图片'}<span>原始内容仍保留</span></div>;
 return <div className="image-viewer">
  <div className="image-tools" role="toolbar" aria-label="图片缩放">
   <button className={zoom===null?'active':''} onClick={()=>setZoom(null)}>适应窗口</button>
   <button className={zoom===1?'active':''} onClick={()=>setZoom(1)}>原始大小</button>
   <button aria-label="缩小图片" disabled={scale<=0.05} onClick={()=>setZoom(Math.max(0.05,scale/1.25))}>−</button>
   <span aria-live="polite">{Math.round(scale*100)}%</span>
   <button aria-label="放大图片" disabled={scale>=4} onClick={()=>setZoom(Math.min(4,scale*1.25))}>＋</button>
   <small>{natural.width?`${natural.width} × ${natural.height}`:''}</small>
  </div>
  <div className="image-viewport" ref={viewport} tabIndex={0} aria-label="图片画布"
   onPointerDown={e=>{if(e.button!==0)return;const el=e.currentTarget;pan.current={x:e.clientX,y:e.clientY,left:el.scrollLeft,top:el.scrollTop};el.setPointerCapture(e.pointerId);}}
   onPointerMove={e=>{const start=pan.current;if(start){e.currentTarget.scrollLeft=start.left+start.x-e.clientX;e.currentTarget.scrollTop=start.top+start.y-e.clientY;}}}
   onPointerUp={e=>{pan.current=undefined;e.currentTarget.releasePointerCapture(e.pointerId);}} onLostPointerCapture={()=>{pan.current=undefined;}}>
   <div className="image-canvas" style={{width:Math.max(area.width,natural.width*scale+32),height:Math.max(area.height,natural.height*scale+32)}}>
    <img src={src} alt="完整图片预览" draggable={false} onLoad={e=>setNatural({width:e.currentTarget.naturalWidth,height:e.currentTarget.naturalHeight})} style={{width:natural.width*scale,height:natural.height*scale}}/>
   </div>
  </div>
 </div>;
}
/** 详情必须保留全文；超出高亮预算时用完整纯文本，避免静默截断。 */
export function TextPreview({text}:{text:string}) {const result=text.length<=16_384?highlighted(text):undefined;return result?.language?<pre className="code-preview" dangerouslySetInnerHTML={{__html:result.html}}/>:<pre>{text}</pre>;}
export function clearPreviewCache(){code.clear();}
export function RichPreview({html}:{html:string}){
    const safe=`<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; form-action 'none'; base-uri 'none'"><style>body{font:14px/1.5 'Segoe UI',sans-serif;overflow-wrap:anywhere;color:#20262e}img{max-width:100%}</style></head><body>${html}</body></html>`;
    return <iframe title="富文本预览" sandbox="" referrerPolicy="no-referrer" srcDoc={safe}/>;
}
