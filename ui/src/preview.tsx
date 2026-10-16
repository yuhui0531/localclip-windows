// 代码、文件与富文本预览；派生内容只保存在内存。by AI.Coding
import { useEffect, useState } from 'react';
import hljs from 'highlight.js/lib/common';
import swift from 'highlight.js/lib/languages/swift';
import { bytes, call, Clip, thumb } from './api';
hljs.registerLanguage('swift',swift);
const code = new Map<string,{html:string;language?:string}>();
const pictures = new Map<string,string>();
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
export function Thumbnail({clip,revision,fallback,info}:{clip:Clip;revision:number;fallback?:string;info?:FileInfo}) {
    const [src,setSrc]=useState<string>();const [failed,setFailed]=useState(false);
    useEffect(()=>{let live=true;let url:string|undefined;const abort=new AbortController();setFailed(false);setSrc(undefined);
      fetch(thumb(clip.id,revision),{signal:abort.signal}).then(r=>{if(!r.ok)throw new Error('unavailable');return r.blob();}).then(b=>{if(!live)return;url=URL.createObjectURL(b);setSrc(url);}).catch(()=>{if(live)setFailed(true);});
      return()=>{live=false;abort.abort();if(url)URL.revokeObjectURL(url);};
    },[clip.id,revision,info?.fileVersion]);
    const text=info?.previewText||fallback;const code=text?highlighted(text):undefined;
    return src?<img className="thumbnail" src={src} alt="内容预览"/>:failed&&text?(code?.language?<pre className="code-preview" dangerouslySetInnerHTML={{__html:code.html}}/>:<pre className="text-preview">{text}</pre>):<div className="image-placeholder">{failed?'预览不可用':'内容预览'}<span>{failed?'原始内容仍保留':'正在生成预览'}</span></div>;
}
export function TextPreview({text}:{text:string}) {const result=highlighted(text);return result.language?<pre className="code-preview" dangerouslySetInnerHTML={{__html:result.html}}/>:<pre>{text}</pre>;}
export function clearPreviewCache(){code.clear();for(const value of pictures.values())URL.revokeObjectURL(value);pictures.clear();}
export function RichPreview({html}:{html:string}){
    const safe=`<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; form-action 'none'; base-uri 'none'"><style>body{font:14px/1.5 'Segoe UI',sans-serif;overflow-wrap:anywhere;color:#20262e}img{max-width:100%}</style></head><body>${html}</body></html>`;
    return <iframe title="富文本预览" sandbox="" referrerPolicy="no-referrer" srcDoc={safe}/>;
}
