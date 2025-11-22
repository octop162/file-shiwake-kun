/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'selector',
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#646cff',
          hover: '#535bf2',
        },
        success: '#4caf50',
        warning: '#ff9800',
        error: '#ff4444',
        dark: {
          bg: '#242424',
          surface: '#1a1a1a',
          elevated: '#2a2a2a',
          border: '#333',
        },
        light: {
          bg: '#ffffff',
          surface: '#f5f5f5',
          elevated: '#ffffff',
          border: '#e0e0e0',
        }
      },
      animation: {
        'float': 'float 3s ease-in-out infinite',
        'spin': 'spin 1s linear infinite',
        'fadeIn': 'fadeIn 0.2s ease-in-out',
        'slideUp': 'slideUp 0.3s ease-out',
        'slideDown': 'slideDown 0.2s ease-out',
      },
      keyframes: {
        float: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        fadeIn: {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
        slideUp: {
          from: { transform: 'translateY(20px)', opacity: '0' },
          to: { transform: 'translateY(0)', opacity: '1' },
        },
        slideDown: {
          from: { opacity: '0', transform: 'translateY(-10px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
      },
    },
  },
  plugins: [],
}
