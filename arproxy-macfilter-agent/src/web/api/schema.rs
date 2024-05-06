use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

fn validate_mac_address(text: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^([0-9A-Fa-f]{2}[:]){5}([0-9A-Fa-f]{2})$").unwrap();
    if re.is_match(&text) {
        Ok(())
    } else {
        Err(ValidationError::new(&"Invalid MAC address format"))
    }
}

#[derive(Debug, Validate, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(super) struct AllowedMacSchema {
    #[validate(custom(function = "validate_mac_address"))]
    pub mac_address: String,
}

pub(super) type AllowedMacPostResponseSchema = AllowedMacSchema;
pub(super) type AllowedMacPostSchema = AllowedMacSchema;
pub(super) type AllowedMacDeleteSchema = AllowedMacSchema;

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use pnet::util::MacAddr;
    use validator::Validate;

    use super::AllowedMacSchema;

    #[test]
    fn validate_good_input() {
        let mut samples = Vec::new();
        samples.push(AllowedMacSchema {
            mac_address: "02:ab:cd:ef:09:00".to_string(),
        });
        samples.push(AllowedMacSchema {
            mac_address: "02:AB:CD:EF:09:00".to_string(),
        });
        samples.push(AllowedMacSchema {
            mac_address: "02:Ab:cD:Ef:09:00".to_string(),
        });
        for sample in samples.iter() {
            sample.validate().expect("Validation Error");
        }
        // validationに成功するものがパース可能であることの確認
        for sample in samples.iter() {
            MacAddr::from_str(&sample.mac_address).expect("Parse Error");
        }
    }

    #[test]
    fn validate_bad_input() {
        let mut samples = Vec::new();
        samples.push(AllowedMacSchema {
            mac_address: "02-ab-cd-ef-09-00".to_string(),
        });
        samples.push(AllowedMacSchema {
            mac_address: "02-ab:cd-ef:09-00".to_string(),
        });
        samples.push(AllowedMacSchema {
            mac_address: "02:Ab:cD:Ef:09:00:01:13".to_string(),
        });
        samples.push(AllowedMacSchema {
            mac_address: "hello, world".to_string(),
        });
        for sample in samples.iter() {
            sample
                .validate()
                .expect_err("Vaildation for bad input was suceed");
        }
    }
}
