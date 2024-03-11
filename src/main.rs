use std::process::ExitCode;
use reqwest::blocking::Client;
use serde_json::{json, Value};

fn get_password() -> Option<String>
{
    // TODO add .netrc support
    return match std::env::var("CP_PASSWORD") {
        Ok(password) => Some(password),
        Err(_) => None
    }
}

fn _simple()
{
    let password = match get_password() {
        Some(password) => password,
        _ => panic!("unable to find CP_PASSWORD in environment")
    };

    let client = Client::new();

    let result = client
        .get("http://172.16.253.1/api/status/wlan/state")
        .basic_auth("admin", Some(password))
        .send();

    println!("result = {:?}", result);

    let response = result.unwrap();
    println!("response={:?}", response);
    println!("status={}", response.status());
    let text = response.text().unwrap();
    let j_data = json!(text);
    println!("j_data={:?}", j_data);
}

fn make_string( o: Option<&Value> ) -> String
{
    match o {
        Some(&ref s) => s.as_str().unwrap_or("(none)").to_string(),
        None => "(none)".to_string()
    }
}

fn print_connectors( v: &Value) -> Option<()>
{
    if let Some(conns) = v.as_array() {

        if conns.len() > 0 {
            println!("\n                                    NAME  STATE           EXCEPTION  TIMEOUT  ");
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

    if v.is_array() {

    }



    Some(())
}

fn harder() -> reqwest::Result<()>
{
    let password = match get_password() {
        Some(password) => password,
        _ => panic!("unable to find CP_PASSWORD in environment")
    };

    let client = Client::new();

    let result = client
//        .get("http://172.16.253.1/api/status/wlan/state")
        .get("http://172.16.253.1/api/status/wan")
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

        let connectors = fields.get("connectors");
        let _ = connectors.and_then(print_connectors);
    }

    Ok(())
}

fn main() -> ExitCode {
//    simple();

    let _ = harder();

//    if let Ok(ref response) = result {
//        println!("response={:?}", *response);
//        println!("status={}", (*response).status());
//
//        let ref text = (*response).text();
//
////        println!("text={}", response.text().unwrap() );
////        if let Ok(data) = &response.text() {
////            let j_data = json!(data);
////            println!("j_data={:?}", j_data);
////        }
//    }

//    println!("result = {:?}", result);
    ExitCode::SUCCESS
}

