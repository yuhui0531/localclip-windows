// 轻量本地线条图标，统一工具栏和设置的视觉尺度；不依赖网络字体。by AI.Coding
import logo from './assets/localclip.png';
const paths={
 search:'m14.5 14.5 5 5 M16 9.5a6.5 6.5 0 1 1-13 0 6.5 6.5 0 0 1 13 0',
 star:'m12 3 2.8 5.7 6.2.9-4.5 4.4 1.1 6.2-5.6-3-5.6 3 1.1-6.2L3 9.6l6.2-.9Z',
 filter:'M7 8h10 M9 12h6 M11 16h2',
 more:'M7 12h.01 M12 12h.01 M17 12h.01',
 close:'m8 8 8 8 M16 8l-8 8',
 pin:'m9 5 6 0-1 5 3 3v1H7v-1l3-3Z M12 14v6',
 info:'M12 11v6 M12 7h.01',
 edit:'m15 5 4 4 M5 15l-1 5 5-1L20 8a2.8 2.8 0 0 0-4-4Z',
 copy:'M9 8h10v12H9Z M15 8V4H4v12h5',
 paste:'M9 4h6v4H9Z M7 6H5v15h14V6h-2 M9 13h6 M9 17h4',
 folder:'M3 7V5h7l2 3h9v12H3Z',
 text:'M5 5h14 M12 5v15 M8 20h8',
 link:'m10 14 4-4 M9 15l-1 1a4 4 0 0 1-6-6l4-4a4 4 0 0 1 6 0 M15 9l1-1a4 4 0 0 1 6 6l-4 4a4 4 0 0 1-6 0',
 image:'M3 4h18v16H3Z M3 16l5-5 5 5 3-3 5 5 M16 8h.01',
 file:'M6 3h8l4 4v14H6Z M14 3v5h4 M9 12h6 M9 16h6',
 mixed:'M9 8h12v13H9Z M15 8V3H3v13h6',
 settings:'M5 6h14 M5 12h14 M5 18h14 M9 4v4 M15 10v4 M10 16v4',
 privacy:'M5 10V7l7-4 7 4v5c0 4-4 7-7 9-3-2-7-5-7-9 M9 12l2 2 4-4',
 storage:'M5 4h14l3 12v4H2v-4Z M2 16h20 M16 18h.01 M19 18h.01',
 appearance:'M12 3a9 9 0 1 0 0 18Z',
};
export type IconName=keyof typeof paths;
/** 所有装饰图标由按钮的文字或 aria-label 负责朗读。 */
export function Icon({name,circle=false}:{name:IconName;circle?:boolean}){return <svg className="ui-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">{circle&&<circle cx="12" cy="12" r="10"/>}<path d={paths[name]} transform={circle&&name==='pin'?'translate(3 3) scale(.75)':undefined}/></svg>;}
/** 品牌图复用 macOS 工程资源，保留透明背景和原始比例。 */
export function BrandMark({large=false}:{large?:boolean}){return <img className={`brand-mark${large?' large':''}`} src={logo} alt="" draggable={false}/>;}
