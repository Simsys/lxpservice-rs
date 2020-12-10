/// LxpApi - a general usable crate to access the LetterXpress Web API
///
/// LetterXpres (https://www.letterxpress.de/) offers a service using a Web
/// API to make printing services easy to use. PDF documents can be
/// transferred to be printed and sent by Letterxpress. This is not only
/// convenient, but also very inexpensive.
///
/// The Crate LxpApi encapsulates all the software routines necessary to use
/// Letterxpress' Web API. This Crate should be usable in any application.
///
/// The error handling is done in a way that outside of this crate it can be
///  decided how to handle errors. For logging, LxpApi uses the Crate log
/// (https://github.com/rust-lang/log), which allows flexible use and
/// integration in any app.
extern crate base64;
extern crate md5;
extern crate reqwest;
extern crate serde_json;

use crate::lxptypes::*;

use log::*;
use std::fmt;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct LxpApi {
    url: String,
    auth: SubNameAndKey,
    client: reqwest::Client,
}

pub enum LxpApiError {
    RestError,
    JsonError,
}

impl fmt::Display for LxpApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LxpApiError::RestError => write!(f, "Web service: check url, user and apikey "), // user-facing output
            LxpApiError::JsonError => write!(f, "Internal JSON error, please the developer"), // user-facing output
        }
    }
}

impl fmt::Debug for LxpApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!()) // programmer-facing output
    }
}

impl LxpApi {
    pub fn new(user_name: &str, api_key: &str, url: &str) -> LxpApi {
        let auth = SubNameAndKey {
            username: user_name.into(),
            apikey: api_key.into(),
        };
        let client = reqwest::Client::new();
        LxpApi {
            url: url.into(),
            auth: auth,
            client: client,
        }
    }

    pub async fn delete_job(&self, id: i32) -> Result<Response, LxpApiError> {
        let sub_url = format!("deleteJob/{}", id);
        let mut body = RequestLetter::default();
        body.auth = self.auth.clone();
        self._request(RequestType::Delete, &sub_url, &body).await
    }

    pub async fn get_blance(&self) -> Result<Response, LxpApiError> {
        let mut body = RequestLetter::default();
        body.auth = self.auth.clone();
        self._request(RequestType::Get, "getBalance", &body).await
    }

    pub async fn get_jobs_hold(&self) -> Result<Response, LxpApiError> {
        let mut body = RequestLetter::default();
        body.auth = self.auth.clone();
        self._request(RequestType::Get, "getJobs/hold", &body).await
    }

    pub async fn get_jobs_queue(&self, days: i32) -> Result<Response, LxpApiError> {
        let mut body = RequestLetter::default();
        body.auth = self.auth.clone();
        let sub_url = format!("getJobs/queue/{}", days);
        self._request(RequestType::Get, &sub_url, &body).await
    }

    pub async fn get_jobs_sent(&self, days: i32) -> Result<Response, LxpApiError> {
        let mut body = RequestLetter::default();
        body.auth = self.auth.clone();
        let sub_url = format!("getJobs/sent/{}", days);
        self._request(RequestType::Get, &sub_url, &body).await
    }

    pub async fn list_invoices(&self) -> Result<Response, LxpApiError> {
        let mut body = RequestLetter::default();
        body.auth = self.auth.clone();
        self._request(RequestType::Get, "listInvoices", &body).await
    }

    pub async fn get_last_invoice(&self) -> Result<(Response, Vec<u8>), LxpApiError> {
        let mut body = RequestLetter::default();
        body.auth = self.auth.clone();
        let r: Response = self._request(RequestType::Get, "getInvoice", &body).await?;
        match &r.invoice {
            Some(invoice) => {
                let pdf_base64_data = invoice.pdf_data.clone().unwrap();
                let pdf_data = base64::decode(pdf_base64_data).unwrap();
                return Ok((r, pdf_data));
            }
            None => return Ok((r, Vec::new())),
        }
    }

    pub async fn get_invoice(&self, id: i32) -> Result<(Response, Vec<u8>), LxpApiError> {
        let mut body = RequestLetter::default();
        body.auth = self.auth.clone();
        let sub_url = format!("getInvoice/{}", id);
        let r: Response = self._request(RequestType::Get, &sub_url, &body).await?;
        match &r.invoice {
            Some(invoice) => {
                let pdf_base64_data = invoice.pdf_data.clone().unwrap();
                let pdf_data = base64::decode(pdf_base64_data).unwrap();
                return Ok((r, pdf_data));
            }
            None => return Ok((r, Vec::new())),
        }
    }

    pub async fn set_job(
        &self,
        file_name: &str,
        color: &ColorPrint,
        mode: &Mode,
        ship: &Ship,
    ) -> Result<Response, LxpApiError> {
        let mut letter = SubLetterData::default();
        match color {
            ColorPrint::Color => letter.specification.color = 4,
            ColorPrint::BlackAndWhite => letter.specification.color = 1,
        }
        match mode {
            Mode::Simplex => letter.specification.mode = "simplex".into(),
            Mode::Duplex => letter.specification.mode = "duplex".into(),
        }
        match ship {
            Ship::International => letter.specification.ship = "international".into(),
            Ship::National => letter.specification.ship = "national".into(),
        }
        let path = std::path::Path::new(&file_name);
        let mut pdf_file = match std::fs::File::open(&path) {
            Err(why) => panic!("couldn't open {}", why),
            Ok(file) => file,
        };
        letter.address = path.file_name().unwrap().to_str().unwrap().to_string();

        let mut pdf_content = Vec::new();
        match pdf_file.read_to_end(&mut pdf_content) {
            Err(why) => error!("Couldn't read PDF file {}", why),
            Ok(_c) => (),
        };

        letter.base64_file = base64::encode(pdf_content);
        letter.base64_checksum = format!("{:x}", md5::compute(&letter.base64_file));

        let body = RequestLetter {
            auth: self.auth.clone(),
            letter: letter,
        };

        let sub_url = format!("setJob");
        self._request(RequestType::Post, &sub_url, &body).await
    }

    async fn _request(
        &self,
        request_type: RequestType,
        sub_url: &str,
        body: &RequestLetter,
    ) -> Result<Response, LxpApiError> {
        let url = self.url.clone() + sub_url;
        trace!("Url {}", &url);

        let r1 = match request_type {
            RequestType::Delete => {
                let r1 = self.client.delete(&url).json(body).send().await;
                r1
            }
            RequestType::Get => {
                let r1 = self.client.get(&url).json(body).send().await;
                r1
            }
            RequestType::Post => {
                let r1 = self.client.post(&url).json(body).send().await;
                r1
            }
        };

        let r2 = match r1 {
            Ok(r) => {
                debug!("Response received");
                r
            }
            Err(e) => {
                debug!("Request was: {}", serde_json::to_string(body).unwrap());
                debug!("{}", e);
                return Err(LxpApiError::RestError);
            }
        };

        let json_res = match r2.text().await {
            Ok(r) => r,
            Err(e) => {
                debug!("Request was: {}", serde_json::to_string(body).unwrap());
                debug!("{}", e);
                return Err(LxpApiError::RestError);
            }
        };
        trace!("Respond {}", &json_res);

        match serde_json::from_str::<Response>(&json_res) {
            Ok(r) => return Ok(r),
            Err(e) => {
                debug!("Respond was {}", &json_res);
                debug!("Problem during JSON parsing: {}", e);
                return Err(LxpApiError::JsonError);
            }
        };
    }
}
