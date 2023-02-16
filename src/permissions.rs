use url::Url;

pub enum Permission {
    All,
    OutboundUnrestricted,
    Outbound(Vec<Url>),
}
