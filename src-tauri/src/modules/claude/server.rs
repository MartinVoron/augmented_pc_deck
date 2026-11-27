//! Petit serveur HTTP local qui reçoit les événements des hooks de Claude Code.
//!
//! - Il n'écoute que sur `127.0.0.1` : aucun autre ordinateur ne peut l'atteindre.
//! - Chaque requête doit porter le jeton de l'application (`Authorization: Bearer …`) :
//!   un autre programme du PC ne peut pas envoyer de fausses notifications.
//! - Une seule adresse : `POST /claude/event`, avec le JSON que Claude Code donne au
//!   hook sur son entrée standard.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Port d'écoute, choisi pour ne pas gêner les ports courants.
pub const PORT: u16 = 47821;

/// Adresse appelée par le script du hook.
pub const EVENT_PATH: &str = "/claude/event";

/// Taille maximale d'une requête (le JSON d'un hook fait quelques centaines d'octets).
const MAX_REQUEST: usize = 64 * 1024;

/// Délai pour lire une requête : une connexion muette ne bloque rien.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Requête HTTP lue (seulement ce qui sert).
#[derive(Debug, PartialEq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub authorization: Option<String>,
    pub body: Vec<u8>,
}

/// Réponse renvoyée au script du hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Accepted,
    BadRequest,
    Unauthorized,
    NotFound,
    TooLarge,
}

impl Status {
    fn line(self) -> &'static str {
        match self {
            Self::Accepted => "204 No Content",
            Self::BadRequest => "400 Bad Request",
            Self::Unauthorized => "401 Unauthorized",
            Self::NotFound => "404 Not Found",
            Self::TooLarge => "413 Payload Too Large",
        }
    }
}

/// Lit les en-têtes et le corps d'une requête. `Err` donne la réponse à renvoyer.
pub fn parse(raw: &[u8]) -> Result<Option<Request>, Status> {
    let Some(end) = raw.windows(4).position(|w| w == b"\r\n\r\n") else {
        return if raw.len() >= MAX_REQUEST {
            Err(Status::TooLarge)
        } else {
            Ok(None) // En-têtes incomplets : il faut lire la suite.
        };
    };
    let head = std::str::from_utf8(&raw[..end]).map_err(|_| Status::BadRequest)?;
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    let (method, path) = match (first.next(), first.next()) {
        (Some(method), Some(path)) => (method.to_string(), path.to_string()),
        _ => return Err(Status::BadRequest),
    };
    let mut length = 0usize;
    let mut authorization = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("content-length") {
            length = value.parse().map_err(|_| Status::BadRequest)?;
        } else if name.eq_ignore_ascii_case("authorization") {
            authorization = Some(value.to_string());
        }
    }
    if length > MAX_REQUEST {
        return Err(Status::TooLarge);
    }
    let body_start = end + 4;
    if raw.len() < body_start + length {
        return Ok(None); // Corps incomplet.
    }
    Ok(Some(Request {
        method,
        path,
        authorization,
        body: raw[body_start..body_start + length].to_vec(),
    }))
}

