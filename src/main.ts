import "./styles/global.css";
import App from "./App.svelte";
import "@stardazed/streams-polyfill";

const app = new App({
  target: document.getElementById("app"),
});

export default app;
