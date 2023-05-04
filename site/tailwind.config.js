/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./**/*.{js,html,vue}",
  ],
  theme: {
    extend: {
        screens: {
            print: {raw: 'print'},
        },
        fontFamily: {
            'special': ['Lato', 'sans-serif']
          },
    },
  },
  plugins: [],
}
