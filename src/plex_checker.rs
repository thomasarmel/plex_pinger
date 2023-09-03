use crate::config::Config;
use dns_lookup::lookup_host;

pub(crate) struct PlexChecker {
    base_url: String,
    plex_token: String,
    libraries_to_check: Vec<String>,
}

impl PlexChecker {
    const PLEX_TOKEN_PARAM: &'static str = "X-Plex-Token";
    const LIBRARIES_LIST_PATH: &'static str = "library/sections/";
    pub(crate) fn new(config: &Config) -> Self {
        Self {
            base_url: Self::get_base_plex_url(config),
            plex_token: config.plex.plex_token.clone(),
            libraries_to_check: config.plex.libraries.clone(),
        }
    }

    pub(crate) async fn check_plex_up(&self) -> bool {
        let check_base_index_result = self.check_base_index_status();
        let check_libraries_enum_result = self.check_libraries_enum_status();

        if !check_base_index_result.await {
            return false;
        }

        if !check_libraries_enum_result.await {
            return false;
        }

        return true;
    }

    /// Check that you can connect to the base Plex URL
    async fn check_base_index_status(&self) -> bool {
        let plex_index_resp = match reqwest::Client::new()
            .get(format!(
                "{}?{}={}",
                self.base_url,
                Self::PLEX_TOKEN_PARAM,
                self.plex_token
            ))
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Error connecting to Plex: {}", e);
                return false;
            }
        };
        if plex_index_resp.status() != 200 {
            eprintln!(
                "Authorization error when connecting to Plex: {}",
                plex_index_resp.status()
            );
            return false;
        }

        return true;
    }

    /// Check that you can list all element from Plex libraries
    async fn check_libraries_enum_status(&self) -> bool {
        let library_ids = match self.list_libraries_ids().await {
            Ok(ids) => ids,
            Err(_) => return false,
        };

        for library_id in library_ids {
            let plex_lib_details_resp = match reqwest::Client::new()
                .get(format!(
                    "{}{}{}/all?{}={}",
                    self.base_url,
                    Self::LIBRARIES_LIST_PATH,
                    library_id,
                    Self::PLEX_TOKEN_PARAM,
                    self.plex_token
                ))
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Error connecting to Plex: {}", e);
                    return false;
                }
            };
            if plex_lib_details_resp.status() != 200 {
                eprintln!(
                    "Authorization error when connecting to Plex: {}",
                    plex_lib_details_resp.status()
                );
                return false;
            }
        }
        true
    }

    async fn list_libraries_ids(&self) -> Result<Vec<u64>, ()> {
        const DIRECTORY_XML_TAG_NAME: &'static str = "Directory";
        const XML_KEY_ATTRIBUTE: &'static str = "key";
        const XML_TITLE_ATTRIBUTE: &'static str = "title";

        let plex_libs_list_resp = match reqwest::Client::new()
            .get(format!(
                "{}{}?{}={}",
                self.base_url,
                Self::LIBRARIES_LIST_PATH,
                Self::PLEX_TOKEN_PARAM,
                self.plex_token
            ))
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Error connecting to Plex: {}", e);
                return Err(());
            }
        };
        if plex_libs_list_resp.status() != 200 {
            eprintln!(
                "Authorization error when connecting to Plex: {}",
                plex_libs_list_resp.status()
            );
            return Err(());
        }

        let http_text_resp = match plex_libs_list_resp.text().await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Error reading Plex response: {}", e);
                return Err(());
            }
        };

        let mut return_directories_ids: Vec<u64> = Vec::new();

        let doc = roxmltree::Document::parse(http_text_resp.as_str()).map_err(|_| ())?;
        for xmltoken in doc.descendants() {
            if xmltoken.tag_name().name() == DIRECTORY_XML_TAG_NAME {
                if xmltoken.attribute(XML_KEY_ATTRIBUTE).is_none()
                    || xmltoken.attribute(XML_TITLE_ATTRIBUTE).is_none()
                {
                    eprintln!("Error: Directory tag has no key or title attribute");
                    return Err(());
                }
                let key_attribute = xmltoken.attribute(XML_KEY_ATTRIBUTE).unwrap();
                let title_attribute = xmltoken.attribute(XML_TITLE_ATTRIBUTE).unwrap();
                if self
                    .libraries_to_check
                    .contains(&title_attribute.to_string())
                {
                    return_directories_ids.push(key_attribute.parse::<u64>().map_err(|_| ())?);
                }
            }
        }

        Ok(return_directories_ids)
    }

    fn get_base_plex_url(config: &Config) -> String {
        const PLEX_DIRECT_TRAIL_DOMAIN: &'static str = "plex.direct";
        const HTTPS_PREFIX: &'static str = "https://";
        const HTTP_PREFIX: &'static str = "http://";
        format!(
            "{}{}.{}.{}:{}/",
            if config.plex.ssl {
                HTTPS_PREFIX
            } else {
                HTTP_PREFIX
            },
            Self::get_ip_from_domain(&config.plex.domain),
            config.plex.certificate_uuid,
            PLEX_DIRECT_TRAIL_DOMAIN,
            config.plex.port
        )
    }

    fn get_ip_from_domain(domain: &str) -> String {
        let ips = match lookup_host(domain) {
            Ok(ips) => ips,
            Err(e) => {
                eprintln!("Error looking up domain: {}", e);
                std::process::exit(2);
            }
        };
        if ips.len() < 1 {
            eprintln!("Error: {} resolves to no IPs", domain);
            std::process::exit(2);
        }
        ips[0].to_string().replace(".", "-")
    }
}
