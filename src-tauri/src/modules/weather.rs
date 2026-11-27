//! Météo : prévisions d'Open-Meteo (gratuit, sans clé d'API ni compte).
//!
//! Deux requêtes possibles :
//! - la recherche d'une ville par son nom (service de géocodage d'Open-Meteo) ;
//! - la météo actuelle et les prévisions des prochains jours pour une position.
//!
//! La tuile demande la météo toutes les 30 minutes, très loin de la limite
//! d'usage gratuit (10 000 requêtes par jour).

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::http::{client, network_error};

const FORECAST_API: &str = "https://api.open-meteo.com/v1/forecast";
const GEOCODING_API: &str = "https://geocoding-api.open-meteo.com/v1/search";

/// Nombre de jours de prévisions renvoyés (aujourd'hui compris).
const FORECAST_DAYS: u32 = 5;

/// Nombre de villes proposées par une recherche.
const SEARCH_RESULTS: u32 = 6;

/// Une ville trouvée par la recherche.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    /// Région et pays, pour distinguer deux villes du même nom.
    #[serde(default)]
    pub detail: String,
}

/// Météo du moment.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Current {
    /// Température, en °C.
    pub temperature: f64,
    /// Température ressentie, en °C.
    pub feels_like: f64,
    /// Code météo de l'OMM (0 = ciel dégagé, 61 = pluie…).
    pub code: u32,
    pub is_day: bool,
    /// Vent, en km/h.
    pub wind: f64,
}

/// Prévision d'une journée.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    /// Date, au format `AAAA-MM-JJ`.
    pub date: String,
    pub code: u32,
    pub min: f64,
    pub max: f64,
}

/// Ce que la tuile reçoit.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Forecast {
    pub current: Current,
    /// Aujourd'hui, puis les jours suivants.
    pub days: Vec<Day>,
}

#[derive(Deserialize)]
struct ForecastResponse {
    current: CurrentResponse,
    daily: DailyResponse,
}

#[derive(Deserialize)]
struct CurrentResponse {
    temperature_2m: f64,
    apparent_temperature: f64,
    weather_code: u32,
    is_day: u8,
    wind_speed_10m: f64,
}

#[derive(Deserialize)]
struct DailyResponse {
    time: Vec<String>,
    weather_code: Vec<u32>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    name: String,
    latitude: f64,
    longitude: f64,
    #[serde(default)]
    admin1: Option<String>,
    #[serde(default)]
    country: Option<String>,
}

/// Erreur renvoyée par Open-Meteo : `{"error": true, "reason": "…"}`.
#[derive(Deserialize)]
struct ApiError {
    reason: String,
}

fn check_position(latitude: f64, longitude: f64) -> AppResult<()> {
    if (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude) {
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            "position de la ville incorrecte".into(),
        ))
    }
}

fn to_forecast(response: ForecastResponse) -> Forecast {
    let c = response.current;
    let d = response.daily;
    let days = d
        .time
        .into_iter()
        .zip(d.weather_code)
        .zip(d.temperature_2m_min.into_iter().zip(d.temperature_2m_max))
        .map(|((date, code), (min, max))| Day {
            date,
            code,
            min,
            max,
        })
        .collect();
    Forecast {
        current: Current {
            temperature: c.temperature_2m,
            feels_like: c.apparent_temperature,
            code: c.weather_code,
            is_day: c.is_day == 1,
            wind: c.wind_speed_10m,
        },
        days,
    }
}

fn to_place(result: SearchResult) -> Place {
    let detail = [result.admin1, result.country]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    Place {
        name: result.name,
        latitude: result.latitude,
        longitude: result.longitude,
        detail,
    }
}

async fn get_json<T: serde::de::DeserializeOwned>(
    url: &str,
    query: &[(&str, String)],
) -> AppResult<T> {
    let response = client()
        .get(url)
        .query(query)
        .send()
        .await
        .map_err(|e| network_error("Le service météo", e))?;
    let status = response.status();
    if !status.is_success() {
        let reason = response
            .json::<ApiError>()
            .await
            .map(|e| e.reason)
            .unwrap_or_else(|_| status.to_string());
        return Err(AppError::Other(format!(
            "le service météo a refusé la demande : {reason}"
        )));
    }
    response
        .json()
        .await
        .map_err(|e| network_error("Le service météo", e))
}

/// Météo actuelle et prévisions des prochains jours pour une position.
#[tauri::command]
pub async fn weather_forecast(latitude: f64, longitude: f64) -> AppResult<Forecast> {
    check_position(latitude, longitude)?;
    let query = [
        ("latitude", latitude.to_string()),
        ("longitude", longitude.to_string()),
        (
            "current",
            "temperature_2m,apparent_temperature,weather_code,is_day,wind_speed_10m".into(),
        ),
        (
            "daily",
            "weather_code,temperature_2m_max,temperature_2m_min".into(),
        ),
        ("timezone", "auto".into()),
        ("forecast_days", FORECAST_DAYS.to_string()),
    ];
    let response: ForecastResponse = get_json(FORECAST_API, &query).await?;
    Ok(to_forecast(response))
}

/// Cherche une ville par son nom.
#[tauri::command]
pub async fn weather_search(name: String) -> AppResult<Vec<Place>> {
    let name = name.trim();
    if name.chars().count() < 2 {
        return Ok(Vec::new());
    }
    let query = [
        ("name", name.to_string()),
        ("count", SEARCH_RESULTS.to_string()),
        ("language", "fr".into()),
        ("format", "json".into()),
    ];
    let response: SearchResponse = get_json(GEOCODING_API, &query).await?;
    Ok(response.results.into_iter().map(to_place).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_forecast() {
        let response: ForecastResponse = serde_json::from_str(
            r#"{"latitude":49.18,"longitude":-0.37,
                "current":{"time":"2026-11-27T14:00","temperature_2m":9.4,"apparent_temperature":6.8,
                           "weather_code":61,"is_day":1,"wind_speed_10m":21.3},
                "daily":{"time":["2026-11-27","2026-11-28"],"weather_code":[61,3],
                         "temperature_2m_max":[10.2,11.0],"temperature_2m_min":[5.1,4.3]}}"#,
        )
        .unwrap();
        let forecast = to_forecast(response);
        assert_eq!(forecast.current.temperature, 9.4);
        assert_eq!(forecast.current.code, 61);
        assert!(forecast.current.is_day);
        assert_eq!(
            forecast.days[1],
            Day {
                date: "2026-11-28".into(),
                code: 3,
                min: 4.3,
                max: 11.0
            }
        );
    }

    #[test]
    fn reads_search_results_with_their_region() {
        let response: SearchResponse = serde_json::from_str(
            r#"{"results":[{"id":3029241,"name":"Caen","latitude":49.18585,"longitude":-0.35912,
                "country":"France","admin1":"Normandie"},
               {"id":1,"name":"Caen","latitude":-30.0,"longitude":150.0}]}"#,
        )
        .unwrap();
        let places: Vec<Place> = response.results.into_iter().map(to_place).collect();
        assert_eq!(places[0].detail, "Normandie, France");
        assert_eq!(places[1].detail, "");
    }

    #[test]
    fn a_search_without_result_is_empty() {
        let response: SearchResponse =
            serde_json::from_str(r#"{"generationtime_ms":0.5}"#).unwrap();
        assert!(response.results.is_empty());
    }

    #[test]
    fn rejects_an_impossible_position() {
        assert!(check_position(49.18, -0.37).is_ok());
        assert!(check_position(120.0, 0.0).is_err());
        assert!(check_position(0.0, -200.0).is_err());
    }
}
