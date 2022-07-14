use super::*;

#[derive(Debug, PartialEq)]
pub struct Scheme {
    pub name: String,
    pub file_name: Option<String>,
    pub data: ColorSchemeFile,
}

impl Scheme {
    pub fn to_toml_value(&self) -> anyhow::Result<toml::Value> {
        let value = self.data.to_dynamic();
        Ok(dynamic_to_toml(value)?)
    }

    pub fn to_toml(&self) -> anyhow::Result<String> {
        let value = self.to_toml_value()?;
        Ok(toml::ser::to_string_pretty(&value)?)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        let mut value = self.to_toml_value()?;
        let (prefix, _) = make_prefix(&self.name);
        match &mut value {
            toml::Value::Table(map) => {
                let meta = map.get_mut("metadata").unwrap();
                match meta {
                    toml::Value::Table(meta) => {
                        meta.insert(
                            "prefix".to_string(),
                            toml::Value::String(prefix.to_string()),
                        );
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }

        Ok(serde_json::to_string_pretty(&value)?)
    }

    pub fn to_json_value(&self) -> anyhow::Result<serde_json::Value> {
        let json = self.to_json()?;
        Ok(serde_json::from_str(&json)?)
    }
}

fn dynamic_to_toml(value: Value) -> anyhow::Result<toml::Value> {
    Ok(match value {
        Value::Null => anyhow::bail!("cannot map Null to toml"),
        Value::Bool(b) => toml::Value::Boolean(b),
        Value::String(s) => toml::Value::String(s),
        Value::Array(a) => {
            let mut arr = vec![];
            for v in a {
                arr.push(dynamic_to_toml(v)?);
            }
            toml::Value::Array(arr)
        }
        Value::Object(o) => {
            let mut map = toml::value::Map::new();
            for (k, v) in o {
                let k = match k {
                    Value::String(s) => s,
                    _ => anyhow::bail!("toml keys must be strings {k:?}"),
                };
                let v = match v {
                    Value::Null => continue,
                    other => dynamic_to_toml(other)?,
                };
                map.insert(k, v);
            }
            toml::Value::Table(map)
        }
        Value::U64(i) => toml::Value::Integer(i.try_into()?),
        Value::I64(i) => toml::Value::Integer(i.try_into()?),
        Value::F64(f) => toml::Value::Float(*f),
    })
}
