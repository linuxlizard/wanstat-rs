use std::env;
use std::process::ExitCode;
use reqwest::blocking::Client;
use serde_json::{Value};

fn get_password() -> Option<String>
{
    // TODO add .netrc support
    return match std::env::var("CP_PASSWORD") {
        Ok(password) => Some(password),
        Err(_) => None
    }
}

fn make_string( o: Option<&Value> ) -> String
{
    match o {
        Some(&ref s) => s.as_str().unwrap_or("(none)").to_string(),
        None => "(none)".to_string()
    }
}

fn print_connectors( v: &Value, dev: &String ) -> Option<()>
{
    if let Some(conns) = v.as_array() {

        if conns.len() > 0 {
            println!("\nconnectors for {}", dev);
            println!("                                    NAME  STATE           EXCEPTION  TIMEOUT  ");
        }

        for c in conns {
            if let Some(entry) = c.as_object() {
//                println!("entry={:?}", entry);

                let name = match entry.get("name") {
                    Some(&ref s) => s.as_str().unwrap_or("(none)").to_string(),
                    None => "(none)".to_string()
                };
                let state = make_string(entry.get("state"));
                let exception = make_string(entry.get("exception"));
                let timeout = make_string(entry.get("timeout"));

                println!("{name:>40}  {state:<15} {exception:<10} {timeout:<10}");
            };
        }
    }

    Some(())
}

fn wanstat(router_ip: &str) -> reqwest::Result<()>
{
    let password = match get_password() {
        Some(password) => password,
        _ => panic!("unable to find CP_PASSWORD in environment")
    };

    let client = Client::new();

    let target_url = format!("http://{}/api/status/wan", router_ip);

    let result = client
        .get(target_url)
        .basic_auth("admin", Some(password))
        .send();

    println!("result = {:?}", result);

    if let Ok(ref r) = result {
        println!("status={}", r.status().as_u16());
    }

    let text = result?.text()?;
//    println!("text={:?}", text);

    let j_resp:Value = serde_json::from_str(&text).unwrap();

//    println!("j_resp={:?}", j_resp);

    println!("success={}", j_resp["success"]);
   
    let success:bool = j_resp["success"].as_bool().unwrap();

    if !success {
        // TODO better error messages
        println!("transaction failed");
        return Ok(())
    }

    let j_data:&Value = &j_resp["data"];

//    let device_list:&Value = &j_data["devices"];
    let devices = j_data["devices"].as_object().unwrap();
    println!("                                    NAME TYPE       PLUGGED REASON     SUMMARY");
    for dev in devices.keys() {
//        println!("dev={}", dev);

        let fields = devices.get(dev).unwrap().as_object().unwrap();
//        for f in fields.keys() {
//            println!("f={}", f);
//        }

        let info = fields.get("info").unwrap().as_object().unwrap();
        let status = fields.get("status").unwrap().as_object().unwrap();
        let _diagnostics = fields.get("diagnostics");

        let type_ = make_string(info.get("type"));

       // boolean
       let plugged:String = match status.get("plugged") {
            Some(&ref s) => s.to_string(),
            None => "(none)".to_string()
        };

        let reason = make_string(status.get("reason"));

        let summary = make_string(status.get("summary"));

        println!("{dev:>40} {type_:<10} {plugged:<7} {reason:<10} {summary}");
        
    }

    for dev in devices.keys() {
        let fields = devices.get(dev)
                        .unwrap()
                        .as_object()
                        .unwrap();

        let printer = |conn| print_connectors(conn, dev);

        let connectors = fields.get("connectors");
        let _ = connectors.and_then(printer);
    }

    Ok(())
}

fn main() -> ExitCode {

    let args: Vec<String> = env::args().collect();
    let router_ip:&str = &args[1];

    let _ = wanstat(router_ip);

    ExitCode::SUCCESS
}

