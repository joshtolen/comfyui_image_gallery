import React, { useState, useEffect } from 'react';
import { Routes, Route } from 'react-router-dom';

// Components
import Header from './components/Header';
import Footer from './components/Footer';

// Pages
import Gallery from './pages/Gallery';

// Theme context
export const ThemeContext = React.createContext({
  theme: 'dark',
  toggleTheme: () => {},
});

function App() {
  // Initialize theme from localStorage or default to dark
  const [theme, setTheme] = useState(() => {
    const savedTheme = localStorage.getItem('theme');
    return savedTheme || 'dark';
  });

  // Apply theme class to body when theme changes
  useEffect(() => {
    if (theme === 'dark') {
      document.body.classList.add('dark');
    } else {
      document.body.classList.remove('dark');
    }
    // Save to localStorage
    localStorage.setItem('theme', theme);
  }, [theme]);

  // Toggle theme function to be passed via context
  const toggleTheme = async () => {
    try {
      const response = await fetch('/api/toggle-theme', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (response.ok) {
        const data = await response.json();
        setTheme(data.theme);
      }
    } catch (error) {
      console.error('Failed to toggle theme:', error);
      // Fallback: Toggle theme locally even if API fails
      setTheme(prevTheme => prevTheme === 'dark' ? 'light' : 'dark');
    }
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      <div className="flex flex-col min-h-screen">
        <Header />
        <main className="flex-grow">
          <Routes>
            <Route path="/" element={<Gallery />} />
            <Route path="/type/:fileType" element={<Gallery />} />
          </Routes>
        </main>
        <Footer />
      </div>
    </ThemeContext.Provider>
  );
}

export default App;