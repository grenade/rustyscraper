extern crate reqwest;
extern crate select;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;

use select::document::Document;
use select::predicate::{Class, Name, Predicate};

use std::collections::BTreeMap;
use std::fs;

//https://stackoverflow.com/a/28392068/68115
macro_rules! btreemap {
  ($( $key: expr => $val: expr ),*) => {{
    let mut map = ::std::collections::BTreeMap::new();
    $( map.insert($key, $val); )*
    map
  }}
}

fn main() {
  coin_market_cap("https://coinmarketcap.com/all/views/all/", "/home/rob/git/grenade/crypto-market-data");
}

fn coin_market_cap(url: &str, path: &str) {
  for format in vec!["json", "yaml"].iter() {
    fs::remove_dir_all(format!("{}/{}", path, format)).expect("unable to delete directory");
    fs::create_dir_all(format!("{}/{}", path, format)).expect("unable to create directory");
  }
  let resp = reqwest::get(url).unwrap();
  assert!(resp.status().is_success());
  let document = Document::from_read(resp).unwrap();

  let mut all = Vec::new();

  for row in document.find(Name("tbody").descendant(Name("tr"))) {

    let symbol = row.find(Class("currency-symbol").descendant(Name("a"))).next().unwrap().text();
    let market_cap = row.find(Class("market-cap")).next().unwrap();
    let price = row.find(Class("price")).next().unwrap();
    let volume = row.find(Class("volume")).next().unwrap();
    let mut percent_change = BTreeMap::new();
    for cell in row.find(Class("percent-change")) {
      percent_change.insert(cell.attr("data-timespan").unwrap().to_string(), cell.attr("data-percentusd").unwrap().to_string());
    }

    let crypto = Crypto {
      id: row.attr("id").unwrap()[3..].to_string(),
      rank: row.find(Class("text-center")).next().unwrap().text().trim().to_string().parse::<i32>().unwrap(),
      symbol: symbol.to_string(),
      name: row.find(Class("currency-name")).next().unwrap().attr("data-sort").unwrap().to_string(),
      market_cap: btreemap!["BTC".to_string() => market_cap.attr("data-btc").unwrap().to_string().replace("?", ""), "USD".to_string() => market_cap.attr("data-usd").unwrap().to_string().replace("?", "")],
      price: btreemap!["BTC".to_string() => price.attr("data-btc").unwrap().to_string(), "USD".to_string() => price.attr("data-usd").unwrap().to_string()],
      volume: btreemap!["BTC".to_string() => volume.attr("data-btc").unwrap().to_string().replace("None", "0"), "USD".to_string() => volume.attr("data-usd").unwrap().to_string().replace("None", "0")],
      supply: btreemap![symbol.to_string() => row.find(Class("circulating-supply")).next().unwrap().attr("data-sort").unwrap().to_string().replace("-1", "0")],
      change: percent_change,
      is_mineable: row.find(Class("circulating-supply")).next().unwrap().text().trim().chars().last().unwrap() != '*',
    };

    let json_crypto = serde_json::to_string_pretty(&crypto).unwrap();
    fs::write(format!("{}/json/{}.json", path, crypto.id), &json_crypto).expect("unable to write json file");
    //println!("{}",  json_crypto);

    let yaml_crypto = serde_yaml::to_string(&crypto).unwrap();
    fs::write(format!("{}/yaml/{}.yaml", path, crypto.id), &yaml_crypto).expect("unable to write yaml file");
    //println!("{}",  yaml_crypto);

    all.push(crypto);
  }
  let json_all = serde_json::to_string_pretty(&all).unwrap();
  fs::write(format!("{}/all.json", path), &json_all).expect("unable to write json file");
  let yaml_all = serde_yaml::to_string(&all).unwrap();
  fs::write(format!("{}/all.yaml", path), &yaml_all).expect("unable to write yaml file");

  let json_top_10 = serde_json::to_string_pretty(&all[..10]).unwrap();
  fs::write(format!("{}/top-10.json", path), &json_top_10).expect("unable to write json file");
  let yaml_top_10 = serde_yaml::to_string(&all[..10]).unwrap();
  fs::write(format!("{}/top-10.yaml", path), &yaml_top_10).expect("unable to write yaml file");

  let json_top_100 = serde_json::to_string_pretty(&all[..100]).unwrap();
  fs::write(format!("{}/top-100.json", path), &json_top_100).expect("unable to write json file");
  let yaml_top_100 = serde_yaml::to_string(&all[..100]).unwrap();
  fs::write(format!("{}/top-100.yaml", path), &yaml_top_100).expect("unable to write yaml file");
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
struct Crypto {
  id: String,
  rank: i32,
  symbol: String,
  name: String,
  market_cap: BTreeMap<String, String>,
  price: BTreeMap<String, String>,
  volume: BTreeMap<String, String>,
  supply: BTreeMap<String, String>,
  change: BTreeMap<String, String>,
  is_mineable: bool,
}