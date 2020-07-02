use super::{Error, Result, Session};

// TODO: auth_id はログインページでJSにより動的に追加されるため、ヘッドレスブラウザを使わないと取得が困難
// auth_id があれば user_session が Cookie に含まれるものと見られる

impl Session {
	pub async fn signin(email: &str, password: &str) -> Result<Self> {
		let client = reqwest::Client::new();

		let res = client
			.get("https://account.nicovideo.jp/login")
			.send()
			.await?
			.error_for_status()?
			.text()
			.await?;
		let doc = select::document::Document::from(&res as &str);

		let auth_id = doc
			.find(select::predicate::Attr("name", "auth_id"))
			.next()
			.and_then(|n| n.attr("value"))
			.ok_or(Error::InvalidSignInPage)?; // ここでエラー発生

		let res = client
			.post("https://account.nicovideo.jp/login/redirector")
			.form(&[
				("mail_tel", email),
				("password", password),
				("auth_id", auth_id),
			])
			.send()
			.await?
			.error_for_status()?;

		let cookie = res
			.headers()
			.get(reqwest::header::SET_COOKIE)
			.map(|v| v.to_str().unwrap_or(""))
			.unwrap_or("");

		Ok(Self {
			cookie: cookie.to_string(),
			client,
		})
	}
}
