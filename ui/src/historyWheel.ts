// 横向历史列表的滚轮与闲置滚动条行为。by AI.Coding
import { RefObject, useEffect } from 'react';

export function useHistoryWheel(ref:RefObject<HTMLDivElement|null>) {
 useEffect(()=>{
  const list=ref.current;if(!list)return;
  let timer:ReturnType<typeof setTimeout>|undefined;
  const active=()=>{list.classList.add('scrolling');clearTimeout(timer);timer=setTimeout(()=>list.classList.remove('scrolling'),800);};
  const wheel=(event:WheelEvent)=>{
   // 触控板的横向分量保留原生行为；普通竖向滚轮映射到横轴。
   if(event.ctrlKey||event.metaKey||event.deltaX!==0||event.deltaY===0)return;
   const unit=event.deltaMode===1?32:event.deltaMode===2?list.clientWidth:1;
   const next=Math.max(0,Math.min(list.scrollWidth-list.clientWidth,list.scrollLeft+event.deltaY*unit));
   if(next!==list.scrollLeft){event.preventDefault();list.scrollLeft=next;active();}
  };
  const hover=(event:PointerEvent)=>list.classList.toggle('scrollbar-hover',event.clientY>=list.getBoundingClientRect().bottom-14);
  const leave=()=>list.classList.remove('scrollbar-hover');
  list.addEventListener('wheel',wheel,{passive:false});list.addEventListener('scroll',active);
  list.addEventListener('pointermove',hover);list.addEventListener('pointerleave',leave);
  return()=>{clearTimeout(timer);list.removeEventListener('wheel',wheel);list.removeEventListener('scroll',active);list.removeEventListener('pointermove',hover);list.removeEventListener('pointerleave',leave);};
 },[ref]);
}
