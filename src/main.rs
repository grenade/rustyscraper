extern crate chrono;
extern crate glob;
extern crate reqwest;
extern crate rust_decimal;
extern crate select;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;

use chrono::NaiveDate;
use glob::glob;

use rust_decimal::Decimal;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};

use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::str::FromStr;
use std::io::prelude::*;


//https://stackoverflow.com/a/28392068/68115
macro_rules! btreemap {
  ($( $key: expr => $val: expr ),*) => {{
    let mut map = ::std::collections::BTreeMap::new();
    $( map.insert($key, $val); )*
    map
  }}
}

fn main() {
  let path = "/home/rob/git/grenade/crypto-market-data";
  coin_market_cap_all_today("https://coinmarketcap.com/all/views/all/", &path);
  for file in glob(&format!("{}/today/json/*.json", &path)).expect("unable to interpret glob pattern") {
    let id = file.as_ref().unwrap().file_stem().unwrap().to_str().unwrap();
    if std::path::Path::new(&format!("{}/history/json/{}.json", &path, &id)).exists() {
      println!("skipping history for {}",  &id);
    } else {
      coin_market_cap_history(&id, &format!("https://coinmarketcap.com/currencies/{}/historical-data/?start=20130428&end=20190625", &id), &path);
      std::thread::sleep(std::time::Duration::from_secs(6));
    }
  }
}

fn coin_market_cap_all_today(url: &str, path: &str) {
  if std::path::Path::new(&format!("{}/today", path)).exists() {
    fs::remove_dir_all(format!("{}/today", path)).expect("unable to delete directory");
  }
  for format in vec!["json", "yaml"].iter() {
    fs::create_dir_all(format!("{}/today/{}", path, format)).expect("unable to create directory");
  }
  let response = reqwest::get(url).unwrap();
  assert!(response.status().is_success());
  let document = Document::from_read(response).unwrap();

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
    let json_crypto_path = format!("{}/today/json/{}.json", path, crypto.id);
    fs::write(&json_crypto_path, &json_crypto).expect("unable to write json file");
    println!("{} updated",  json_crypto_path);

    let yaml_crypto = serde_yaml::to_string(&crypto).unwrap();
    let yaml_crypto_path = format!("{}/today/yaml/{}.yaml", path, crypto.id);
    fs::write(&yaml_crypto_path, &yaml_crypto).expect("unable to write yaml file");
    println!("{} updated",  yaml_crypto_path);

    all.push(crypto);
  }
  /*
  write_to_file(&all, &format!("{}/today/all.json", path));
  write_to_file(&all, &format!("{}/today/all.yaml", path));

  write_to_file(&all[..10], &format!("{}/today/top-10.json", path));
  write_to_file(&all[..10], &format!("{}/today/top-10.yaml", path));

  write_to_file(&all[..10], &format!("{}/today/top-10.json", path));
  write_to_file(&all[..10], &format!("{}/today/top-10.yaml", path));
  */

  let json_all = serde_json::to_string_pretty(&all).unwrap();
  let json_all_path = format!("{}/today/all.json", path);
  fs::write(&json_all_path, &json_all).expect("unable to write json file");
  println!("{} updated",  json_all_path);
  
  let yaml_all = serde_yaml::to_string(&all).unwrap();
  let yaml_all_path = format!("{}/today/all.yaml", path);
  fs::write(&yaml_all_path, &yaml_all).expect("unable to write yaml file");
  println!("{} updated",  yaml_all_path);

  let json_top_10 = serde_json::to_string_pretty(&all[..10]).unwrap();
  let json_top_10_path = format!("{}/today/top-10.json", path);
  fs::write(&json_top_10_path, &json_top_10).expect("unable to write json file");
  println!("{} updated",  json_top_10_path);
  
  let yaml_top_10 = serde_yaml::to_string(&all[..10]).unwrap();
  let yaml_top_10_path = format!("{}/today/top-10.yaml", path);
  fs::write(&yaml_top_10_path, &yaml_top_10).expect("unable to write yaml file");
  println!("{} updated",  yaml_top_10_path);

  let json_top_100 = serde_json::to_string_pretty(&all[..100]).unwrap();
  let json_top_100_path = format!("{}/today/top-100.json", path);
  fs::write(&json_top_100_path, &json_top_100).expect("unable to write json file");
  println!("{} updated",  json_top_100_path);
  
  let yaml_top_100 = serde_yaml::to_string(&all[..100]).unwrap();
  let yaml_top_100_path = format!("{}/today/top-100.yaml", path);
  fs::write(&yaml_top_100_path, &yaml_top_100).expect("unable to write yaml file");
  println!("{} updated",  yaml_top_100_path);
}

