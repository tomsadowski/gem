// gemstatus

use regex::Regex;
use std::str::FromStr;
use crate::{
    constants,
};

#[derive(Debug, Clone)]
pub enum Input {
    Input(i16),
    Sensitive,
}
#[derive(Debug, Clone)]
pub enum Success {
    Success(i16),
}
#[derive(Debug, Clone)]
pub enum Redirect {
    Temporary(i16),
    Permanent,
}
#[derive(Debug, Clone)]
pub enum TemporaryFailure {
    TemporaryFailure(i16),
    ServerUnavailable,
    CGIError,
    ProxyError,
    SlowDown,
}
#[derive(Debug, Clone)]
pub enum PermanentFailure {
    PermanentFailure(i16),
    NotFound,             
    Gone,                 
    ProxyRequestRefused,  
    BadRequest,           
}
#[derive(Debug, Clone)]
pub enum ClientCertificateRequired {
    ClientCertificateRequired(i16),
    TransientCertificateRequired,   
    AuthorizedCertificateRequired,  
    CertificateNotAccepted,         
    FutureCertificateRejected,      
    ExpiredCertificateRejected,     
} 
#[derive(Debug, Clone)]
pub enum Status {
    InputExpected(Input, String),
    Success(Success, String),
    Redirect(Redirect, String),
    TemporaryFailure(TemporaryFailure, String),
    PermanentFailure(PermanentFailure, String),
    ClientCertificateRequired(ClientCertificateRequired, String),
}

impl Input {
    pub fn new(code: i16) -> Option<Self> {
        match code {
            10      => Some(Self::Input(code)),
            11      => Some(Self::Sensitive),
            12..=19 => Some(Self::Input(code)),
            _       => None,
        }
    }
}
impl Success {
    pub fn new(code: i16) -> Option<Self> {
        match code {
            20..=29 => Some(Self::Success(code)),
            _       => None,
        }
    }
}
impl Redirect {
    pub fn new(code: i16) -> Option<Self> {
        match code {
            30      => Some(Self::Temporary(code)),
            31      => Some(Self::Permanent),
            32..=39 => Some(Self::Temporary(code)),
            _       => None,
        }
    }
}
impl TemporaryFailure {
    pub fn new(code: i16) -> Option<Self> {
        match code {
            40      => Some(Self::TemporaryFailure(code)),
            41      => Some(Self::ServerUnavailable),
            42      => Some(Self::CGIError),
            43      => Some(Self::ProxyError),
            44      => Some(Self::SlowDown),          
            45..=49 => Some(Self::TemporaryFailure(code)),
            _       => None,
        }
    }
}
impl PermanentFailure {
    pub fn new(code: i16) -> Option<Self> {
        match code {
            50      => Some(Self::PermanentFailure(code)),
            51      => Some(Self::NotFound),
            52      => Some(Self::Gone),
            53      => Some(Self::ProxyRequestRefused),
            54..=58 => Some(Self::PermanentFailure(code)),
            59      => Some(Self::BadRequest),          
            _       => None,
        }
    }
}
impl ClientCertificateRequired {
    pub fn new(code: i16) -> Option<Self> {
        match code {
            60      => Some(Self::ClientCertificateRequired(code)),
            61      => Some(Self::TransientCertificateRequired),
            62      => Some(Self::AuthorizedCertificateRequired),
            63      => Some(Self::CertificateNotAccepted),
            64      => Some(Self::FutureCertificateRejected),          
            65      => Some(Self::ExpiredCertificateRejected),          
            66..=69 => Some(Self::ClientCertificateRequired(code)),
            _       => None,
        }
    }
}
impl Status {
    pub fn new(code: i16, meta: String) -> Result<Self, String> {
        if let Some(status) = Input::new(code) {
            return Ok(Status::InputExpected(status, meta))
        } 
        else if let Some(status) = Success::new(code) {
            return Ok(Status::Success(status, meta))
        } 
        else if let Some(status) = Redirect::new(code) {
            return Ok(Status::Redirect(status, meta))
        }
        else if let Some(status) = TemporaryFailure::new(code) {
            return Ok(Status::TemporaryFailure(status, meta))
        }
        if let Some(status) = PermanentFailure::new(code) {
            return Ok(Status::PermanentFailure(status, meta))
        }
        else if let Some(status) = ClientCertificateRequired::new(code) {
            return Ok(Status::ClientCertificateRequired(status, meta))
        }
        Err(format!("this =>>> ({}) ? ... is not meth", code))
    }
} 
impl FromStr for Status {
    type Err = String;
    fn from_str(line: &str) -> Result<Status, Self::Err> {
        // get regex
        let Ok(regex) = Regex::new(constants::STATUS_REGEX)
            else {return Err("".to_string())};
        // get captures
        let Some(captures) = regex.captures(&line) 
            else {return Err("".to_string())};
        // get code from captures
        let Ok(code) = captures
            .get(1)
            .map_or("", |m| m.as_str())
            .parse()
            else {return Err("".to_string())};
        // get meta from captures
        let meta = captures
            .get(2)
            .map_or("", |m| m.as_str())
            .to_string();
        // return Result
        let status = Status::new(code, meta)
            .or_else(|e| Err(e))?;
        Ok(status)
    }
}
