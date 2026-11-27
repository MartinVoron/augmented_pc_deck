// Météo : prévisions d'Open-Meteo (voir src-tauri/src/modules/weather.rs).
import { call } from "../../tauri";

/** Une ville, avec sa position. */
export interface Place {
  name: string;
  latitude: number;
  longitude: number;
  /** Région et pays. */
  detail: string;
}

export interface Forecast {
  current: {
    temperature: number;
    feelsLike: number;
    /** Code météo de l'OMM. */
    code: number;
    isDay: boolean;
    /** Vent, en km/h. */
    wind: number;
  };
  /** Aujourd'hui, puis les jours suivants. */
  days: { date: string; code: number; min: number; max: number }[];
}

/** Ville d'une nouvelle tuile Météo. */
export const DEFAULT_PLACE: Place = {
  name: "Caen",
  latitude: 49.18585,
  longitude: -0.35912,
  detail: "Normandie, France",
};

/** Intervalle de rafraîchissement : la météo change lentement. */
export const REFRESH_MS = 30 * 60_000;

export function getForecast(place: Place): Promise<Forecast> {
  return call<Forecast>("weather_forecast", {
    latitude: place.latitude,
    longitude: place.longitude,
  });
}

export function searchPlaces(name: string): Promise<Place[]> {
  return call<Place[]>("weather_search", { name });
}

/** Famille d'icône d'un code météo. */
export type WeatherIcon =
  "clear" | "partly" | "cloudy" | "fog" | "drizzle" | "rain" | "snow" | "storm";

/** Description en français et icône d'un code météo de l'OMM. */
export function describe(code: number): { label: string; icon: WeatherIcon } {
  if (code === 0) return { label: "Ciel dégagé", icon: "clear" };
  if (code === 1) return { label: "Plutôt dégagé", icon: "clear" };
  if (code === 2) return { label: "Éclaircies", icon: "partly" };
  if (code === 3) return { label: "Couvert", icon: "cloudy" };
  if (code === 45 || code === 48) return { label: "Brouillard", icon: "fog" };
  if (code >= 51 && code <= 55) return { label: "Bruine", icon: "drizzle" };
  if (code === 56 || code === 57) return { label: "Bruine verglaçante", icon: "drizzle" };
  if (code === 61) return { label: "Pluie faible", icon: "rain" };
  if (code === 63) return { label: "Pluie", icon: "rain" };
  if (code === 65) return { label: "Forte pluie", icon: "rain" };
  if (code === 66 || code === 67) return { label: "Pluie verglaçante", icon: "rain" };
  if (code >= 71 && code <= 77) return { label: "Neige", icon: "snow" };
  if (code >= 80 && code <= 82) return { label: "Averses", icon: "rain" };
  if (code === 85 || code === 86) return { label: "Averses de neige", icon: "snow" };
  if (code === 95) return { label: "Orage", icon: "storm" };
  if (code === 96 || code === 99) return { label: "Orage et grêle", icon: "storm" };
  return { label: "Temps inconnu", icon: "cloudy" };
}

/** Température arrondie : « 9° ». Évite « -0° ». */
export function formatTemperature(value: number): string {
  const rounded = Math.round(value);
  return `${rounded === 0 ? 0 : rounded}°`;
}

/** Nom court du jour d'une date `AAAA-MM-JJ` : « Auj. », puis « sam. », « dim. »… */
export function dayLabel(date: string, index: number): string {
  if (index === 0) return "Auj.";
  const [year, month, day] = date.split("-").map(Number);
  return new Date(year, month - 1, day).toLocaleDateString("fr-FR", { weekday: "short" });
}
