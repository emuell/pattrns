use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

#[cfg(feature = "scripting")]
use mlua::prelude::{IntoLua, Lua, LuaInteger, LuaResult, LuaValue};

// -------------------------------------------------------------------------------------------------

/// Value representation of a parameter.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum ParameterType {
    Boolean,
    #[default]
    Float,
    Integer,
    Enum,
}

// -------------------------------------------------------------------------------------------------

/// A vector of Parameter RefCells. Ids are unique, so this actually is a set, but is stored as
/// a vector to preserve the order of the parameters.
pub type ParameterSet = Vec<Rc<RefCell<Parameter>>>;

// -------------------------------------------------------------------------------------------------

/// Template parameter for a [`Pattern`](crate::Pattern), to dynamically change pattern behavior.
///
/// Parameter values can be changed by the user during playback, and will usually be applied in
/// scripted callbacks only, as those are the only runtime dynamic components in patterns.
#[derive(Debug, Clone)]
pub struct Parameter {
    id: String,
    name: String,
    description: String,
    parameter_type: ParameterType,
    range: RangeInclusive<f64>,
    default: f64,
    value: f64,
    value_strings: Vec<String>,
}

impl Parameter {
    /// Create a new boolean parameter with the given properties.
    ///
    /// Name and description are optional and may be empty, all other values
    /// must be valid.
    ///
    /// ### Panics
    /// Panics if the default value is not in the specified range.
    pub fn with_boolean(id: &str, name: &str, description: &str, default: bool) -> Self {
        let id: String = id.to_string();
        let mut name: String = name.to_string();
        if name.is_empty() {
            name.clone_from(&id);
        }
        let description = description.to_string();
        let parameter_type = ParameterType::Boolean;
        let range = 0.0..=1.0;
        let default = match default {
            true => 1.0,
            false => 0.0,
        };
        let value = default;
        let value_strings = vec![];
        Self {
            id,
            name,
            description,
            parameter_type,
            range,
            default,
            value,
            value_strings,
        }
    }

    /// Create a new integer parameter with the given properties.
    ///
    /// Name and description are optional and may be empty, all other values
    /// must be valid.
    ///
    /// ### Panics
    /// Panics if the default value is not in the specified range.
    pub fn with_integer(
        id: &str,
        name: &str,
        description: &str,
        range: RangeInclusive<i32>,
        default: i32,
    ) -> Self {
        debug_assert!(range.contains(&default), "Invalid parameter default value");

        let id: String = id.to_string();
        let mut name: String = name.to_string();
        if name.is_empty() {
            name.clone_from(&id);
        }
        let description = description.to_string();
        let parameter_type = ParameterType::Integer;
        let range = RangeInclusive::new(*range.start() as f64, *range.end() as f64);
        let default = default as f64;
        let value = default;
        let value_strings = vec![];
        Self {
            id,
            name,
            description,
            parameter_type,
            range,
            default,
            value,
            value_strings,
        }
    }

    /// Create a new float parameter with the given properties.
    ///
    /// Name and description are optional and may be empty, all other values
    /// must be valid.
    ///
    /// ### Panics
    /// Panics if the default value is not in the specified range.
    pub fn with_float(
        id: &str,
        name: &str,
        description: &str,
        range: RangeInclusive<f64>,
        default: f64,
    ) -> Self {
        debug_assert!(range.contains(&default), "Invalid parameter default value");

        let id: String = id.to_string();
        let mut name: String = name.to_string();
        if name.is_empty() {
            name.clone_from(&id);
        }
        let description = description.to_string();
        let parameter_type = ParameterType::Float;
        let value = default;
        let value_strings = vec![];
        Self {
            id,
            name,
            description,
            parameter_type,
            range,
            default,
            value,
            value_strings,
        }
    }

    /// Create a new enum parameter with the given properties.
    ///
    /// Name and description are optional and may be empty, all other values
    /// must be valid.
    ///
    /// ### Panics
    /// Panics if the default value is not in the specified values set.
    pub fn with_enum(
        id: &str,
        name: &str,
        description: &str,
        values: Vec<String>,
        default: String,
    ) -> Self {
        debug_assert!(
            values.iter().any(|v| v.eq_ignore_ascii_case(&default)),
            "Invalid parameter default value"
        );

        let id: String = id.to_string();
        let mut name: String = name.to_string();
        if name.is_empty() {
            name.clone_from(&id);
        }
        let description = description.to_string();
        let parameter_type = ParameterType::Enum;
        let range = 0.0..=(values.len() - 1) as f64;
        let default = values
            .iter()
            .position(|e| e.eq_ignore_ascii_case(&default))
            .unwrap_or(0) as f64;
        let value = default;
        let value_strings = values;
        Self {
            id,
            name,
            description,
            parameter_type,
            range,
            default,
            value,
            value_strings,
        }
    }

    /// Unique id of the parameter. The id will be used in callback context tables as key.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Optional name of the parameter, as displayed to the user. Falls back to id, when unspecified.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Optional long description of the parameter describing what the parameter does.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Defines the value type and display options (boolean -> switch, float -> slider ...).
    pub fn parameter_type(&self) -> ParameterType {
        self.parameter_type
    }

    /// Valid internal value range. Falls back to (0..=1) when unspecified.
    pub fn range(&self) -> &RangeInclusive<f64> {
        &self.range
    }

    /// Valid values for enum parameters.
    pub fn value_strings(&self) -> &[String] {
        &self.value_strings
    }

    /// Default value to reset the parameter.
    pub fn default(&self) -> f64 {
        self.default
    }

    /// Actual parameter value in range.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Set a new parameter value. Value must be in the specified range.
    ///
    /// ### Panics
    /// Panics if the passed value exceeds the specified range
    pub fn set_value(&mut self, value: f64) {
        assert!(self.range.contains(&value), "Invalid value");
        self.value = value;
    }

    /// Reset the value to the default value.
    pub fn reset(&mut self) {
        self.value = self.default
    }

    /// String representation of the value, depending on the parameter type.
    pub fn string_value(&self) -> String {
        match self.parameter_type {
            ParameterType::Boolean => {
                if self.value > 0.5 {
                    "On".to_string()
                } else {
                    "Off".to_string()
                }
            }
            ParameterType::Float => self.value.to_string(),
            ParameterType::Integer => (self.value.round() as i64).to_string(),
            ParameterType::Enum => self.value_strings[self.value.round() as usize].clone(),
        }
    }

    /// Lua value representation of the internal value, depending on the parameter type.
    #[cfg(feature = "scripting")]
    pub fn lua_value(&self, lua: &Lua) -> LuaResult<LuaValue> {
        match self.parameter_type {
            ParameterType::Boolean => {
                if self.value > 0.5 {
                    true.into_lua(lua)
                } else {
                    false.into_lua(lua)
                }
            }
            ParameterType::Float => self.value.into_lua(lua),
            ParameterType::Integer => (self.value.round() as LuaInteger).into_lua(lua),
            ParameterType::Enum => self.value_strings[self.value.round() as usize]
                .clone()
                .into_lua(lua),
        }
    }
}

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.description == other.description
            && self.parameter_type == other.parameter_type
            && self.range == other.range
            && self.default == other.default
            // SKIP value
            && self.value_strings == other.value_strings
    }
}
