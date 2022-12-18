use serde::{Deserialize, Serialize};
use serde_json::Value;

trait MatchesValue {
    fn matches(self, other: &Value) -> bool;
}

macro_rules! operator_struct {
    ($struct_name:ident, $json_operator:expr) => {
        operator_struct!($struct_name, $json_operator, Box<ObjMatcher>);
    };
    ($struct_name:ident, $json_operator:expr, $type:ty) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $struct_name {
            #[serde(rename = $json_operator)]
            val: $type,
        }
    };
}

operator_struct!(EqOperator, "$eq");

impl MatchesValue for EqOperator {
    #[inline]
    fn matches(self, other: &Value) -> bool {
        return self.val.matches(other);
    }
}

operator_struct!(InOperator, "$in", Vec<ObjMatcher>);

impl MatchesValue for InOperator {
    #[inline]
    fn matches(self, other: &Value) -> bool {
        for v in self.val {
            if v.matches(other) {
                return true;
            }
        }

        return false;
    }
}

operator_struct!(NeOperator, "$ne");

impl MatchesValue for NeOperator {
    #[inline]
    fn matches(self, other: &Value) -> bool {
        return !self.val.matches(other);
    }
}

operator_struct!(NinOperator, "$nin", Vec<ObjMatcher>);

impl MatchesValue for NinOperator {
    #[inline]
    fn matches(self, other: &Value) -> bool {
        for v in self.val {
            if v.matches(other) {
                return false;
            }
        }

        return true;
    }
}

operator_struct!(AndOperator, "$and", Vec<ObjMatcher>);

impl MatchesValue for AndOperator {
    #[inline]
    fn matches(self, other: &Value) -> bool {
        for v in self.val {
            if !v.matches(other) {
                return false;
            }
        }

        return true;
    }
}

operator_struct!(NotOperator, "$not");

impl MatchesValue for NotOperator {
    #[inline]
    fn matches(self, other: &Value) -> bool {
        if self.val.matches(other) {
            return false;
        }

        return true;
    }
}

// operator_struct!(NorOperator, "$nor", Vec<ObjMatcher>);
operator_struct!(OrOperator, "$or", Vec<ObjMatcher>);

impl MatchesValue for OrOperator {
    #[inline]
    fn matches(self, other: &Value) -> bool {
        for v in self.val {
            if v.matches(other) {
                return true;
            }
        }

        return false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObjMatcher {
    Eq(EqOperator),
    In(InOperator),
    Ne(NeOperator),
    Nin(NinOperator),
    And(AndOperator),
    Not(NotOperator),
    Or(OrOperator),
    Value(Value),
}

fn try_into_operator<'a>(value: &'a Value) -> Option<ObjMatcher> {
    if let Some(obj) = value.as_object() {
        if obj.contains_key("$eq") {
            return Some(ObjMatcher::Eq(
                serde_json::from_value(value.clone()).unwrap(),
            ));
        } else if obj.contains_key("$in") {
            return Some(ObjMatcher::In(
                serde_json::from_value(value.clone()).unwrap(),
            ));
        } else if obj.contains_key("$ne") {
            return Some(ObjMatcher::Ne(
                serde_json::from_value(value.clone()).unwrap(),
            ));
        } else if obj.contains_key("$nin") {
            return Some(ObjMatcher::Nin(
                serde_json::from_value(value.clone()).unwrap(),
            ));
        } else if obj.contains_key("$and") {
            return Some(ObjMatcher::And(
                serde_json::from_value(value.clone()).unwrap(),
            ));
        } else if obj.contains_key("$not") {
            return Some(ObjMatcher::Not(
                serde_json::from_value(value.clone()).unwrap(),
            ));
        } else if obj.contains_key("$or") {
            return Some(ObjMatcher::Or(
                serde_json::from_value(value.clone()).unwrap(),
            ));
        }
    }
    None
}

impl MatchesValue for ObjMatcher {
    fn matches(self, other: &Value) -> bool {
        match self {
            ObjMatcher::Eq(op) => op.matches(other),
            ObjMatcher::In(op) => op.matches(other),
            ObjMatcher::Ne(op) => op.matches(other),
            ObjMatcher::Nin(op) => op.matches(other),
            ObjMatcher::And(op) => op.matches(other),
            ObjMatcher::Not(op) => op.matches(other),
            ObjMatcher::Or(op) => op.matches(other),
            ObjMatcher::Value(value) => match try_into_operator(&value) {
                Some(obj_matcher) => obj_matcher.matches(other),
                None => match value {
                    Value::Number(n) => match other {
                        Value::Number(n2) => &n == n2,
                        _ => false,
                    },
                    Value::Object(o) => {
                        for (key, value) in o {
                            if let Some(obj_matcher) = try_into_operator(&value) {
                                if !obj_matcher.matches(&other[key]) {
                                    return false;
                                }
                            } else {
                                if value != other[key] {
                                    return false;
                                }
                            }
                        }
                        true
                    }
                    _ => {
                        todo!("not implemented value match {:?}", other)
                    }
                },
            },
            e => todo!("{:?} not implemented", e),
        }
    }
}

//             match v {
//                 Value::Null => true,
//                 Value::Bool(b) => obj.is_boolean() && obj.as_bool().unwrap() == *b,
//                 Value::Number(n) => obj.is_number() && obj.as_f64().unwrap() == n.as_f64().unwrap(),
//                 Value::String(s) => obj.is_string() && obj.as_str().unwrap() == s,
//                 Value::Array(a) => {
//                     if !obj.is_array() {
//                         return false;
//                     }

//                     let obj = obj.as_array().unwrap();

//                     if a.len() != obj.len() {
//                         return false;
//                     }

//                     for (_i, _v) in a.iter().enumerate() {
//                         // println!("{}: {:?}", k, v);
//                         // if !matches(v, &obj[i]) {
//                         // return false;
//                         // }
//                     }

//                     return true;
//                 }
//                 Value::Object(o) => {
//                     let obj = obj.as_object().unwrap();

//                     for (k, v) in o {
//                         if !obj.contains_key(k) {
//                             return false;
//                         }

//                         if let Ok(o) = v.clone().try_into() {
//                             if !obj_matches(&o, &obj[k]) {
//                                 return false;
//                             }
//                         } else if !obj_matches(&ObjMatcher::Spec(v.clone()), &obj[k]) {
//                             return false;
//                         }
//                     }

//                     return true;

pub fn from_str(s: &str) -> Result<ObjMatcher, serde_json::Error> {
    let v: Value = serde_json::from_str(s)?;
    match try_into_operator(&v) {
        Some(obj_matcher) => Ok(obj_matcher),
        None => Ok(ObjMatcher::Value(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    pub fn test() {
        let input = r#"{"$or": [{ "a": {"$or": [ 1, 2 ]} }, { "b": 2 }]}"#;
        let matcher: ObjMatcher = from_str(input).unwrap();
        let val = json!({"a": 1});
        assert_eq!(matcher.matches(&val), true);

        let matcher: ObjMatcher = from_str(input).unwrap();
        let val = json!({"a": 2});
        assert_eq!(matcher.matches(&val), true);

        let matcher: ObjMatcher = from_str(input).unwrap();
        let val = json!({"a": 3});
        assert_eq!(matcher.matches(&val), false);

        let matcher: ObjMatcher = from_str(input).unwrap();
        let val = json!({"b": 1});
        assert_eq!(matcher.matches(&val), false);

        let matcher: ObjMatcher = from_str(input).unwrap();
        let val = json!({"b": 2});
        assert_eq!(matcher.matches(&val), true);
    }
}
