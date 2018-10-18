use std::net;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::mpsc::Sender;
use std::thread;
use std::time;

use becommand::BECommand;
use bepackets::{parse_packet, RemotePacket};
use crossbeam;
use packet::{construct, RconMessageType};
use rcon_error::RconError;

pub struct RConClient {
    socket: net::UdpSocket,
    port: u16,
    seq: AtomicU8,
    logged_in: AtomicBool,

    waiting_for_ack: AtomicBool,
    alive_thread_started: AtomicBool,
}

use std::str::FromStr;
impl RConClient {
    pub fn new(ip: String, port: u16) -> Self {
        let ip = net::Ipv4Addr::from_str(&ip).unwrap();
        let this_thing = net::SocketAddrV4::new(ip, 23308);

        let socket = net::UdpSocket::bind(this_thing).unwrap();

        RConClient {
            port: port,
            socket: socket,
            seq: AtomicU8::new(0),
            logged_in: AtomicBool::new(false),
            waiting_for_ack: AtomicBool::new(false),
            alive_thread_started: AtomicBool::new(false),
        }
    }

    pub fn start(
        &self,
        ip: String,
        password: &str,
        tx: Sender<RemotePacket>,
    ) -> Result<(), RconError> {
        self.logged_in.store(false, Ordering::SeqCst);
        try!(self.connect(ip));
        crossbeam::scope(|scope| {
            scope.spawn(move || loop {
                if !self.logged_in.load(Ordering::SeqCst) {
                    self.alive_thread_started.store(false, Ordering::SeqCst);
                } else {
                    self.alive_thread_started.store(true, Ordering::SeqCst);
                }
                thread::sleep(time::Duration::from_secs(45));
                
                self.send(BECommand::Login(password.into())).unwrap();
                //self.send(BECommand::KeepAlive).unwrap();
              //  println!("sent keep-alive");
            });

            scope.spawn(move || {
                self.send(BECommand::Login(password.into())).unwrap();
                loop {
                    let mut recv: Vec<u8> = vec![];
                    // This should be optimized somehow? feels way too hacky
                    recv.resize(512, 0x0);
                    let c = self.socket.recv_from(&mut recv).unwrap();
                    recv.resize(c.0, 0x0);

                    let rp = parse_packet(recv);
                   // println!("Received Packet: {:#?}", rp);
                    self.send_ack(&rp);
                    tx.send(rp.clone()).unwrap();

                    match rp {
                        RemotePacket::Login(success) => {
                            if !success {
                                panic!("could not log in");
                            }
                            self.logged_in.store(true, Ordering::SeqCst);
                        }
                        RemotePacket::Command(0, _) => {
                            self.waiting_for_ack.store(false, Ordering::SeqCst);
                        }
                        _ => (),
                    };
                }
            });
        });
        Ok(())
    }

    fn connect(&self, ip: String) -> Result<(), RconError> {
        let ip = net::Ipv4Addr::from_str(&ip).unwrap();
        let be_server = net::SocketAddrV4::new(ip, self.port);

        Ok(try!(self.socket.connect(net::SocketAddr::V4(be_server))))
    }

    fn send_ack(&self, rp: &RemotePacket) -> bool {
        match rp {
            &RemotePacket::Log(seq, _) => {
               // println!("SENDICK ACK for SQUENCE: {}", seq);
                self.socket
                    .send(&construct(RconMessageType::Log, vec![seq]))
                    .is_ok()
            }
            _ => true,
        }
    }

    fn prepend_seq(&self, mut vec: Vec<u8>) -> Vec<u8> {
        vec.insert(0, self.seq.load(Ordering::SeqCst));
        self.seq.fetch_add(1, Ordering::SeqCst);
        vec
    }

    pub fn send(&self, command: BECommand) -> Result<usize, RconError> {
        if self.waiting_for_ack.load(Ordering::SeqCst) {
            loop {
                if !self.waiting_for_ack.load(Ordering::SeqCst) {
                    std::thread::sleep_ms(10);
                    break;
                    
                }
            }
        }
        //println!("SEND {:#?}", command);
        let vec = match command {
            BECommand::Login(password) => construct(RconMessageType::Login, password.into_bytes()),
            BECommand::KeepAlive => {
                self.waiting_for_ack.store(true, Ordering::SeqCst);
                construct(RconMessageType::Command, vec![0x00])
            }

            BECommand::Say(channel, msg) => {
                let msg: String = ["say", &channel.to_string(), &msg].join(" ");
             //   println!("BECHATCOMMAND: {}", msg);
                construct(RconMessageType::Command, self.prepend_seq(Vec::from(msg)))
            }

            BECommand::Players => construct(
                RconMessageType::Command,
                self.prepend_seq(Vec::from("players")),
            ),
            _ => unimplemented!(),
        };

        Ok(try!(self.socket.send(&vec)))
    }
}
