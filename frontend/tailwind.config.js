/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          dark: '#6c5ce7',
          light: '#5352ed'
        },
        secondary: {
          dark: '#a29bfe',
          light: '#7b68ee'
        },
        accent: {
          dark: '#00cec9',
          light: '#00b894'
        },
        background: {
          dark: '#1a1a1a',
          light: '#f5f5f5'
        },
        card: {
          dark: '#2d2d2d',
          light: '#ffffff'
        }
      },
      backgroundSize: {
        '200%': '200% 100%',
      },
      keyframes: {
        shimmer: {
          '0%': { backgroundPosition: '200% 0' },
          '100%': { backgroundPosition: '-200% 0' },
        },
      },
      animation: {
        shimmer: 'shimmer 1.5s infinite',
      },
    },
  },
  plugins: [],
}