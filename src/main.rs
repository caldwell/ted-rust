#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate sxd_document;
extern crate sxd_xpath;
extern crate reqwest;
extern crate serde_json;

use docopt::Docopt;
use sxd_document::parser;
use sxd_xpath::{evaluate_xpath, Value};

#[derive(Debug, Deserialize)]
struct Args {
    flag_verbose: bool,
    flag_h: String,
    flag_s: i32,
}

#[derive(Debug, Serialize)]
struct Output {
    power: f64,
    voltage: f64,
    mtu: String,
}

fn main() {
    let args: Args = Docopt::new(format!("
Usage: {} [options]

--help         Show this.
-v --verbose   Print more text.
-o FILE        Specify output file [default: ./test.txt].
-h <ted-host>  TED6000 host [default: ted].
-s <seconds>   Number of seconds to average [default: 10]
", std::env::current_exe().unwrap().file_name().unwrap().to_str().unwrap())).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());
    if args.flag_verbose { eprintln!("{:?}", args); }

    let url = format!("http://{}/history/export.xml?T=1&D=0&M=1&C={}", args.flag_h, args.flag_s);
    if args.flag_verbose { eprintln!("GET {}", url) }
    let text = reqwest::get(&url).expect(&format!("Failed to GET {}", url)).text().expect("Failed to get body for request");
    if args.flag_verbose { eprintln!("=> {}", text); }
    let package = parser::parse(&text).expect(&format!("Failed to parse XML: {}", text));
    let doc = package.as_document();

    let power_val   = evaluate_xpath(&doc, "//POWER/text()")  .expect(&format!("Couldn't find <POWER> in: {}",   text));
    let voltage_val = evaluate_xpath(&doc, "//VOLTAGE/text()").expect(&format!("Couldn't find <VOLTAGE> in: {}", text));
    let out = match (power_val, voltage_val) {
        (Value::Nodeset(pow_ns), Value::Nodeset(volt_ns)) => {
            Output {
                power:   mean(pow_ns.iter() .map(|n| { let s = n.string_value(); s.parse::<f64>().expect(&format!("Couldn't parse <POWER> float '{}'", s))})),
                voltage: mean(volt_ns.iter().map(|n| { let s = n.string_value(); s.parse::<f64>().expect(&format!("Couldn't parse <VOLTAGE> float '{}'", s))})),
                mtu: "mains".to_string(),
            }
        },
        _ => panic!("//POWER or //VOLTAGE didn't return a Nodeset {}", text),
    };
    println!("{}", serde_json::to_string(&out).unwrap());
}

fn mean<I>(it: I) -> f64 where I: Iterator<Item = f64> {
    let (count, sum) = it.fold((0.0,0.0), |(c, s), n| (c+1.0, s+n));
    sum / count
}
