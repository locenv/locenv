use std::borrow::Cow;
use std::str::FromStr;

pub enum Header<'c> {
    AcceptRanges,
    AccessControlAllowOrigin,
    AccessControlExposeHeaders,
    Age,
    CacheControl,
    Connection,
    ContentDisposition,
    ContentLength,
    ContentMd5,
    ContentSecurityPolicy,
    ContentType,
    Date,
    ETag,
    FastlyRestarts,
    LastModified,
    Location,
    ReferrerPolicy,
    Server,
    StrictTransportSecurity,
    TransferEncoding,
    Vary,
    Via,
    Custom(Cow<'c, str>),
}

pub enum HeaderError {
    Invalid,
}

// Header

impl<'c> FromStr for Header<'c> {
    type Err = HeaderError;

    fn from_str(s: &str) -> Result<Self, HeaderError> {
        let v: Header<'c> = match s.to_lowercase().as_str() {
            "accept-ranges" => Header::AcceptRanges,
            "access-control-allow-origin" => Header::AccessControlAllowOrigin,
            "access-control-expose-headers" => Header::AccessControlExposeHeaders,
            "age" => Header::Age,
            "cache-control" => Header::CacheControl,
            "connection" => Header::Connection,
            "content-disposition" => Header::ContentDisposition,
            "content-length" => Header::ContentLength,
            "content-md5" => Header::ContentMd5,
            "content-security-policy" => Header::ContentSecurityPolicy,
            "content-type" => Header::ContentType,
            "date" => Header::Date,
            "etag" => Header::ETag,
            "fastly-restarts" => Header::FastlyRestarts,
            "last-modified" => Header::LastModified,
            "location" => Header::Location,
            "referrer-policy" => Header::ReferrerPolicy,
            "server" => Header::Server,
            "strict-transport-security" => Header::StrictTransportSecurity,
            "transfer-encoding" => Header::TransferEncoding,
            "vary" => Header::Vary,
            "via" => Header::Via,
            v => {
                if v.starts_with("x-") {
                    Header::Custom(Cow::Owned(s.into()))
                } else {
                    return Err(HeaderError::Invalid);
                }
            }
        };

        Ok(v)
    }
}