/// Compare le jeton reçu au jeton attendu, en temps constant.
pub fn authorized(authorization: Option<&str>, token: &str) -> bool {
    let Some(received) = authorization.and_then(|a| a.strip_prefix("Bearer ")) else {
        return false;
    };
    let (a, b) = (received.trim().as_bytes(), token.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn read_request(stream: &mut TcpStream) -> Result<Request, Status> {
    let mut raw = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        let read = stream
            .read(&mut buffer)
            .await
            .map_err(|_| Status::BadRequest)?;
        if read == 0 {
            return Err(Status::BadRequest);
        }
        raw.extend_from_slice(&buffer[..read]);
        if raw.len() > MAX_REQUEST + 8192 {
            return Err(Status::TooLarge);
        }
        if let Some(request) = parse(&raw)? {
            return Ok(request);
        }
    }
}

/// Traite une connexion : lit la requête, la vérifie et la confie à `handle`.
async fn serve(mut stream: TcpStream, token: &str, handle: &(impl Fn(&[u8]) -> Status + Sync)) {
    let status = match tokio::time::timeout(READ_TIMEOUT, read_request(&mut stream)).await {
        Err(_) => Status::BadRequest,
        Ok(Err(status)) => status,
        Ok(Ok(request)) if request.path != EVENT_PATH => Status::NotFound,
        Ok(Ok(request)) if request.method != "POST" => Status::NotFound,
        Ok(Ok(request)) if !authorized(request.authorization.as_deref(), token) => {
            Status::Unauthorized
        }
        Ok(Ok(request)) => handle(&request.body),
    };
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        status.line()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
}

/// Ouvre le port d'écoute (seulement sur `127.0.0.1`).
pub async fn bind(port: u16) -> std::io::Result<TcpListener> {
    TcpListener::bind(("127.0.0.1", port)).await
}

/// Accepte les connexions une à une, sans fin. `token` donne le jeton attendu.
pub async fn run(
    listener: TcpListener,
    token: impl Fn() -> Option<String>,
    handle: impl Fn(&[u8]) -> Status + Sync,
) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        let Some(expected) = token() else {
            log::warn!("événement Claude ignoré : aucun jeton enregistré");
            continue;
        };
        serve(stream, &expected, &handle).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUEST: &[u8] = b"POST /claude/event HTTP/1.1\r\nHost: 127.0.0.1\r\n\
        Authorization: Bearer jeton\r\nContent-Type: application/json\r\n\
        Content-Length: 13\r\n\r\n{\"a\":\"\xc3\xa9t\xc3\xa9\"}";

    #[test]
    fn parses_a_complete_request() {
        let request = parse(REQUEST).unwrap().unwrap();
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, EVENT_PATH);
        assert_eq!(request.authorization.as_deref(), Some("Bearer jeton"));
        assert_eq!(request.body, "{\"a\":\"été\"}".as_bytes());
    }

    #[test]
    fn waits_for_the_rest_of_a_request() {
        assert_eq!(parse(&REQUEST[..20]), Ok(None));
        assert_eq!(parse(&REQUEST[..REQUEST.len() - 3]), Ok(None));
    }

    #[test]
    fn refuses_a_too_large_request() {
        let raw = format!(
            "POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_REQUEST + 1
        );
        assert_eq!(parse(raw.as_bytes()), Err(Status::TooLarge));
    }

    #[test]
    fn checks_the_token() {
        assert!(authorized(Some("Bearer jeton"), "jeton"));
        assert!(!authorized(Some("Bearer autre"), "jeton"));
        assert!(!authorized(Some("jeton"), "jeton"));
        assert!(!authorized(None, "jeton"));
    }

    async fn send(port: u16, raw: Vec<u8>) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        stream.write_all(&raw).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    }

    #[tokio::test]
    async fn serves_events_with_the_right_token_only() {
        let listener = bind(0).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let (sender, mut received) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(run(
            listener,
            || Some("jeton".to_string()),
            move |body: &[u8]| {
                sender.send(body.to_vec()).unwrap();
                Status::Accepted
            },
        ));

        assert!(send(port, REQUEST.to_vec())
            .await
            .starts_with("HTTP/1.1 204"));
        assert_eq!(received.recv().await.unwrap(), "{\"a\":\"été\"}".as_bytes());

        let wrong = String::from_utf8_lossy(REQUEST).replace("Bearer jeton", "Bearer faux!");
        assert!(send(port, wrong.into_bytes())
            .await
            .starts_with("HTTP/1.1 401"));

        let other = b"GET / HTTP/1.1\r\n\r\n".to_vec();
        assert!(send(port, other).await.starts_with("HTTP/1.1 404"));
        assert!(received.try_recv().is_err());
    }
}
