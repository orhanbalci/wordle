use anyhow::Result;

pub fn meaning(word: &str) -> Result<String> {
    let body = reqwest::blocking::get(format!("https://sozluk.gov.tr/gts?ara={}", word))?.text()?;
    let v: serde_json::Value = serde_json::from_str(&body)?;
    let result = v
        .get(0)
        .and_then(|first_result| first_result.get("anlamlarListe"))
        .and_then(|meanings| meanings.get(0))
        .and_then(|meaning| meaning.get("anlam"))
        .map_or(String::new(), |result| result.to_string());
    Ok(result)
}
