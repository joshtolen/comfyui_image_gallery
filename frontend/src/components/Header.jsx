import React, { useContext } from 'react';
import { ThemeContext } from '../App';
import { MoonIcon, SunIcon, TrashIcon } from '@heroicons/react/24/outline';

const Header = () => {
  const { theme, toggleTheme } = useContext(ThemeContext);

  return (
    <header className={`p-4 border-b ${
      theme === 'dark' ? 'border-white/10' : 'border-black/10'
    }`}
    style={{ backgroundColor: theme === 'dark' ? '#1a1a1a' : '#f5f5f5' }}>
      <div className="flex justify-between items-center flex-wrap max-w-7xl mx-auto gap-4">
        <div className="flex-1 text-center">
          <img 
            className="max-w-[300px] h-auto mx-auto transition-transform duration-300 hover:scale-105" 
            src="/api/static/logo.png" 
            alt="ComfyUI Logo" 
            width="400" 
            height="100"
          />
        </div>
        <div className="flex gap-4 items-center">
          <button 
            onClick={toggleTheme}
            className="w-10 h-10 rounded-full flex items-center justify-center text-xl border transition-transform hover:rotate-12 hover:scale-110"
            style={{ 
              borderColor: theme === 'dark' ? '#00cec9' : '#00b894',
              color: theme === 'dark' ? '#00cec9' : '#00b894'
            }}
            aria-label="Toggle light/dark theme"
          >
            {theme === 'dark' ? <SunIcon className="w-5 h-5" /> : <MoonIcon className="w-5 h-5" />}
          </button>
          
          <button 
            id="delete-selected" 
            className="bg-red-600 text-white rounded-md shadow hover:bg-red-700 hover:-translate-y-0.5 hover:shadow-md 
                    transition-all text-sm px-4 py-2 font-semibold flex items-center gap-2"
          >
            <TrashIcon className="w-4 h-4" />
            <span>Delete Selected</span>
          </button>
        </div>
      </div>
    </header>
  );
};

export default Header;