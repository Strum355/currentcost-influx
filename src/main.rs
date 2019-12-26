use influxdb::{Client, Timestamp, WriteQuery};
use quick_xml::de::from_str;
use regex::Regex;
use serde::Deserialize;
use serialport::open_with_settings;
use serialport::prelude::*;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Config {
    #[structopt(short, long, default_value = "/dev/ttyUSB0")]
    usb_path: String,

    #[structopt(short, long, default_value = "http://localhost:8086")]
    influx_url: String,
}

#[derive(Debug, Deserialize)]
struct Message {
    #[serde(rename = "src")]
    source: String,
    time: String,
    #[serde(rename = "tmpr")]
    temperature: f32,
    #[serde(rename = "sensor")]
    sensor_num: u8,
    #[serde(rename = "ch")]
    sensors_watts: Vec<Watt>,
}

#[derive(Debug, Deserialize)]
struct Watt {
    watts: u16,
}

#[tokio::main]
async fn main() {
    let opt: Config = Config::from_args();
    
    let influx = Client::new(opt.influx_url, "currentcost");

    let result = influx.ping().await;
    if result.is_err() {
        panic!("couldn't ping influxdb: {}", result.err().unwrap());
    }

    let serial = open_with_settings(
        opt.usb_path.as_str(),
        &SerialPortSettings {
            baud_rate: 57600,
            timeout: Duration::from_secs(15),
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            stop_bits: StopBits::One,
            parity: Parity::None,
        },
    ).unwrap();

    let re = Regex::new(r"<(/?ch)\d>").unwrap();
    let mut reader = BufReader::new(serial);
    loop {
        let mut buf = String::new();
        let cleaned = match reader.read_line(&mut buf) {
            Ok(_) => re.replace_all(buf.as_str().trim(), "<$1>"),
            Err(err) => {
                eprintln!("error reading from serial: {}", err);
                continue;
            }
        };

        let msg = match from_str::<Message>(cleaned.to_string().as_str()) {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("error deserializing: {}\n\t{}", err, buf.as_str().trim());
                continue;
            }
        };

        for (i, sensor) in msg.sensors_watts.iter().enumerate() {
            let query = WriteQuery::new(Timestamp::Now, "watts")
                .add_tag("model", msg.source.clone())
                .add_tag("channel", format!("channel{}", i + 1))
                .add_tag("sensor_num", msg.sensor_num)
                .add_field("watts", sensor.watts);
            match influx.query(&query).await {
                Ok(_) => (),
                Err(err) => eprintln!("error writing wattage datapoint: {}", err),
            }
        }

        let query = WriteQuery::new(Timestamp::Now, "temperature")
            .add_tag("model", msg.source.clone())
            .add_tag("sensor_num", msg.sensor_num)
            .add_field("temp", msg.temperature);
        match influx.query(&query).await {
            Ok(_) => (),
            Err(err) => eprintln!("error writing temperature datapoint: {}", err),
        }
    }
}
