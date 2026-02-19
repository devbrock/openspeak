import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import { OverlayApp } from './OverlayApp';
import './styles.css';

const isOverlayWindow = new URLSearchParams(window.location.search).get('overlay') === '1';
document.documentElement.classList.toggle('overlay-window', isOverlayWindow);
document.body.classList.toggle('overlay-window', isOverlayWindow);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    {isOverlayWindow ? <OverlayApp /> : <App />}
  </React.StrictMode>
);
