use k8s_openapi::api::core::v1::{ConfigMapKeySelector, EnvVar, EnvVarSource, SecretKeySelector};

#[allow(unused)]
pub fn env_var_from_secret(var_name: &str, secret: &str, secret_key: &str) -> EnvVar {
    EnvVar {
        name: String::from(var_name),
        value_from: Some(EnvVarSource {
            secret_key_ref: Some(SecretKeySelector {
                name: Some(String::from(secret)),
                key: String::from(secret_key),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[allow(unused)]
pub fn env_var_from_config(var_name: &str, config: &str, config_key: &str) -> EnvVar {
    EnvVar {
        name: String::from(var_name),
        value_from: Some(EnvVarSource {
            config_map_key_ref: Some(ConfigMapKeySelector {
                name: Some(String::from(config)),
                key: String::from(config_key),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}
