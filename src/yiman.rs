use http_body_util::BodyExt;
use hudsucker::{
    Body,
    hyper::{Method, Request, Response, StatusCode, Uri, header},
};

const YIMAN_JS: &str = include_str!("yiman.js");

pub(crate) fn is_page(method: &Method, uri: &Uri) -> bool {
    method == Method::GET
        && uri.host() == Some("game.maj-soul.com")
        && matches!(uri.path(), "/1/" | "/1/index.html")
}

pub(crate) fn prepare_request(req: &mut Request<Body>) {
    let headers = req.headers_mut();
    headers.insert(
        header::ACCEPT_ENCODING,
        header::HeaderValue::from_static("identity"),
    );
    for name in [
        header::IF_NONE_MATCH,
        header::IF_MODIFIED_SINCE,
        header::IF_RANGE,
        header::RANGE,
    ] {
        headers.remove(name);
    }
}

fn inject_html(html: &str) -> Option<String> {
    if html.contains("data-majsoulmax-yiman") || !html.contains("createUnityInstance") {
        return None;
    }
    let at = html.to_ascii_lowercase().find("</head>")?;
    Some(format!(
        "{}<script data-majsoulmax-yiman=\"1\">\n{YIMAN_JS}\n</script>\n{}",
        &html[..at],
        &html[at..],
    ))
}

pub(crate) async fn inject_response(res: Response<Body>) -> Response<Body> {
    let value = |name| res.headers().get(name).and_then(|v| v.to_str().ok());
    let html = value(header::CONTENT_TYPE).is_some_and(|v| {
        v.split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .eq_ignore_ascii_case("text/html")
    });
    if res.status() != StatusCode::OK
        || !html
        || !value(header::CONTENT_ENCODING)
            .unwrap_or("identity")
            .eq_ignore_ascii_case("identity")
    {
        return res;
    }
    let (mut parts, body) = res.into_parts();
    let bytes = match body.collect().await {
        Ok(body) => body.to_bytes(),
        Err(error) => {
            tracing::warn!("读取游戏页面失败：{error}");
            let mut res = Response::new(Body::empty());
            *res.status_mut() = StatusCode::BAD_GATEWAY;
            return res;
        }
    };
    let Some(html) = std::str::from_utf8(&bytes).ok().and_then(inject_html) else {
        return Response::from_parts(parts, Body::from(bytes));
    };
    // The modified document must not reuse the upstream entity's cache metadata.
    for name in ["transfer-encoding", "etag", "last-modified", "content-md5"] {
        parts.headers.remove(name);
    }
    parts.headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    parts.headers.insert(
        header::CONTENT_LENGTH,
        header::HeaderValue::from_str(&html.len().to_string()).expect("decimal content length"),
    );
    tracing::debug!("已注入役满动画脚本");
    Response::from_parts(parts, Body::from(html))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_verified_entry_points_are_selected() {
        for url in [
            "https://game.maj-soul.com/1/",
            "https://game.maj-soul.com/1/index.html?paipu=test",
        ] {
            assert!(is_page(&Method::GET, &url.parse().unwrap()));
        }
        for url in [
            "https://game.maj-soul.com.example/1/",
            "https://other.example/1/",
            "https://game.maj-soul.com/assetbundles/DXT/test.majset",
            "https://game.maj-soul.com/1/version.json",
        ] {
            assert!(!is_page(&Method::GET, &url.parse().unwrap()));
        }
        assert!(!is_page(
            &Method::POST,
            &"https://game.maj-soul.com/1/".parse().unwrap(),
        ));
    }

    #[test]
    fn injection_requests_bypass_conditional_and_partial_responses() {
        let mut req = Request::builder()
            .header(header::ACCEPT_ENCODING, "gzip, br")
            .header(header::IF_NONE_MATCH, "old")
            .header(header::IF_MODIFIED_SINCE, "old")
            .header(header::RANGE, "bytes=0-100")
            .header(header::IF_RANGE, "old")
            .body(Body::empty())
            .unwrap();
        prepare_request(&mut req);
        assert_eq!(req.headers()[header::ACCEPT_ENCODING], "identity");
        for name in [
            header::IF_NONE_MATCH,
            header::IF_MODIFIED_SINCE,
            header::IF_RANGE,
            header::RANGE,
        ] {
            assert!(!req.headers().contains_key(name));
        }
    }

    #[tokio::test]
    async fn rewritten_html_has_fresh_headers_and_only_one_injection() {
        let html = "<html><head></head><body><script>createUnityInstance()</script></body></html>";
        let res = Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CONTENT_LENGTH, html.len().to_string())
            .header(header::ETAG, "original")
            .header(header::CACHE_CONTROL, "max-age=3600")
            .body(Body::from(html))
            .unwrap();
        let res = inject_response(res).await;
        assert_eq!(res.headers()[header::CACHE_CONTROL], "no-store");
        assert!(!res.headers().contains_key(header::ETAG));
        let length = res.headers()[header::CONTENT_LENGTH]
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.len(), length);
        let html = std::str::from_utf8(&body).unwrap();
        assert!(html.contains("LoadMgr"));
        assert_eq!(html.matches("data-majsoulmax-yiman").count(), 1);
        assert!(inject_html(html).is_none());
        assert!(inject_html("<html><head></head><body>Laya</body></html>").is_none());
    }

    #[tokio::test]
    async fn unexpected_encoding_or_content_is_forwarded_unchanged() {
        for (status, content_type, encoding) in [
            (StatusCode::OK, "text/html", "gzip"),
            (StatusCode::OK, "application/octet-stream", "identity"),
            (StatusCode::NOT_FOUND, "text/html", "identity"),
        ] {
            let res = Response::builder()
                .status(status)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CONTENT_ENCODING, encoding)
                .header(header::ETAG, "original")
                .body(Body::from("original body"))
                .unwrap();
            let res = inject_response(res).await;
            assert_eq!(res.status(), status);
            assert_eq!(res.headers()[header::ETAG], "original");
            let bytes = res.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(bytes.as_ref(), b"original body");
        }
    }
}
