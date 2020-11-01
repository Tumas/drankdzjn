use log::info;
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

        let img_url = format!("https://res.cloudinary.com/boozeboodcdn/image/upload/{}", banner.imgsrc);
        let simple_attempt_url = img_url.replace("-de-", "-de-dd-");
        let success = ping(&client, &simple_attempt_url).await?;

        if stop_when_found && success {
            return Ok(())
        }
    }

    Ok(())
}