use log::info;
use regex::Regex;
use serde::Deserialize;

use crate::APP_USER_AGENT;

const BASE_URL: &str = "https://es-api.drankdozijn.nl/home?top_level_domain=de&country=DE&language=de";

#[derive(Debug, Deserialize)]
struct Banner {
    imgsrc: String,
}

#[derive(Debug, Deserialize)]
struct HomeResponse {
    #[serde(default, rename(deserialize = "homeGridBanners"))]
    banners: Vec<Banner>
}

async fn ping(client: &reqwest::Client, url: &str) -> Result<bool, reqwest::Error> {
    let response = client.head(url).send().await?;
    let status = response.status().is_success();

    if status {
        green_ln!("URL: {}, success: {}", url, status)
    } else {
        red_ln!("URL: {}, success: {}", url, status);
    }

    Ok(status)
}

fn candidates(base_url: &str) -> Vec<String> {
    let mut results = vec![];
    let add_dd = |url: &str| url.replace("-de-", "-de-dd-");

    // no change, de -> de-dd
    results.push(add_dd(base_url));

    // without numbers, de -> de-dd
    let mut parts: Vec<&str> = base_url.split("/").collect();

    if let Some(title) = parts.pop() {
        let re = Regex::new(r"[0-9]").expect("Number regexp must be valid");
        let title = re.replace_all(title, "").to_string();
        let result = add_dd(&format!("{}/{}", parts.join("/"), title));

        results.push(result);
    }

    results
}

pub async fn find(stop_when_found: bool) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("HTTP client should be buildable");

    let resp = client.get(BASE_URL).send().await?;
    let response = resp.json::<HomeResponse>().await?;

    info!("Found {} home banners", response.banners.len());

    for banner in &response.banners {
        if !banner.imgsrc.contains("-de-") {
            info!("Skipping banner: {}", banner.imgsrc);
        }

        for candidate_url in &candidates(&banner.imgsrc) {
            let img_url = format!("https://res.cloudinary.com/boozeboodcdn/image/upload/{}", candidate_url);
            let success = ping(&client, &img_url).await?;

            if stop_when_found && success {
                return Ok(())
            }
        }

    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::candidates;

    #[test]
    fn tests_candidates() {
        assert_eq!(
            candidates("homepage/drankdozijn/drankdozijn-enjoyislay20-de-lagavulin.jpg"),
            vec![
                "homepage/drankdozijn/drankdozijn-enjoyislay20-de-dd-lagavulin.jpg",
                "homepage/drankdozijn/drankdozijn-enjoyislay-de-dd-lagavulin.jpg"
            ]
        )
    }
}