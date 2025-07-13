use home::home_dir;
use inquire::Password;
use inquire::Text;
use reqwest::{
    header::{
        HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, COOKIE, ORIGIN, REFERER,
        SET_COOKIE, USER_AGENT,
    },
    Client,
};
use sha1::{Digest, Sha1};
use std::io::Read;
use std::io::Write;
use std::{collections::HashMap, env};
use tl::{parse, ParserOptions};

pub struct Login;

lazy_static::lazy_static! {
     static ref DATA_DIR: String = format!("{}/.entab", home_dir().unwrap().display());
}

impl Login {
    pub fn store_credentials(username: &str, password: &str) {
        std::fs::create_dir_all(DATA_DIR.clone()).unwrap();
        let mut file = std::fs::File::create(format!("{}/credentials", DATA_DIR.clone())).unwrap();
        file.write_all(format!("{}:{}", username, password).as_bytes())
            .unwrap();
    }
    pub fn fetch_credentials() -> Result<(String, String), anyhow::Error> {
        let mut file = std::fs::File::open(format!("{}/credentials", DATA_DIR.clone()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut split = contents.split(':');
        let username = split.next().unwrap();
        let password = split.next().unwrap();
        Ok((username.to_string(), password.to_string()))
    }
    pub async fn login(
        store_credentials: bool,
        fetch_credentials: bool,
    ) -> Result<(), anyhow::Error> {
        let res_token = Self::get_request_verification_token().await?;
        env::set_var("ENTAB_REQUEST_VERIFICATION_TOKEN", &res_token);
        #[allow(unused_assignments)]
        let mut username = String::new();
        #[allow(unused_assignments)]
        let mut hash = String::new();
        if fetch_credentials {
            match Self::fetch_credentials() {
                Ok((name, pwd)) => {
                    username = name;
                    hash = pwd;
                }
                Err(_) => {
                    username = Text::new("Username").prompt()?;
                    let password = Password::new("Password").without_confirmation().prompt()?;
                    let mut hasher = Sha1::new();
                    hasher.update(password.as_bytes());
                    let hashed = hasher.finalize();
                    hash = hex::encode(hashed);
                    if store_credentials {
                        Self::store_credentials(&username, &hash);
                    }
                }
            }
        } else {
            username = Text::new("Username").prompt()?;
            let password = Password::new("Password").without_confirmation().prompt()?;
            let mut hasher = Sha1::new();
            hasher.update(password.as_bytes());
            let hashed = hasher.finalize();
            hash = hex::encode(hashed);
            if store_credentials {
                Self::store_credentials(&username, &hash);
            }
        }

        let client = Client::new();

        // Set up headers
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-GB,en-US;q=0.9,en;q=0.8"),
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
        );
        headers.insert(
            ORIGIN,
            HeaderValue::from_static("https://www.lviscampuscare.org"),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str(format!("__RequestVerificationToken={}", res_token).as_str())?,
        );
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://www.lviscampuscare.org/Logon/Logon"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Mobile Safari/537.36"));
        headers.insert(
            "x-requested-with",
            HeaderValue::from_static("XMLHttpRequest"),
        );
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static(r#""Chromium";v="135", "Not-A.Brand";v="8""#),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?1"));
        headers.insert(
            "sec-ch-ua-platform",
            HeaderValue::from_static(r#""Android""#),
        );
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
        headers.insert("priority", HeaderValue::from_static("u=1, i"));

        // Raw body (URL-encoded)
        let mut form_data = HashMap::new();
        form_data.insert("log[UserName]", username.as_str());
        form_data.insert("log[UserPassword]", hash.as_str());
        form_data.insert("log[UserTypeID]", "3");

        // Send POST request
        let res = client
            .post("https://www.lviscampuscare.org/Logon/Logon")
            .headers(headers)
            .form(&form_data)
            .send()
            .await?;

        let headers = res.headers();
        let cookies = headers.get_all(SET_COOKIE).iter().collect::<Vec<_>>();
        let cookies = cookies
            .iter()
            .map(|cookie| {
                let cookie = cookie.to_str().unwrap();
                let name = cookie.split("=").collect::<Vec<_>>()[0];
                if name == "ASP.NET_SessionId" {
                    let value = cookie.split("=").collect::<Vec<_>>()[1]
                        .split(';')
                        .next()
                        .unwrap();
                    ("ENTAB_SESSION_ID", value)
                } else {
                    let value = cookie.split("=").collect::<Vec<_>>()[1]
                        .split(';')
                        .next()
                        .unwrap();
                    ("ENTAB_ASPXAUTH", value)
                }
            })
            .collect::<Vec<_>>();
        for (name, value) in &cookies {
            env::set_var(name, value);
        }
        let mut export_command = String::from("export ");
        cookies.iter().for_each(|(name, value)| {
            env::set_var(name, value);
            export_command.push_str(name);
            export_command.push('=');
            export_command.push_str(value);
            export_command.push(' ');
        });
        export_command.push_str("ENTAB_REQUEST_VERIFICATION_TOKEN");
        export_command.push('=');
        export_command.push_str(&res_token);

        Ok(())
    }

    pub async fn get_request_verification_token() -> Result<String, anyhow::Error> {
        let client = reqwest::Client::builder().build()?;

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-GB,en-US;q=0.9,en;q=0.8"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Mobile Safari/537.36"));
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://www.lviscampuscare.org/"),
        );

        let res = client
            .get("https://www.lviscampuscare.org/Logon/Logon")
            .headers(headers.clone())
            .send()
            .await?;

        let body = res.text().await?;

        let parsed_table = parse(&body, ParserOptions::default())?;
        let mut token = String::new();
        parsed_table.nodes().iter().for_each(|row| {
            let tag = row.as_tag();
            if tag.is_some() && tag.unwrap().name() == "input" {
                let attributes = tag.unwrap().attributes();
                if let Some(name) = attributes.get("name") {
                    if name.unwrap().as_utf8_str() == "__RequestVerificationToken" {
                        let value = attributes
                            .get("value")
                            .unwrap()
                            .unwrap()
                            .as_utf8_str()
                            .to_string();
                        token = value.to_string();
                    }
                }
            }
        });
        Ok(token)
    }
}
