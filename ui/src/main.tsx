import { createRoot } from 'react-dom/client';
import App, { FeedbackWindow } from './App';
import './style.css';
const feedback=location.hash==='#feedback';
if(feedback)document.body.classList.add('feedback-window');
createRoot(document.getElementById('root')!).render(feedback?<FeedbackWindow/>:<App/>);
