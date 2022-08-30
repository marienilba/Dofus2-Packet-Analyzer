/** @type {import('tailwindcss').Config} */
module.exports = {
  plugins: [],
  theme: {
    extend: {},
  },
  purge: ["./index.html", "./src/**/*.{svelte,js,ts}"], // for unused CSS
  variants: {
    extend: {},
  },
  darkMode: "media", // or 'media' or 'class'
};
