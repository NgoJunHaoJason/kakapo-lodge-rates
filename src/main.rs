use chrono::{DateTime, Utc};
use std::time::SystemTime;
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use tide::Request;
use tide::{prelude::*, Response};

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/hello").get(hello);
    app.at("/rates").get(rates);

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("https://kakapolodge.github.io/"))
        .allow_origin(Origin::from("https://www.kakapolodge.co.nz/"))
        .allow_origin(Origin::from("https://kakapolodge.co.nz/"))
        .allow_credentials(false);
    app.with(cors);

    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(default)]
struct Query {
    name: String,
}
impl Default for Query {
    fn default() -> Self {
        Self {
            name: "world".to_owned(),
        }
    }
}

async fn hello(request: Request<()>) -> tide::Result {
    let query: Query = request.query()?;
    let reply = format!("Hello, {}!", query.name);
    Ok(reply.into())
}

#[derive(Deserialize, Serialize)]
struct LittleHotelierRates {
    name: String,
    rate_plans: Vec<RatePlan>,
}

#[derive(Deserialize, Serialize)]
struct RatePlan {
    id: u32,
    name: String,
    rate_plan_dates: Vec<RatePlanDate>,
}

#[derive(Deserialize, Serialize)]
struct RatePlanDate {
    id: Option<u32>,
    date: String,
    rate: u16,
    min_stay: u8,
    stop_online_sell: bool,
    close_to_arrival: bool,
    close_to_departure: bool,
    max_stay: Option<u8>,
    available: u8,
}

#[derive(Deserialize, Serialize)]
struct LodgeRates {
    rates: Vec<LodgeRate>,
}

#[derive(Deserialize, Serialize)]
struct LodgeRate {
    accommodation_type: String,
    rate: u16,
    num_available: u8,
}

const LITTLE_HOTELIER_BASE_URL: &str =
    "https://apac.littlehotelier.com/api/v1/properties/kakapolodgedirect/rates.json";

async fn rates(_request: Request<()>) -> tide::Result {
    let now: DateTime<Utc> = SystemTime::now().into();

    println!("got here");

    let todays_date = now
        .to_rfc3339()
        .split('T')
        .map(|string_slice| string_slice.to_owned())
        .collect::<Vec<_>>()
        .first()
        .unwrap_or(&String::from(""))
        .to_owned();

    println!("today's date: {}", todays_date);

    let url = format!(
        "{}?start_date={}&end_date={}",
        LITTLE_HOTELIER_BASE_URL, todays_date, todays_date
    );

    println!("url: {}", url);

    let little_hotelier_response: Vec<LittleHotelierRates> = surf::get(url).recv_json().await?;

    let little_hotelier_rates = little_hotelier_response.first().unwrap();

    println!("got response from LH");

    let lodge_rates = map_rates(little_hotelier_rates);
    let response_body = json!(lodge_rates);

    let response = Response::builder(200).body(response_body).build();
    Ok(response)
}

fn map_rates(little_hotelier_rates: &LittleHotelierRates) -> LodgeRates {
    let rate_plans = &little_hotelier_rates.rate_plans;

    let rates = rate_plans
        .into_iter()
        .map(|rate_plan| map_rate_plan_to_lodge_rate(rate_plan))
        .collect();

    LodgeRates { rates }
}

fn map_rate_plan_to_lodge_rate(rate_plan: &RatePlan) -> LodgeRate {
    let rate_plan_date = rate_plan.rate_plan_dates.first().unwrap();

    LodgeRate {
        accommodation_type: rate_plan.name.to_owned(),
        rate: rate_plan_date.rate,
        num_available: rate_plan_date.available,
    }
}
