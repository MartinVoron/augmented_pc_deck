import { call } from "../tauri";

/** Version de l'application, renvoyée par le Rust. */
export function getAppVersion(): Promise<string> {
  return call<string>("app_version");
}
