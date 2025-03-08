import React, { useContext } from 'react';
import { ThemeContext } from '../App';

const Loading = ({ message = 'Loading gallery...' }) => {
  const { theme } = useContext(ThemeContext);
  
  return (
    <div className="flex flex-col items-center justify-center h-[300px] w-full">
      <div className={`w-12 h-12 rounded-full border-4 border-t-transparent animate-spin mb-4 ${
        theme === 'dark' ? 'border-primary-dark' : 'border-primary-light'
      }`}></div>
      <p className="text-center">{message}</p>
    </div>
  );
};

export default Loading;