use std::{collections::HashMap, env};

use inquire::Text;
use reqwest::{
    header::{
        HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, COOKIE, ORIGIN, REFERER,
        SET_COOKIE, USER_AGENT,
    },
    Client,
};
use sha1::{Digest, Sha1};
use tl::{parse, ParserOptions};

pub struct Login;

impl Login {
    pub async fn login() -> Result<(), anyhow::Error> {
        let username = Text::new("What is your name?").prompt()?;
        let password = Text::new("What is your password?").prompt()?;

        let res_token = Self::get_request_verification_token().await?;
        env::set_var("ENTAB_REQUEST_VERIFICATION_TOKEN", &res_token);

        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());
        let hash = hasher.finalize();
        let hash = hex::encode(hash);
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
        println!("{}", export_command);

        Ok(())
    }

    pub async fn get_request_verification_token() -> Result<String, anyhow::Error> {
        let client = reqwest::Client::builder().build()?;

        // Headers to mimic a real browser
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

        // Step 1: Fetch the login page
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
