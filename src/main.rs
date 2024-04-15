use std::env;
use std::process::ExitCode;
use reqwest::blocking::Client;
use serde;
use serde_json::{Value};
//use std::net::{IpAddr,};
use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr};

pub trait Connector 
{
    fn print(&self) -> String;
}

// connector we don't have a full parser for
pub struct GenericConnector
{
    pub name: String,
    pub enabled: bool,
    pub state: String
}

impl Connector for GenericConnector
{
    fn print(&self) -> String {
        format!("{} {} {}", self.name, self.enabled, self.state)
    }
}

pub struct WiFiClientConnector
{
    pub name: String,
    pub ssid: String,
    pub signal_strength : Option<i32>,
    pub channel : Option<u32>,
    pub enabled : bool,
    pub state : String,
}

impl Connector for WiFiClientConnector
{
    fn print(&self) -> String {
        // TODO extract from Option

        let signal_strength:String = match self.signal_strength {
            Some(v) => v.to_string(),
            None => "<unset>".to_string()
        };

        let channel:String = match self.channel {
            Some(v) => v.to_string(),
            None => "<unset>".to_string()
        };

        format!("{} {} {} \"{}\" rssi={} channel={}", 
            self.name, 
            self.enabled, 
            self.state, 
            self.ssid, 
            signal_strength, 
            channel)
    }
}

#[derive(Debug,serde::Serialize,serde::Deserialize)]
pub struct IPInfo
{
    pub ip_address : IpAddr,
    pub netmask : IpAddr,
    pub gateway: IpAddr,
    pub dnslist: Vec<IpAddr>
}

impl std::fmt::Display for IPInfo
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dnslist:String = self.dnslist
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",")
                ;

        write!(f, "ip={} sm={} gw={} dns=[{}]", 
            self.ip_address, 
            self.netmask, 
            self.gateway,
            dnslist
        )
    }
}


pub struct DHCPConnector 
{
    pub name: String,
    pub ipinfo : Option<IPInfo>,

    pub enabled : bool,
    pub state : String,
}

impl Connector for DHCPConnector
{
    fn print(&self) -> String {
        let s_ipinfo:String = match &self.ipinfo {
            Some(ipinfo) => ipinfo.to_string(),
            None => "<none>".to_string()
        };

        format!("{} {} {} ipinfo={}", self.name, self.enabled, self.state, s_ipinfo)
    }
}

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

//fn get_none() -> &'static str
//{
//    const NONE: &str = "(none)";
//    NONE
//}

fn str_or_none<'a>( entry: &'a serde_json::Map<String, Value>, key: &str) -> &'a str
{
    const NONE: &str = "(none)";

    match entry.get(key) {
        Some(s) => s.as_str().unwrap_or(NONE),
        None => NONE
    }
}

fn get_i32( field: Option<&Value> ) -> Option<i32>
{
    match field?.as_f64()? {
        n => Some(n as i32),
    }
}

fn get_u32( field: Option<&Value> ) -> Option<u32>
{
    match field?.as_u64()? {
        n => Some(n as u32),
    }
}

fn parse_connector( fields: &serde_json::Map<String,Value>, conn: &serde_json::Map<String, Value> ) -> Box<dyn Connector>
{
    let name:&str = str_or_none(conn, "name");

    let state:String = make_string(conn.get("state"));

    let enabled:bool = conn.get("enabled").unwrap().as_bool().unwrap();

//    for field in conn.keys() {
//        println!("parse_connector field={:?}", field);
//    }

//    for v in conn {
//        let (a,b) = v;
//        println!("parse_connector a={:?}", a);
//        println!("parse_connector b={:?}", b);
//    }

//    println!("parse connector name={}", name);

    match name {
        "WiFiClient" => {
            let diagnostics = fields.get("diagnostics").unwrap().as_object().unwrap();
            Box::new(
                WiFiClientConnector {
                    name: String::from(name),
                    ssid: str_or_none(diagnostics, "SSID").to_string(),
                    signal_strength: get_i32(diagnostics.get("signal_strength")),
                    channel: get_u32(diagnostics.get("channel")),
                    enabled: enabled,
                    state: state
                })
        },

        "DHCP" => {
            println!("{} ipinfo get={:?}", name, conn.get("ipinfo"));

            Box::new(
                DHCPConnector {
                    name: String::from(name),
                    ipinfo: parse_conn_ipinfo(conn),
                    enabled: enabled,
                    state: state
                })
        },
    
        _ => {
            Box::new(
                GenericConnector {
                    name: String::from(name),
                    enabled: enabled,
                    state: state
                })
        }
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

fn get_connectors(fields: &serde_json::Map<String,Value> ) -> Option<Vec<Box<dyn Connector>>>
{
    let connectors = fields.get("connectors");

    Some(connectors?
        .as_array()?
        .iter()
        .filter_map(|c| c.as_object() )
        .map(|cc| parse_connector(fields, cc))
        .collect())
}

fn parse_ipinfo( contents: &serde_json::Map<String,Value> ) -> Option<IPInfo>
{
    let mut ipinfo:IPInfo = IPInfo {
        ip_address: IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
        netmask: IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
        gateway: IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
        dnslist: vec![]
    };
    
    for field in contents {
        let (key,value) = field;

//        println!("field={:?} {}", field, value.is_string());

        if value.is_string() {
            let ip_str = value.as_str()
                            .unwrap();
//            println!("field={:?} ip={}", field, ip_str);
            
            let ip = IpAddr::from_str(ip_str).unwrap();
//            println!("field={:?} ip={} rip={:?}", field, ip_str, ip);

            match key.as_str() {
                "ip_address" => { ipinfo.ip_address = ip; },
                "netmask" => { ipinfo.netmask = ip; },
                "gateway" => { ipinfo.gateway = ip; },
                _ => {}
            };
        }
        else if value.is_array() {
            // parse out the dnslist
            for ipv in value.as_array().unwrap() {
                println!("dns ip={}", ipv);
            }
            ipinfo.dnslist = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|i| IpAddr::from_str(i.as_str().unwrap()).unwrap())
                        .collect()
                        ;
        }
    }

    println!("ipinfo={:?}", ipinfo);
    println!("ipinfo={}", ipinfo);

    Some(ipinfo)
}

fn parse_conn_ipinfo(conn: &serde_json::Map<String,Value>) -> Option<IPInfo>
{
//    let ipi = conn.get("ipinfo")?
//        .as_object()?
//        ;
//    parse_ipinfo(&ipi)
    parse_ipinfo(conn.get("ipinfo")?
        .as_object()?
        )
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

//    println!("result = {:?}", result);

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
        let _diagnostics = fields.get("diagnostics");

        let printer = |conns| print_connectors(conns, dev);

        let _ = fields.get("connectors").and_then(printer);

        let connectors:Option<Vec<Box<dyn Connector>>> = get_connectors(fields);
        if let Some(conns) = connectors {
            for c in conns {
                println!("c={}", c.print());
            }
        }

    }

    Ok(())
}

