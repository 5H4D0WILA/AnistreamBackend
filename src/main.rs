#[macro_use]
extern crate rocket;
extern crate serde;
extern crate serde_json;

// ROCKET -> HTTPServer code
use rocket::response::content::RawJson;
use rocket::response::status::NotFound;

// REQWEST -> HTTP Requests
use reqwest::ClientBuilder;
use reqwest::Response;

// SCRAPER -> HTML parsing and CSS Selection
use scraper::Html;
use scraper::Selector;

// SERDE -> JSON parsing
use serde::Deserialize;
use serde::Serialize;

// Timeout
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct AnimeData {
    name: String,
    synopsis: String,
    poster_image: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchData {
    id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    results: Vec<SearchData>,
}

#[get("/")]
fn info() -> String {
    "This is definitely a string with important info".to_string()
}

#[get("/")]
fn get_status() -> String {
    "{\"status\" : \"UP\"}".to_string()
}

// search for an Anime on Zoro.to
// usage /zoro/<name>  :   e.g.: name -> Jujutsu-Kaisen
#[get("/<name>")]
async fn search_zoro(name: String) -> Result<RawJson<String>, NotFound<String>> {
    // base search url
    let base_url = "https://zoro.to/search?keyword=".to_string();

    // asynchronically called HTTP Response from request_from_url
    let resp = request_from_url(format!("{}{}", base_url, name)).await;

    // Check if the HTTP Request was successfull
    if resp.status().is_success() {
        // Parse the HTML Response file
        let document = Html::parse_document(&resp.text().await.unwrap());

        // CSS Selector for the list of Anime from the Search
        let list_selector = Selector::parse("div.film_list-wrap").unwrap();

        // Empty Vector of type SearchData
        let mut vector: Vec<SearchData> = Vec::new();

        // Loop through the list of Anime from the Search
        // element is the HTML Code for the Anime Data
        for element in document.select(&list_selector) {
            // CSS Selector for the Hyperlink
            let selector = Selector::parse(
                "div.flw-item > div:nth-child(2) > h3:nth-child(1) > a:nth-child(1)",
            )
            .unwrap();

            // Loop through all Anime in the List
            for item in Html::parse_fragment(&element.inner_html()).select(&selector) {
                // Push the ID in the Hyperlink to the SearchData Vector
                // The ID is the href value without the starting "/" and the trailing "?ref=search"
                vector.push(SearchData {
                    id: item
                        .value()
                        .attr("href")
                        .unwrap()
                        .replace("?ref=search", "")
                        .replace("/", ""),
                });
            }
        }

        // SearchResult Struct with the SearchData Vector as its "results" value
        let search_result = SearchResult { results: vector };

        // Return Result with the SearchResult converted to a JSON object
        Ok(RawJson(format!(
            "{}",
            serde_json::to_string(&search_result).unwrap()
        )))
    }
    // If the Request wasnt successfull
    else {
        // Return a "JSON" that shows that something went wrong
        Ok(RawJson(format!("Something went wrong!")))
    }
}

// helper function to get an html file response from a given url
// only used internally
async fn request_from_url(request_url: String) -> Response {
    // timeout value as a fallback to reduce the amount of retries
    let timeout = Duration::new(5, 0);

    // HTTP client with the timeout
    let client = ClientBuilder::new().timeout(timeout).build().unwrap();

    // a Result having either the HTTP Response or an Error
    let response = client.get(request_url).send().await;

    // return the Response
    response.unwrap()
}

// get anime details from zoro.to
// usage /zoro/info/<anime_id>  :   e.g.: anime_id -> jujutsu-kaisen-tv-534
#[get("/<anime_id>")]
async fn get_anime(anime_id: String) -> Result<RawJson<String>, NotFound<String>> {
    // base info url
    let base_url = format!("https://zoro.to/{}?ref=search", anime_id);

    // asynchronically called HTTP Response from request_from_url
    let resp = request_from_url(base_url).await;

    // check if the Response was successfull
    if resp.status().is_success() {
        // Parse the HTML Response
        let document = Html::parse_document(&resp.text().await.unwrap());

        // CSS Selector for the Synopsis
        let synopsis_selector = Selector::parse("div.film-description > div").unwrap();
        // Synopsis of the anime
        let synopsis_string = document
            .select(&synopsis_selector)
            .next()
            .unwrap()
            .inner_html()
            .trim()
            .to_string();

        // CSS Selector for the Name
        let name_selector = Selector::parse("h2.film-name.dynamic-name").unwrap();
        let name_string = document
            .select(&name_selector)
            .next()
            .unwrap()
            .inner_html()
            .trim()
            .to_string();

        // CSS Selector for the Poster Image
        let poster_selector = Selector::parse("img.film-poster-img").unwrap();
        let poster_string = document
            .select(&poster_selector)
            .next()
            .unwrap()
            .value()
            .attr("src")
            .unwrap()
            .to_string();

        // AnimeData Struct with the data above
        let anime_data = AnimeData {
            name: name_string,
            synopsis: synopsis_string,
            poster_image: poster_string,
        };

        // Return Result with the AnimeData converted to a JSON object
        Ok(RawJson(format!(
            "{}",
            serde_json::to_string(&anime_data).unwrap()
        )))
    }
    // If the Response was not successfull
    else {
        // Return a "JSON" that shows that something went wrong
        Ok(RawJson(format!("Something went wrong!")))
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/info", routes![info])
        .mount("/status", routes![get_status])
        .mount("/zoro", routes![search_zoro])
        .mount("/zoro/info", routes![get_anime])
}
