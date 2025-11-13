// gemstatus

use regex::Regex;
use std::str::FromStr;
use crate::{
    util::{ParseError},
    constants,
};

#[derive(Debug, Clone)]
pub enum Input {
    Input,                          // 10
    Sensitive,                      // 11
}
#[derive(Debug, Clone)]
pub enum Success {
    Success,                        // 20
}
#[derive(Debug, Clone)]
pub enum Redirect {
    Temporary,                      // 30
    Permanent,                      // 31
}
#[derive(Debug, Clone)]
pub enum TemporaryFailure {
    TemporaryFailure,               // 40
    ServerUnavailable,              // 41
    CGIError,                       // 42
    ProxyError,                     // 43 
    SlowDown,                       // 44
}
#[derive(Debug, Clone)]
pub enum PermanentFailure {
    PermanentFailure,               // 50
    NotFound,                       // 51
    Gone,                           // 52
    ProxyRequestRefused,            // 53
    BadRequest,                     // 59
}
#[derive(Debug, Clone)]
pub enum ClientCertificateRequired {
    ClientCertificateRequired,      // 60
    TransientCertificateRequired,   // 61
    AuthorizedCertificateRequired,  // 62
    CertificateNotAccepted,         // 63
    FutureCertificateRejected,      // 64
    ExpiredCertificateRejected,     // 65
} 
#[derive(Debug, Clone)]
pub enum Status {
    // 10
    InputExpected(Input, String),
    // 20
    Success(Success, String),
    // 30
    Redirect(Redirect, String),
    // 40
    TemporaryFailure(TemporaryFailure, String),
    // 50
    PermanentFailure(PermanentFailure, String),
    // 60
    ClientCertificateRequired(ClientCertificateRequired, String),
    // _
    Unknown(String),
}
impl Status {
    // create status from integer
    pub fn new(code: i16, meta: String) -> Self {
        match code {
            10 => Status::InputExpected(Input::Input, meta),
            11 => Status::InputExpected(Input::Sensitive, meta),

            20 => Status::Success(Success::Success, meta),

            30 => Status::Redirect(Redirect::Temporary, meta),
            31 => Status::Redirect(Redirect::Permanent, meta),

            40 => Status::TemporaryFailure
                (TemporaryFailure::TemporaryFailure, meta),
            41 => Status::TemporaryFailure
                (TemporaryFailure::ServerUnavailable, meta),
            42 => Status::TemporaryFailure
                (TemporaryFailure::CGIError, meta),
            43 => Status::TemporaryFailure
                (TemporaryFailure::ProxyError, meta),
            44 => Status::TemporaryFailure
                (TemporaryFailure::SlowDown, meta),

            50 => Status::PermanentFailure
                (PermanentFailure::PermanentFailure, meta),
            51 => Status::PermanentFailure
                (PermanentFailure::NotFound, meta),
            52 => Status::PermanentFailure
                (PermanentFailure::Gone, meta),
            53 => Status::PermanentFailure
                (PermanentFailure::ProxyRequestRefused, meta),
            59 => Status::PermanentFailure
                (PermanentFailure::BadRequest, meta),

            60 => Status::ClientCertificateRequired
                (ClientCertificateRequired::ClientCertificateRequired, meta),
            61 => Status::ClientCertificateRequired
                (ClientCertificateRequired::TransientCertificateRequired, meta),
            62 => Status::ClientCertificateRequired
                (ClientCertificateRequired::AuthorizedCertificateRequired, meta),
            63 => Status::ClientCertificateRequired
                (ClientCertificateRequired::CertificateNotAccepted, meta),
            64 => Status::ClientCertificateRequired
                (ClientCertificateRequired::FutureCertificateRejected, meta),
            65 => Status::ClientCertificateRequired
                (ClientCertificateRequired::ExpiredCertificateRejected, meta),

            _ => Status::Unknown(meta),
        }
    }
} 
impl FromStr for Status {
    type Err = ParseError;
    fn from_str(line: &str) -> Result<Status, Self::Err> {
        // get regex
        let Ok(regex) = Regex::new(constants::STATUS_REGEX)
            else {return Err(ParseError)};
        // get captures
        let Some(captures) = regex.captures(&line) 
            else {return Err(ParseError)};
        // get code from captures
        let Ok(code) = captures
            .get(1)
            .map_or("", |m| m.as_str())
            .parse()
            else {return Err(ParseError)};
        // get meta from captures
        let meta = captures
            .get(2)
            .map_or("", |m| m.as_str())
            .to_string();
        // return Result
        Ok(Status::new(code, meta))
    }
}
