// src/msg.rs

#[derive(Clone, Debug)]
pub enum ViewMsg {
    Default, 
    Global, 
    ReloadConfig, 
    Msg(String), 
    NewConfig(String),
    CycleLeft, 
    CycleRight, 
    DeleteMe, 
    NewTab, 
    Reply,
    Go(String), 
}
#[derive(Clone, Debug)]
pub enum InputMsg {
    Default, 
    Cancel, 
    Ack, 
    Yes, 
    No, 
    Text(String),
}
// view currently in use
#[derive(Debug, Clone)]
pub enum Focus {
    Tab,
    Global,
}
