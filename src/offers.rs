use log::{debug, info};
use math::round;
use prettytable::{Table, Row, Cell};
use serde::Deserialize;

use crate::APP_USER_AGENT;

const BASE_URL: &str = "https://es-api.drankdozijn.nl/products?country=DE&language=nl&page_template=groep&group=whisky";

#[derive(Debug, Deserialize)]
struct WhiskeyFeatureValue {
    #[serde(default)]
    alias: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct WhiskeyFeature {
    #[serde(default)]
    alias: String,
    #[serde(default)]
    description: String,
    value: WhiskeyFeatureValue
}

#[derive(Debug, Deserialize)]
struct Whiskey {
    availability: String,
    alias: String,
    description: String,
    price: f32,
    #[serde(default, rename(deserialize = "salePrice"))]
    sale_price: f32,
    features: Vec<WhiskeyFeature>
}

impl Whiskey {
    fn feature(&self, alias: &str) -> Option<String> {
        debug!("{:?}", self.features.iter().map(|f| f.alias.clone()).collect::<Vec<String>>());

        self.features
            .iter()
            .find(|feature| feature.alias == alias)
            .map(|wf| wf.value.description.clone())
    }

    fn country(&self) -> Option<String> {
        self.feature("land")
    }
 
    fn categorie(&self) -> Option<String> {
        self.feature("categorie")
    }

    fn percentage(&self) -> Option<String> {
        self.features
            .iter()
            .find(|feature| feature.description == "Alcoholpercentage")
            .map(|wf| wf.value.description.clone())
    }

    fn discount(&self) -> f64 {
        if self.sale_price > 0.0 {
            return round::ceil((self.price - self.sale_price).into(), 2);
        }

        0.0
    }

    fn price(&self) -> f32 {
        if self.sale_price > 0.0 {
            return self.sale_price;
        }

        self.price
    } 

    fn url(&self) -> String {
        format!("https://drankdozijn.de/artikel/{}", self.alias)
    }
}

pub async fn list(price: u16, discount: u16) -> Result<(), reqwest::Error> {
    info!(
        "Finding offers for {} price with discount {}",
        price, discount
    );

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("HTTP client should be buildable");

    let resp = client.get(BASE_URL).send().await?;
    let whiskeys = resp.json::<Vec<Whiskey>>().await?;

    info!("Found {} whiskeys", whiskeys.len());

    let mut candidates = whiskeys
        .into_iter()
        .filter(|whiskey| {
            whiskey.discount() >= discount.into() && whiskey.price <= price.into()
        })
        .collect::<Vec<Whiskey>>();

    candidates.sort_by(|v1, v2| {
        v2.discount().partial_cmp(&v1.discount()).unwrap()
    });

    info!("Found {} candidate whiskeys for price {} EUR", candidates.len(), price);

    let mut table = Table::new();
    
    table.add_row(
        row![
            "TITLE", 
            "PRICE", 
            "OLD PRICE", 
            "DISCOUNT", 
            "AVAILABITLITY", 
            "CATEGORIE", 
            "PERCENTAGE", 
            "COUNTRY",
            "URL"
        ]
    );
    
    for whiskey in &candidates {
        debug!("{:?}", whiskey);

        table.add_row(
            Row::new(vec![
                Cell::new(&whiskey.description),
                Cell::new(&whiskey.price().to_string()),
                Cell::new(&whiskey.price.to_string()),
                Cell::new(&whiskey.discount().to_string()),
                Cell::new(&whiskey.availability),
                Cell::new(&whiskey.categorie().unwrap_or_default()),
                Cell::new(&whiskey.percentage().unwrap_or_default()),
                Cell::new(&whiskey.country().unwrap_or_default()),
                Cell::new(&whiskey.url()),
            ])
        );
    }

    table.printstd();

    Ok(())
}