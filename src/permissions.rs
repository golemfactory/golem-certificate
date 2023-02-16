use url::Url;

pub mod validator;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Permission {
    All,
    OutboundUnrestricted,
    Outbound(Vec<Url>),
}
