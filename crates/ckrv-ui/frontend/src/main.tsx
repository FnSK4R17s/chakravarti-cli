import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './lib/api' // Tauri fetch interceptor - must be imported before any fetch calls
import './index.css'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
