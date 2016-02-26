//! Reports metrics to Datadog using their HTTPS API.

use chrono::UTC;
use hyper::client::{Client, RequestBuilder};
use hyper::header::ContentType;
use rustc_serialize::json::{self, Json, ToJson};
use std::collections::BTreeMap;

use super::Forwarder;
use super::super::metrics::{AggregatedMetrics, MetricType};

pub struct DatadogForwarder {
    api_key: String,
    base_url: String,
}

impl DatadogForwarder {
    pub fn new(api_key: String) -> DatadogForwarder {
        DatadogForwarder {
            api_key: api_key,
            base_url: "https://app.datadoghq.com/api".to_owned(),
        }
    }

    fn serialize_metrics(metrics: AggregatedMetrics) -> Json {
        let timestamp = UTC::now().timestamp();

        let series: Vec<Json> = metrics
            .iter()
            .map(|metric| {
                let mut object: BTreeMap<String, Json> = BTreeMap::new();

                let (ref metric_type, ref name, ref value) = *metric;

                let api_type = match *metric_type {
                    MetricType::Count   => "counter",
                    MetricType::Measure => "gauge",
                    MetricType::Sample  => "gauge",
                };

                object.insert("metric".to_owned(), name.to_json());
                object.insert("type".to_owned(), api_type.to_json());
                object.insert("points".to_owned(), Json::Array(vec![
                    Json::Array(vec![ timestamp.to_json(), value.to_json() ]),
                ]));

                object.to_json()
            })
            .collect();

        let mut data: BTreeMap<String, Json> = BTreeMap::new();
        data.insert("series".to_owned(), Json::Array(series));

        // Convert it to a `Json::Object`.
        data.to_json()
    }

    fn post<'a>(&'a self, client: &'a Client, path: &str) -> RequestBuilder {
        let path = format!("{}{}?api_key={}", self.base_url, path, self.api_key);

        client.post(&path)
            .header(ContentType::json())
    }
}

impl Forwarder for DatadogForwarder {
    fn forward_metrics(&self, metrics: AggregatedMetrics) {
        let body = json::encode(&DatadogForwarder::serialize_metrics(metrics)).unwrap();
        let client = Client::new();

        let res = self.post(&client, "/v1/series")
            .body(&body)
            .send()
            .unwrap();

        if !res.status.is_success() {
            println!("Datadog API Error: {:#?}", res);
        }
    }
}