use serde::{de::Error, Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrOperator {
    #[serde(rename = "$or")]
    or: Vec<ObjMatcher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndOperator {
    #[serde(rename = "$and")]
    and: Vec<ObjMatcher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeOperator {
    #[serde(rename = "$ne")]
    neq: Box<ObjMatcher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObjMatcher {
    Or(OrOperator),
    And(AndOperator),
    Ne(NeOperator),
    Spec(Value),
}

impl TryInto<ObjMatcher> for Value {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<ObjMatcher, serde_json::Error> {
        match self.as_object() {
            Some(map) => {
                if map.contains_key("$or") {
                    return Ok(ObjMatcher::Or(serde_json::from_value(self)?));
                } else if map.contains_key("$ne") {
                    return Ok(ObjMatcher::Ne(serde_json::from_value(self)?));
                } else if map.contains_key("$and") {
                    return Ok(ObjMatcher::And(serde_json::from_value(self)?));
                }
            }
            _ => {}
        }
        return Err(serde_json::Error::custom("not a valid object matcher"));
    }
}

pub fn obj_matches(spec: &ObjMatcher, obj: &Value) -> bool {
    match spec {
        ObjMatcher::Ne(NeOperator { neq }) => {
            let neq = &**neq;
            if obj_matches(neq, obj) {
                return false;
            }

            return true;
        }

        ObjMatcher::Or(OrOperator { or }) => {
            for v in or {
                if obj_matches(v, obj) {
                    return true;
                }
            }

            return false;
        }
        ObjMatcher::And(AndOperator { and }) => {
            for v in and {
                if obj_matches(v, obj) {
                    return false;
                }
            }

            return true;
        }
        ObjMatcher::Spec(v) => {
            if let Ok::<ObjMatcher, _>(_o) = v.clone().try_into() {
                return false;
            }
            match v {
                Value::Null => true,
                Value::Bool(b) => obj.is_boolean() && obj.as_bool().unwrap() == *b,
                Value::Number(n) => obj.is_number() && obj.as_f64().unwrap() == n.as_f64().unwrap(),
                Value::String(s) => obj.is_string() && obj.as_str().unwrap() == s,
                Value::Array(a) => {
                    if !obj.is_array() {
                        return false;
                    }

                    let obj = obj.as_array().unwrap();

                    if a.len() != obj.len() {
                        return false;
                    }

                    for (_i, _v) in a.iter().enumerate() {
                        // println!("{}: {:?}", k, v);
                        // if !matches(v, &obj[i]) {
                        // return false;
                        // }
                    }

                    return true;
                }
                Value::Object(o) => {
                    let obj = obj.as_object().unwrap();

                    for (k, v) in o {
                        if !obj.contains_key(k) {
                            return false;
                        }

                        if let Ok(o) = v.clone().try_into() {
                            if !obj_matches(&o, &obj[k]) {
                                return false;
                            }
                        } else if !obj_matches(&ObjMatcher::Spec(v.clone()), &obj[k]) {
                            return false;
                        }
                    }

                    return true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test() {
        let input = r#"{"$or": [{ "a": {"$or": [ 1, 2 ]} }, { "b": 2 }]}"#;
        let matcher: ObjMatcher = serde_json::from_str(input).unwrap();
        let val = serde_json::json!({"a": 2});
        assert_eq!(obj_matches(&matcher, &val), true);
    }
}
