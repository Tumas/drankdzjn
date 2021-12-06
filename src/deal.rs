use log::info;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

use crate::{offers, APP_USER_AGENT};

const BASE_URL: &str =
    "https://es-api.drankdozijn.nl/home?top_level_domain=de&country=DE&language=de";

#[derive(Debug, Deserialize)]
pub struct Banner {
    imgsrc: String,
}

#[derive(Debug, Deserialize)]
pub struct HomeResponse {
    #[serde(default, rename(deserialize = "homeGridBanners"))]
    banners: Vec<Banner>,
}

async fn fetch_home_reponse(client: &Client) -> Result<HomeResponse, reqwest::Error> {
    let resp = client.get(BASE_URL).send().await?;
    resp.json::<HomeResponse>().await
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

    let response = fetch_home_reponse(&client).await?;
    let sample_url = &response
        .banners
        .first()
        .expect("At least one banner must be present")
        .imgsrc;

    let re = Regex::new(r"(.+-de-).+(.jpg)").expect("regexp must be valid");
    let template = re.replace(&sample_url, "$1{}$2");

    info!("Using template {}", template);

    let found = find_by_banners(&client, &response, stop_when_found).await?;

    if stop_when_found && found {
        return Ok(());
    }

    find_by_iteration(&client, &template, stop_when_found).await?;

    Ok(())
}

pub async fn find_by_banners(
    client: &Client,
    response: &HomeResponse,
    stop_when_found: bool,
) -> Result<bool, reqwest::Error> {
    info!("Found {} home banners", response.banners.len());

    for banner in &response.banners {
        if !banner.imgsrc.contains("-de-") {
            info!("Skipping banner: {}", banner.imgsrc);
        }

        let success = check_url(client, &banner.imgsrc, stop_when_found).await?;
        if stop_when_found && success {
            return Ok(success);
        }
    }

    Ok(false)
}

pub async fn check_url(
    client: &Client,
    url: &str,
    stop_when_found: bool,
) -> Result<bool, reqwest::Error> {
    for candidate_url in &candidates(url) {
        let img_url = format!(
            "https://res.cloudinary.com/boozeboodcdn/image/upload/{}",
            candidate_url
        );
        let success = ping(client, &img_url).await?;

        if stop_when_found && success {
            return Ok(success);
        }
    }

    Ok(false)
}

pub async fn find_by_iteration(
    client: &Client,
    template: &str,
    stop_when_found: bool,
) -> Result<bool, reqwest::Error> {
    let whiskeys = offers::whiskeys().await?;
    let brands = whiskeys
        .iter()
        .filter_map(|whiskey| whiskey.brand())
        .map(|brand| brand.to_lowercase().replace(" ", "-"))
        .collect::<Vec<String>>();

    for brand in brands {
        let url = template.replace("{}", &brand);
        let success = check_url(client, &url, stop_when_found).await?;

        if stop_when_found && success {
            return Ok(success);
        }
    }

    Ok(false)
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