fn main() -> ExitCode {

    let args: Vec<String> = env::args().collect();
    let router_ip:&str = &args[1];

    let _ = wanstat(router_ip);

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ipinfo() 
    {
        let j = json!({
          "ipinfo": {
            "gateway": "192.168.1.1",
            "ip_address": "192.168.1.9",
            "netmask": "255.255.255.0",
            "dnslist": [
              "192.168.1.1"
            ]
          }});
        println!("test j={:?}", j);

        let ipinfo = parse_ipinfo(j.get("ipinfo").unwrap().as_object().unwrap()).expect("failed to parse");
        println!("parsed ipinfo={}", ipinfo);
        assert_eq!(ipinfo.ip_address.is_ipv4(), true);
        assert_eq!(ipinfo.ip_address.to_string(), "192.168.1.9");
        assert_eq!(ipinfo.gateway.to_string(), "192.168.1.1");
        assert_eq!(ipinfo.netmask.to_string(), "255.255.255.0");
        assert_eq!(ipinfo.dnslist.len(), 1);
    }

    #[test]
    fn test_parse_2ipinfo_conn()
    {
        let devices = json!({
            "connectors" : [{
              "name": "DHCP",
              "enabled": true,
              "traits": [
                "ip"
              ],
              "state": "connected",
              "ipinfo": {
                "gateway": "172.16.253.1",
                "ip_address": "172.16.253.42",
                "netmask": "255.255.255.0",
                "dnslist": [
                  "172.16.253.1",
                  "8.8.8.8"
                ]
              },
              "ip6info": null,
              "exception": null,
              "timeout": null,
              "dhclient_state": "STARTED"
            }]});

//        let conn = json!({});
        let conn = devices 
                    .get("connectors")
                    .expect("should have found connectors")
                    .get(0)
                    .expect("should have found 0");

        println!("conn isobj={} {:?}", conn.is_object(), conn);
        println!("name={:?}", conn.get("name"));
        println!("ipinfo={:?}", conn.get("ipinfo"));

        let ipinfo = conn.get("ipinfo").expect("should have found ipinfo");
        println!("ipinfo={:?}", ipinfo);

        let ipi = parse_conn_ipinfo(conn.as_object().unwrap()).expect("failed to parse 2");
         println!("parsed ipinfo={}", ipi);
    }

    #[test]
    fn test_parse_invalid_ip() 
    {
        let ip_str = "invalid";
        let ip = IpAddr::from_str(ip_str);
        match ip {
            Ok(_) => { println!("parse ok"); },
            Err(e) => { println!("err={}", e); }
        }
//        assert_eq!(ip, Err(err));
    }

    #[test]
    fn test_bad_strings()
    {
        let j = json!({
          "ipinfo": {
            "gateway": "192.168.1.1",
            "ip_address": "192.168.1.9",
            "netmask": "255.255.255.0",
            "dnslist": [
              "192.168.1.1",
            ]
          }});
        let ipinfo = parse_ipinfo(j.as_object().unwrap()).expect("failed to parse");
        println!("parsed ipinfo={}", ipinfo);
    }

    #[test]
    fn test_strongly_typed() 
    {
        let s = r#"
              {
                "gateway": "172.16.253.1",
                "ip_address": "172.16.253.42",
                "netmask": "255.255.255.0",
                "dnslist": [
                  "172.16.253.1",
                  "8.8.8.8"
                ]
              }"#;

        match serde_json::from_str::<IPInfo>(s) {
            Ok(ipinfo) => { println!("found ipinfo={}", ipinfo); },
            Err(err) => { println!("failed err={}", err); }
        };
    }
}

