use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};

static DATA_API_URL: &'static str = "https://data.biconomy.io/api/v1/dapp";

pub enum API {
    Data(Data),
}

pub enum Data {
    UniqueUser,
    UserLimits,
    GasTankBalance,
}

impl From<API> for String {
    fn from(item: API) -> Self {
        String::from(match item {
            API::Data(route) => match route {
                Data::UniqueUser => format!("{}/uniqueUserData", DATA_API_URL),
                Data::UserLimits => format!("{}/user-limt", DATA_API_URL),
                Data::GasTankBalance => format!("{}/gas-tank-balance", DATA_API_URL),
            },
        })
    }
}

pub struct BiconomyClient {
    http_client: Client,
    auth_token: String,
}

impl BiconomyClient {
    pub fn new(auth_token: String) -> Self {
        Self {
            http_client: Client::new(),
            auth_token,
        }
    }

    pub async fn gas_tank_balance(&self, api_key: String) -> String {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_bytes(b"authToken").unwrap(),
            HeaderValue::from_str(self.auth_token.as_str()).unwrap(),
        );
        headers.insert(
            HeaderName::from_bytes(b"apiKey").unwrap(),
            HeaderValue::from_str(api_key.as_str()).unwrap(),
        );
        let res = self
            .http_client
            .get(String::from(API::Data(Data::GasTankBalance)))
            .headers(headers)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        res
    }
}
