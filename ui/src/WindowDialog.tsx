// 详情和编辑复用同一原生窗口表面；移动与缩放交给系统。by AI.Coding
import {useEffect, useRef, type ReactNode, type RefObject, type KeyboardEvent, type PointerEvent} from 'react';
import {call, message} from './api';
const directions=['North','NorthEast','East','SouthEast','South','SouthWest','West','NorthWest'] as const;
/** 只允许主键从标题空白拖动，边缘缩放不干扰正文选择或按钮。 */
export function WindowDialog({className,label,title,closeLabel,close,busy=false,actions,children,dialogRef,onKeyDown,onError}:{className:string;label:string;title:ReactNode;closeLabel:string;close:()=>void;busy?:boolean;actions?:ReactNode;children:ReactNode;dialogRef?:RefObject<HTMLElement|null>;onKeyDown?:(event:KeyboardEvent<HTMLElement>)=>void;onError:(error:string)=>void}){
 const local=useRef<HTMLElement>(null);const ref=dialogRef??local;
 // 隐藏历史仍保留布局和选择，但键盘焦点只能留在当前弹窗内。
 useEffect(()=>{const previous=document.activeElement as HTMLElement|null;const surface=ref.current;function focus(e:FocusEvent){if(surface&&!surface.contains(e.target as Node))surface.focus();}document.addEventListener('focusin',focus);surface?.focus();return()=>{document.removeEventListener('focusin',focus);if(previous?.isConnected)previous.focus();};},[ref]);
 /** 系统拖动只使用当前窗口，失败时沿用调用方的错误反馈。 */
 function pointer(e:PointerEvent<HTMLElement>,direction?:string){if(!e.isPrimary||e.button!==0||(!direction&&(e.target as Element).closest('button,input,a,select')))return;e.preventDefault();void call(direction?'start_window_resize':'start_window_drag',direction?{direction}:{}).catch(reason=>onError(message(reason)));}
 /** Tab 环绕与 Escape 关闭共享；编辑器继续处理保存快捷键与 IME。 */
 function key(e:KeyboardEvent<HTMLElement>){onKeyDown?.(e);if(e.defaultPrevented)return;e.stopPropagation();if(e.nativeEvent.isComposing)return;if(e.key==='Escape'){e.preventDefault();if(!busy)close();}else if(e.key==='Tab'){const controls=Array.from(ref.current?.querySelectorAll<HTMLElement>('textarea:not(:disabled),button:not(:disabled),input:not(:disabled),select:not(:disabled),a[href],iframe')??[]).filter(el=>el.getClientRects().length>0);const first=controls[0],last=controls.at(-1);if(!first){e.preventDefault();ref.current?.focus();}else if(e.shiftKey&&(document.activeElement===first||document.activeElement===ref.current)){e.preventDefault();last?.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first.focus();}}}
 return <section ref={ref} tabIndex={-1} className={`native-dialog dialog-layer ${className}`} role="dialog" aria-modal="true" aria-label={label} onKeyDown={key}>
  <header className="window-titlebar editor-titlebar" title="拖动标题栏移动窗口" onPointerDown={e=>pointer(e)}><h2>{title}</h2>{actions}<button className="icon-button" disabled={busy} aria-label={closeLabel} onClick={close}>×</button></header>
  {children}
  {directions.map(direction=><div key={direction} aria-hidden="true" className="window-resize" data-direction={direction} onPointerDown={e=>pointer(e,direction)}/>)}
 </section>;
}
