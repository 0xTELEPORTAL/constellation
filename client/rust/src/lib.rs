// TODO[SER-338] uncomment me // #![deny(improper_ctypes_definitions, improper_ctypes)]
#![allow(clippy::diverging_sub_expression)]
#![feature(generic_associated_types)]
#![feature(trivial_bounds)]

pub mod action;
pub mod baseline;
pub mod contract;
pub mod engine;
pub mod link;
pub mod object;
pub mod realm;
pub mod time;

use eyre::{eyre, Result};
use link::{Link, RealmServerHandle, Session};
use realm::{Realm, RealmID};

pub use engine::Engine;

#[non_exhaustive]
pub enum Endpoint {
    Local(RealmServerHandle),
    Wss(String, u16),
    Tcp(String, u16),
}

// This could just be directly args on RealmClient::new(), but we expect building will
// be more complicated in the future.
pub struct RealmClientBuilder {
    endpoint: Endpoint,
    realm_id: String,
}
impl RealmClientBuilder {
    pub fn create(endpoint: Endpoint, realm_id: String) -> Self {
        Self { endpoint, realm_id }
    }

    pub fn build(self) -> Result<RealmClient> {
        RealmClient::new(self)
    }
}

/// Represents an API client for the realm.
#[allow(unused)]
pub struct RealmClient {
    session: Session,
    link: Link,
    local_realm: Realm,
}
impl RealmClient {
    fn new(builder: RealmClientBuilder) -> Result<Self> {
        let local_realm = Realm::new(RealmID::new(builder.realm_id));
        let result = match builder.endpoint {
            Endpoint::Local(handle) => {
                let link = Link::new(handle);
                let session = link.create_session();
                Self {
                    link,
                    session,
                    local_realm,
                }
            }
            _ => return Err(eyre!("We only support local endpoints!")),
        };
        Ok(result)
    }
}

// This generates the C header file for the bindings. See safer-ffi's guide.
#[safer_ffi::cfg_headers]
#[test]
fn generate_headers() -> ::std::io::Result<()> {
    let builder = ::safer_ffi::headers::builder();
    if ::std::env::var("HEADERS_TO_STDOUT")
        .ok()
        .map_or(false, |it| it == "1")
    {
        builder.to_writer(::std::io::stdout()).generate()
    } else {
        builder.to_file(&"generated.h".to_string())?.generate()
    }
}