use http_body_util::Full;
use hyper::{body::Bytes, Response};
use serde::{Deserialize, Serialize};

use crate::{Db, GenericError, Req};

#[derive(Serialize, Deserialize)]
pub struct User {
    username: String,
    passwrod: String
}

pub async fn create_user(req: Req, db: Db) -> Result<Response<Full<Bytes>>, GenericError>{
    
    Ok(Response::new(Full::new(Bytes::from("user created!"))))
}

pub async fn get_user(req: Req) -> Result<Response<Full<Bytes>>, GenericError> {
    Ok(Response::new(Full::new(Bytes::from("get user!"))))
}

pub async fn delete_user(req: Req) -> Result<Response<Full<Bytes>>, GenericError> {
    Ok(Response::new(Full::new(Bytes::from("delete user!"))))
}

pub async fn modify_user(req: Req) -> Result<Response<Full<Bytes>>, GenericError> {
    Ok(Response::new(Full::new(Bytes::from("modify  user!"))))
}
