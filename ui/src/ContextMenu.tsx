// 卡片右键菜单：按鼠标位置贴边，键盘与滚动均保持可操作。by AI.Coding
import { ReactNode, useLayoutEffect, useRef, useState } from 'react';
export interface MenuAnchor {x:number;y:number}
/** 测量真实菜单后约束在视口内，低高度面板允许内部滚动。 */
export function ContextMenu({anchor,children,close,wide=false}:{anchor:MenuAnchor;children:ReactNode;close:()=>void;wide?:boolean}) {
 const ref=useRef<HTMLDivElement>(null);
 const [position,setPosition]=useState({left:8,top:8});
 useLayoutEffect(()=>{
  const menu=ref.current;if(!menu)return;
  const place=()=>{const r=menu.getBoundingClientRect();setPosition({left:Math.max(8,Math.min(anchor.x,window.innerWidth-r.width-8)),top:Math.max(8,Math.min(anchor.y,window.innerHeight-r.height-8))});};
  place();const observer=new ResizeObserver(place);observer.observe(menu);
  menu.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus();
  window.addEventListener('resize',place);return()=>{observer.disconnect();window.removeEventListener('resize',place);};
 },[anchor,wide]);
 return <><button className="scrim" aria-label="关闭操作菜单" onClick={close} onContextMenu={e=>{e.preventDefault();close();}}/><div ref={ref} className={`context-menu ${wide?'transforms':''}`} style={position} role="menu" onKeyDown={e=>{
  // 菜单内导航不传到历史卡片，防止右键目标随方向键变化。
  if(['ArrowDown','ArrowUp','Home','End'].includes(e.key)){e.preventDefault();e.stopPropagation();const buttons=[...ref.current!.querySelectorAll<HTMLButtonElement>('button:not(:disabled)')].filter(b=>b.getClientRects().length);const index=buttons.indexOf(document.activeElement as HTMLButtonElement);const next=e.key==='Home'?0:e.key==='End'?buttons.length-1:(index+(e.key==='ArrowDown'?1:-1)+buttons.length)%buttons.length;buttons[next]?.focus();}
 }}>{children}</div></>;
}
