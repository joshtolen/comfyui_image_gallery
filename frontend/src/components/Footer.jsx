import React, { useContext } from 'react';
import { ThemeContext } from '../App';

const Footer = () => {
  const { theme } = useContext(ThemeContext);
  
  return (
    <footer className={`mt-auto py-6 text-center border-t ${
      theme === 'dark' ? 'border-white/10' : 'border-black/10'
    }`}>
      <p>All images are copyrighted © 2023-2025</p>
      <p className="text-xs opacity-70 mt-2">ComfyUI Gallery v3.0</p>
    </footer>
  );
};

export default Footer;