fn coin_market_cap_history(id: &str, url: &str, path: &str) {
  for format in vec!["json", "yaml"].iter() {
    fs::create_dir_all(format!("{}/history/{}", path, format)).expect("unable to create directory");
  }
  let response = reqwest::get(url).unwrap();
  if response.status().is_success() {
    let document = Document::from_read(response).unwrap();

    let mut history = Vec::new();

    let keys = vec!["date_raw", "open", "high", "low", "close", "volume", "market_cap"];
    for row in document.find(Attr("id", "historical-data").descendant(Name("tbody").descendant(Name("tr")))) {
      let date_raw = row.find(Class("text-left")).next().unwrap().text();
      let date_parsed = NaiveDate::parse_from_str(&date_raw, "%b %d, %Y").expect("unable to parse date");

      let mut props = BTreeMap::new();
      let mut cell_index = 0;
      for cell in row.find(Name("td")) {
        match cell_index {
          0 => {
            props.insert(keys[cell_index].to_string(), Value {
              as_string: date_parsed.to_string(),
              as_formatted_string: date_raw.to_string(),
            });
          }
          _ => {
            props.insert(keys[cell_index].to_string(), Value {
              as_string: cell.attr("data-format-value").unwrap().to_string(),
              as_formatted_string: cell.text().to_string(),
            });
          }
        }
        cell_index += 1;
      }
      history.push(props);
    }
    println!("{} updated: {} - {}", id, last(&history).unwrap()["date_raw"].as_string, first(&history).unwrap()["date_raw"].as_string);

    let json_history = serde_json::to_string_pretty(&history).unwrap();
    let json_history_path = format!("{}/history/json/{}.json", path, id);
    fs::write(&json_history_path, &json_history).expect("unable to write json file");
    println!("{} updated",  json_history_path);

    let yaml_history = serde_yaml::to_string(&history).unwrap();
    let yaml_history_path = format!("{}/history/yaml/{}.yaml", path, id);
    fs::write(&yaml_history_path, &yaml_history).expect("unable to write yaml file");
    println!("{} updated",  yaml_history_path);
  } else {
    println!("failed to fetch {}. request status: {:?}",  &url, response.status());
    std::thread::sleep(std::time::Duration::from_secs(60));
  }
}

fn first<T>(v: &Vec<T>) -> Option<&T> {
  v.first()
}

fn last<T>(v: &Vec<T>) -> Option<&T> {
  v.last()
}

/*
fn build_daily_charts(path: &str) {
  for path in glob(&format!("{}/history/json/bitcoin.json", &path)).expect("unable to interpret glob pattern") {
    let mut file = fs::File::open(path);
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    let v: Vec<BTreeMap<str, Value>> = serde_json::from_str(contents).unwrap();
  }
  
  /*
  let mut dt = dt0;
  while dt <= dt1 {
    println!("{:?}", dt);
    dt = dt + Duration::days(1);
  }
  */
}
*/

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
struct Value {
  as_string: String,
  as_formatted_string: String,
}

/*
#[derive(Debug, Serialize, Deserialize)]
struct CryptoDay {
  date: NaiveDate,
  id: String,
  cap: i64,
  high: Decimal,
  low: Decimal,
}
*/