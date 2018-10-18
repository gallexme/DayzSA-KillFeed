#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum BECommand {
    KeepAlive,
    Login(String),

    LoadBans,
    Bans,
    Ban(String, u16, Option<String>),
    AddBan(String, u16, Option<String>),
    RemoveBan(u16),
    WriteBans,

    LoadScripts,
    Missions,
    Players,
    Kick(u16, String),
    RConPassword(String),
    MaxPing(u16),
    Logout,
    Exit,
    Say(i16, String),
}