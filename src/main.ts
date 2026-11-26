import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";
import { logUncaughtErrors } from "./lib/logging";

logUncaughtErrors();

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
