/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./site/**/*.{js,hbs}",
  ],
  theme: {
    extend: {
        screens: {
            print: {raw: 'print'},
        }
    },
  },
  plugins: [],
}
