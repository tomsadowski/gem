// gemstatus

// *** BEGIN IMPORTS ***
use regex::Regex;
use std::str::FromStr;
use crate::{
    constants,
};
// *** END IMPORTS ***


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
impl Status {
    pub fn new(code: i16, meta: String) -> Result<Self, String> {

        match code {

            // Input Expected
            10      => 
                Ok(Self::InputExpected(Input::Input(code), meta)),
            11      => 
                Ok(Self::InputExpected(Input::Sensitive, meta)),
            12..=19 => 
                Ok(Self::InputExpected(Input::Input(code), meta)),

            // Success
            20..=29 => 
                Ok(Self::Success(Success::Success(code), meta)),
            30      => 
                Ok(Self::Redirect(Redirect::Temporary(code), meta)),
            31      => 
                Ok(Self::Redirect(Redirect::Permanent, meta)),
            32..=39 => 
                Ok(Self::Redirect(Redirect::Temporary(code), meta)),

            // TemporaryFailure
            40      => 
                Ok(Self::TemporaryFailure(TemporaryFailure::TemporaryFailure(code), meta)),
            41      => 
                Ok(Self::TemporaryFailure(TemporaryFailure::ServerUnavailable, meta)),
            42      => 
                Ok(Self::TemporaryFailure(TemporaryFailure::CGIError, meta)),
            43      => 
                Ok(Self::TemporaryFailure(TemporaryFailure::ProxyError, meta)),
            44      => 
                Ok(Self::TemporaryFailure(TemporaryFailure::SlowDown, meta)),          
            45..=49 => 
                Ok(Self::TemporaryFailure(TemporaryFailure::TemporaryFailure(code), meta)),

            // Permanent Failure
            50      => 
                Ok(Self::PermanentFailure(PermanentFailure::PermanentFailure(code), meta)),
            51      => 
                Ok(Self::PermanentFailure(PermanentFailure::NotFound, meta)),
            52      => 
                Ok(Self::PermanentFailure(PermanentFailure::Gone, meta)),
            53      => 
                Ok(Self::PermanentFailure(PermanentFailure::ProxyRequestRefused, meta)),
            54..=58 => 
                Ok(Self::PermanentFailure(PermanentFailure::PermanentFailure(code), meta)),
            59      => 
                Ok(Self::PermanentFailure(PermanentFailure::BadRequest, meta)),

            // Client Certificate Required
            60      => 
                Ok(Self::ClientCertificateRequired(
                    ClientCertificateRequired::ClientCertificateRequired(code), meta)),
            61      => 
                Ok(Self::ClientCertificateRequired(
                    ClientCertificateRequired::TransientCertificateRequired, meta)),
            62      => 
                Ok(Self::ClientCertificateRequired(
                    ClientCertificateRequired::AuthorizedCertificateRequired, meta)),
            63      => 
                Ok(Self::ClientCertificateRequired(
                    ClientCertificateRequired::CertificateNotAccepted, meta)),
            64      => 
                Ok(Self::ClientCertificateRequired(
                    ClientCertificateRequired::FutureCertificateRejected, meta)),
            65      => 
                Ok(Self::ClientCertificateRequired(
                    ClientCertificateRequired::ExpiredCertificateRejected, meta)),          
            66..=69 => 
                Ok(Self::ClientCertificateRequired(
                    ClientCertificateRequired::ClientCertificateRequired(code), meta)),

            _ => Err(format!("this =>>> ({}) ? ... is not meth", code)),
        }
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
        Status::new(code, meta)
    }
}
