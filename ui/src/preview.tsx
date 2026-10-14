import { useEffect, useState } from 'react';
import hljs from 'highlight.js/lib/common';
import swift from 'highlight.js/lib/languages/swift';
import { Clip, thumb } from './api';
hljs.registerLanguage('swift',swift);
const code = new Map<string,{html:string;language?:string}>();
const pictures = new Map<string,string>();
export function highlighted(text:string):{html:string;language?:string} {
    const cached=code.get(text);if(cached)return cached;
    const result=hljs.highlightAuto(text.slice(0,16_384), ['json','java','python','javascript','typescript','yaml','xml','css','bash','swift','sql','cpp','csharp','go','ruby','php','rust','ini','markdown']);
    const value={html:result.value,language:result.relevance>=4?result.language:undefined};
    if(code.size>=160)code.delete(code.keys().next().value!);code.set(text,value);return value;
}
export function Thumbnail({clip,revision}:{clip:Clip;revision:number}) {
    const key=`${clip.id}:${clip.lastCopiedAtMs}`;const [src,setSrc]=useState(pictures.get(key));const [failed,setFailed]=useState(false);
    useEffect(()=>{let live=true;const abort=new AbortController();setFailed(false);const cached=pictures.get(key);if(cached){setSrc(cached);return;}setSrc(undefined);
      fetch(thumb(clip.id,revision),{signal:abort.signal}).then(r=>{if(!r.ok)throw new Error('unavailable');return r.blob();}).then(b=>{if(!live)return;const url=URL.createObjectURL(b);if(pictures.size>=80){const first=pictures.keys().next().value!;URL.revokeObjectURL(pictures.get(first)!);pictures.delete(first);}pictures.set(key,url);setSrc(url);}).catch(()=>{if(live)setFailed(true);});
      return()=>{live=false;abort.abort();};
    },[key,clip.id,revision]);
    return src?<img className="thumbnail" src={src} alt="图片预览"/>:<div className="image-placeholder">{failed?'预览不可用':'图片'}<span>{failed?'原始内容仍保留':'正在生成预览'}</span></div>;
}
export function clearPreviewCache(){code.clear();for(const value of pictures.values())URL.revokeObjectURL(value);pictures.clear();}
export function RichPreview({html}:{html:string}){
    const safe=`<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; form-action 'none'; base-uri 'none'"><style>body{font:14px/1.5 'Segoe UI',sans-serif;overflow-wrap:anywhere;color:#20262e}img{max-width:100%}</style></head><body>${html}</body></html>`;
    return <iframe title="富文本预览" sandbox="" referrerPolicy="no-referrer" srcDoc={safe}/>;
}
