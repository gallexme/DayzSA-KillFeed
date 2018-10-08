extern crate bercon;
extern crate crossbeam;
extern crate glob;
extern crate logwatcher;
extern crate regex;
extern crate clap;
use bercon::becommand::BECommand;
use bercon::bepackets::RemotePacket;
use bercon::rcon::RConClient;
use glob::glob;
use logwatcher::LogWatcher;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time;
use clap::{Arg, App, SubCommand};
fn get_files_to_watch(path: &str) -> Vec<String> {
    let mut list: Vec<String> = Vec::<String>::new();
    for entry in glob(path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => list.push(path.to_str().unwrap().to_string()),
            Err(e) => println!("{:?}", e),
        }
    }
    return list;
}
fn watch_log(client: std::sync::Arc<RConClient>, path: &str) {
    loop {
        let files_to_watch = get_files_to_watch(path);
        for file in files_to_watch {
            {
                let file = file.clone();
                if std::path::Path::new(&file).is_file() {
                    let mut log_watcher = LogWatcher::register(file.clone()).unwrap();
                    log_watcher.watch(&|line: String| {
                        let re = regex::Regex::new(r#"Player\s"(?P<victimname>[^"]*)"\(id=(?P<victimid>[^\)]*)\)\shas\sbeen\skilled\sby\s(?P<killertype>\S*)\s"(?P<killername>[^"]*)"\(id=(?P<killerid>[^\)]*)"#)
                        .unwrap();
                        if re.is_match(&line) {
                            let caps = re.captures(&line).unwrap();
                            client
                                .send(BECommand::Say(
                                    1,
                                    format!(
                                        "Player {} Killed Player {} ",
                                        caps.name("killername").unwrap().as_str().to_string(),
                                        caps.name("victimname").unwrap().as_str().to_string()
                                    ),
                                ))
                                .unwrap();
                        }
                        println!("File: {} Line {}", file, line);
                    });
                }
            }
        }

        let ten_millis = time::Duration::from_millis(1000);
        thread::sleep(ten_millis);
    }
}
fn main() {
        let matches = App::new("DayzSA KillFeed")
                          .version("1.0")
                          .author("MisterOutofTime <lostchaos3@gmail.com>")
                          .about("Shows KillFeed Ingame")
                            .args_from_usage("
                            <ip> 'IP to Connect to'
                                <port> 'Port to Connect to'
                                
                                <password> 'RCON PW'
                                         <path_to_log> 'Path to AdminLog'
                                         -d... 'Turn debugging information on'")
                         
                          .get_matches();
     if let Some(o) = matches.value_of("ip") {
        println!("Value for ip: {}", o);
    }
     if let Some(o) = matches.value_of("port") {
        println!("Value for port: {}", o);
    }
     if let Some(o) = matches.value_of("password") {
        println!("Value for password: {}", o);
    }
     if let Some(o) = matches.value_of("path_to_log") {
        println!("Value for path_to_log: {}", o);
    }
    let client = Arc::new(RConClient::new("0.0.0.0".to_string(), matches.value_of("port").unwrap().parse::<u16>().unwrap()));
    let (tx, rx) = mpsc::channel();
    let ip = matches.value_of("ip").unwrap().to_string();
    let password =  matches.value_of("password").unwrap();
    let path_to_log =  matches.value_of("path_to_log").unwrap();
    crossbeam::scope(|scope| {
        {
            let client = client.clone();
            scope.spawn(move || {
                client
                    .start(ip,password, tx)
                    .unwrap();
            });
        }
        {
            let client = client.clone();
            scope.spawn(move || {
                watch_log(client, path_to_log);
            });
        }
        scope.spawn(move || loop {
            match rx.recv().unwrap() {
                RemotePacket::Login(success) => {
                    if success {
                        println!("successfully logged in.");
                        client.send(BECommand::KeepAlive).unwrap();
                    }
                }
                RemotePacket::Command(ref seq, ref data) => {
                    println!("received command response (seq# {}): {}", seq, data)
                }
                RemotePacket::Log(_, ref data) => {
                    println!("[LOG] {}", data);
                }
                _ => println!("PACKET RECEIVED"),
            };
        })
    });
}
