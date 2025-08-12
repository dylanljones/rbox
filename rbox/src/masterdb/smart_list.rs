// Author: Dylan Jones
// Date:   2025-08-12

use anyhow::anyhow;
use serde::de::Error as DError;
use serde::ser::Error as SError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;

/// Converts a 32-bit unsigned integer to a signed integer, wrapping if needed.
///
/// This function masks the input to 32 bits and interprets it as a signed `i32`,
/// then returns the result as `i64`. Values greater than `i32::MAX` will wrap into
/// the negative range, simulating how unsigned values map to signed in two's complement.
///
/// # Arguments
/// * `x` - The integer to convert.
///
/// # Returns
/// The signed 32-bit representation as `i64`.
fn left_bitshift(x: i64) -> i64 {
    (x & 0xFFFFFFFF) as i32 as i64
}

/// Converts a 32-bit unsigned integer to a signed integer, wrapping if needed.
///
/// This function masks the input to 32 bits and interprets it as a signed `i32`,
/// then returns the result as `i64`. Values greater than `i32::MAX` will wrap into
/// the negative range, simulating how unsigned values map to signed in two's complement.
///
/// # Arguments
/// * `x` - The integer to convert.
///
/// # Returns
/// The signed 32-bit representation as `i64`.
fn right_bitshift(x: i64) -> i64 {
    (x as i32 as u32) as i64
}

/// Represents logical operators used in smart lists.
///
/// This enum is used to define how conditions in smart lists are combined.
#[repr(i32)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicalOperator {
    /// All conditions must be true.
    All = 1,
    /// At least one condition must be true.
    Any = 2,
}

/// Implements conversion from `i32` to `LogicalOperator`.
impl TryFrom<i32> for LogicalOperator {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(LogicalOperator::All),
            2 => Ok(LogicalOperator::Any),
            _ => Err("Invalid value for LogicalOperator"),
        }
    }
}

/// Serializes the `LogicalOperator` enum to an integer.
fn serialize_logical_op<S>(op: &LogicalOperator, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(op.clone() as i32)
}

/// Deserializes an integer to the `LogicalOperator` enum.
fn deserialize_logical_op<'de, D>(deserializer: D) -> Result<LogicalOperator, D::Error>
where
    D: Deserializer<'de>,
{
    let num = i32::deserialize(deserializer)?;
    match LogicalOperator::try_from(num) {
        Ok(op) => Ok(op),
        Err(e) => Err(D::Error::custom(format!("Failed to deserialize: {}", e))),
    }
}

/// Represents comparison operators for smart list conditions.
///
/// Each variant specifies a type of comparison or match operation that can be used
/// to filter properties in a smart list.
#[repr(i32)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operator {
    /// Values are equal.
    Equal = 1,
    /// Values are not equal.
    NotEqual = 2,
    /// Value is greater than the specified value.
    Greater = 3,
    /// Value is less than the specified value.
    Less = 4,
    /// Value is in the given range.
    InRange = 5,
    /// Value is last (for example 'date added in last 6 days')
    InLast = 6,
    /// Value is not in last (for example 'date added not in last 6 days')
    NotInLast = 7,
    /// Value contains the specified substring.
    Contains = 8,
    /// Value does not contain the specified substring.
    NotContains = 9,
    /// Value starts with the specified substring.
    StartsWith = 10,
    /// Value ends with the specified substring.
    EndsWith = 11,
}

/// Implements conversion from `i32` to `Operator`.
impl TryFrom<i32> for Operator {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Operator::Equal),
            2 => Ok(Operator::NotEqual),
            3 => Ok(Operator::Greater),
            4 => Ok(Operator::Less),
            5 => Ok(Operator::InRange),
            6 => Ok(Operator::InLast),
            7 => Ok(Operator::NotInLast),
            8 => Ok(Operator::Contains),
            9 => Ok(Operator::NotContains),
            10 => Ok(Operator::StartsWith),
            11 => Ok(Operator::EndsWith),
            _ => Err("Invalid value for Operator"),
        }
    }
}

/// Serializes the `Operator` enum to an integer.
fn serialize_op<S>(op: &Operator, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(op.clone() as i32)
}

/// Deserializes an integer to the `Operator` enum.
fn deserialize_op<'de, D>(deserializer: D) -> Result<Operator, D::Error>
where
    D: Deserializer<'de>,
{
    let num = i32::deserialize(deserializer)?;
    match Operator::try_from(num) {
        Ok(op) => Ok(op),
        Err(e) => Err(D::Error::custom(format!(
            "Failed to convert hex to ID: {}",
            e
        ))),
    }
}

/// Represents properties that can be used in smart lists.
///
/// This enum defines the various properties that can be used to filter tracks in smart lists.
/// Each variant corresponds to a specific track attribute that can be queried or filtered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Property {
    #[serde(rename = "artist")]
    Artist,
    #[serde(rename = "album")]
    Album,
    #[serde(rename = "albumArtist")]
    AlbumArtist,
    #[serde(rename = "originalArtist")]
    OriginalArtist,
    #[serde(rename = "bpm")]
    BPM,
    #[serde(rename = "grouping")]
    Grouping,
    #[serde(rename = "comments")]
    Comments,
    #[serde(rename = "producer")]
    Producer,
    #[serde(rename = "stockDate")]
    StockDate,
    #[serde(rename = "dateCreated")]
    DateCreated,
    #[serde(rename = "counter")]
    Counter,
    #[serde(rename = "filename")]
    Filename,
    #[serde(rename = "genre")]
    Genre,
    #[serde(rename = "key")]
    Key,
    #[serde(rename = "label")]
    Label,
    #[serde(rename = "mix_name")]
    MixName,
    #[serde(rename = "mytag")]
    MyTag,
    #[serde(rename = "rating")]
    Rating,
    #[serde(rename = "dateReleased")]
    DateReleased,
    #[serde(rename = "remixedBy")]
    Remixer,
    #[serde(rename = "duration")]
    Duration,
    #[serde(rename = "name")]
    Name,
    #[serde(rename = "year")]
    Year,
}

/// Represents a condition in a smart list filter.
///
/// Each `Condition` specifies a property to filter on, the operator to use for comparison,
/// the unit of the value(s), and the left and right values for the condition.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Name of the property
    #[serde(rename = "@PropertyName")]
    pub property: Property,
    /// Condition operator
    #[serde(
        rename = "@Operator",
        serialize_with = "serialize_op",
        deserialize_with = "deserialize_op"
    )]
    pub operator: Operator,
    /// Unit of the value(s)
    #[serde(rename = "@ValueUnit")]
    pub unit: String,
    /// Left value
    #[serde(rename = "@ValueLeft")]
    pub left: String,
    /// Right value
    #[serde(rename = "@ValueRight")]
    pub right: String,
}

impl Condition {
    /// Create a new condition with the specified property, operator, unit, left and right values.
    pub fn new(
        property: Property,
        operator: Operator,
        left: String,
        right: Option<String>,
        unit: Option<String>,
    ) -> Self {
        Condition {
            property,
            operator,
            unit: unit.unwrap_or("".to_string()),
            left,
            right: right.unwrap_or("".to_string()),
        }
    }
}

/// Serializes the ID by left bit-shifting it.
fn serialize_id<S>(id: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let num: i64 = match id.parse() {
        Ok(n) => n,
        Err(e) => {
            return Err(S::Error::custom(format!("Failed to parse ID: {}", e)));
        }
    };
    let shifted = left_bitshift(num);
    let id = format!("{}", shifted);
    serializer.serialize_str(&id)
}

/// Deserializes the ID by right bit-shifting it.
fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let id = String::deserialize(deserializer)?;
    let num: i64 = match id.parse() {
        Ok(n) => n,
        Err(_) => {
            return Err(D::Error::custom(format!("Failed to parse ID: {}", id)));
        }
    };
    let shifted = right_bitshift(num);
    let id = format!("{}", shifted);
    Ok(id)
}

/// Represents the root element of a Rekordbox SmartList XML.
///
/// The `SmartList` struct defines a smart playlist, including the corresponding Playlist ID,
/// logical operator for combining conditions, auto-update setting, and the list of conditions.
/// Fields are serialized/deserialized with custom attributes to match the expected XML structure.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "NODE")]
pub struct SmartList {
    /// The ID of the playlist
    #[serde(
        rename = "@Id",
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_id"
    )]
    pub id: String,
    /// Logical operator for combining conditions
    #[serde(
        rename = "@LogicalOperator",
        serialize_with = "serialize_logical_op",
        deserialize_with = "deserialize_logical_op"
    )]
    pub logical_operator: LogicalOperator,
    /// Auto-Update smart list
    #[serde(rename = "@AutomaticUpdate")]
    pub automatic_update: i32,
    /// Smart playlist conditions
    #[serde(rename = "CONDITION")]
    pub conditions: Vec<Condition>,
}

impl SmartList {
    /// Create a new `SmartList` with the specified ID, logical operator, auto-update setting, and conditions.
    pub fn new(id: &str, logical_operator: LogicalOperator) -> Self {
        Self {
            id: id.to_string(),
            logical_operator,
            automatic_update: 0, // Default to auto-update enabled
            conditions: Vec::new(),
        }
    }

    /// Parse the smart list XML string
    pub fn parse(data: &str) -> anyhow::Result<Self> {
        let smart: Self = quick_xml::de::from_str(data)?;
        Ok(smart)
    }

    /// Parse the smart list XML string
    pub fn to_string(&self) -> anyhow::Result<String> {
        let mut string = String::new();
        let ser = quick_xml::se::Serializer::new(&mut string);
        self.serialize(ser)?;
        Ok(string)
    }

    /// Add a condition to the smart list.
    pub fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }

    pub fn new_condition(
        &mut self,
        property: Property,
        operator: Operator,
        left: String,
        right: Option<String>,
        unit: Option<String>,
    ) {
        let condition = Condition::new(property, operator, left, right, unit);
        self.add_condition(condition);
    }

    pub fn build_sql(&self) -> anyhow::Result<String> {
        let mut sql = "SELECT * FROM djmdContent".to_string();
        let (joins, filters) = build_clauses(&self.conditions)?;

        // Add joins
        let unique_joins: HashSet<String> = joins.into_iter().collect();
        for join in unique_joins {
            sql.push_str(&format!(" LEFT JOIN {}", join));
        }

        // Add filters
        let sep = match self.logical_operator {
            LogicalOperator::All => " AND ",
            LogicalOperator::Any => " OR ",
        };
        if !filters.is_empty() {
            sql.push_str(" WHERE ");
            let mut first = true;
            for filter in filters {
                if !first {
                    sql.push_str(sep);
                }
                sql.push_str(&filter);
                first = false;
            }
        }
        Ok(sql)
    }
}

fn build_string_filter(column: &str, condition: &Condition) -> anyhow::Result<String> {
    let value = condition.left.replace("'", "''"); // Escape single quotes
    match condition.operator {
        Operator::Equal => Ok(format!("{} = '{}'", column, value)),
        Operator::NotEqual => Ok(format!("{} != '{}'", column, value)),
        Operator::Contains => Ok(format!("{} LIKE '%{}%'", column, value)),
        Operator::NotContains => Ok(format!("{} NOT LIKE '%{}%'", column, value)),
        Operator::StartsWith => Ok(format!("{} LIKE '{}%'", column, value)),
        Operator::EndsWith => Ok(format!("{} LIKE '%{}'", column, value)),
        _ => Err(anyhow!(
            "Unsupported operator {:?} for {}",
            condition.operator,
            column
        )),
    }
}

fn build_number_filter(column: &str, condition: &Condition) -> anyhow::Result<String> {
    match condition.operator {
        Operator::Equal => Ok(format!("{} = '{}'", column, condition.left)),
        Operator::NotEqual => Ok(format!("{} != '{}'", column, condition.left)),
        Operator::Greater => Ok(format!("{} > '{}'", column, condition.left)),
        Operator::Less => Ok(format!("{} < '{}'", column, condition.left)),
        Operator::InRange => Ok(format!(
            "{} > '{}' AND {} < {}",
            column, condition.left, column, condition.right
        )),
        _ => Err(anyhow!(
            "Unsupported operator {:?} for {:?}",
            condition.operator,
            condition.property
        )),
    }
}

fn build_grouping_filter(condition: &Condition) -> anyhow::Result<String> {
    match condition.operator {
        Operator::Equal => Ok(format!("ColorID = '{}'", condition.left)),
        Operator::NotEqual => Ok(format!("ColorID != '{}'", condition.left)),
        _ => Err(anyhow!(
            "Unsupported operator {:?} for {:?}",
            condition.operator,
            condition.property
        )),
    }
}

fn build_clauses(conditions: &Vec<Condition>) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let mut joins: Vec<String> = Vec::new();
    let mut filters: Vec<String> = Vec::new();
    for condition in conditions {
        println!("{:#?}", condition);
        match condition.property {
            Property::Artist => {
                joins.push("djmdArtist ON djmdContent.ArtistID = djmdArtist.ID".into());
                filters.push(build_string_filter("djmdArtist.Name", condition)?);
            }
            Property::Album => {
                joins.push("djmdAlbum ON djmdContent.AlbumID = djmdAlbum.ID".into());
                filters.push(build_string_filter("djmdAlbum.Name", condition)?);
            }
            Property::AlbumArtist => {
                todo!()
            }
            Property::OriginalArtist => {
                joins.push("djmdArtist ON djmdContent.OrgArtistID = djmdArtist.ID".into());
                filters.push(build_string_filter("djmdArtist.Name", condition)?);
            }
            Property::BPM => {
                filters.push(build_string_filter("djmdArtist.Name", condition)?);
            }
            Property::Grouping => {
                filters.push(build_grouping_filter(condition)?);
            }
            Property::Comments => {
                filters.push(build_string_filter("Commnt", condition)?);
            }
            Property::Producer => {
                joins.push("djmdArtist ON djmdContent.ComposerID = djmdAlbum.ID".into());
                filters.push(build_string_filter("djmdArtist.Name", condition)?);
            }
            Property::StockDate => {
                todo!()
            }
            Property::DateCreated => {
                todo!()
            }
            Property::Counter => {
                filters.push(build_number_filter("DJPlayCount", condition)?);
            }
            Property::Filename => {
                filters.push(build_string_filter("FileNameL", condition)?);
            }
            Property::Genre => {
                joins.push("djmdGenre ON djmdContent.GenreID = djmdGenre.ID".into());
                filters.push(build_string_filter("djmdGenre.Name", condition)?);
            }
            Property::Key => {
                joins.push("djmdKey ON djmdContent.KeyID = djmdKey.ID".into());
                filters.push(build_string_filter("djmdKey.ScaleName", condition)?);
            }
            Property::Label => {
                joins.push("djmdLabel ON djmdContent.LabelID = djmdLabel.ID".into());
                filters.push(build_string_filter("djmdLabel.Name", condition)?);
            }
            Property::MixName => {
                // MixName is not a direct property in djmdContent, so we skip it
                return Err(anyhow!("MixName is not supported in smart lists"));
            }
            Property::MyTag => {
                todo!()
            }
            Property::Rating => {
                filters.push(build_number_filter("Rating", condition)?);
            }
            Property::DateReleased => {
                todo!()
            }
            Property::Remixer => {
                joins.push("djmdArtist ON djmdContent.RemixerID = djmdArtist.ID".into());
                filters.push(build_string_filter("djmdArtist.Name", condition)?);
            }
            Property::Duration => {
                filters.push(build_number_filter("Length", condition)?);
            }
            Property::Name => {
                filters.push(build_string_filter("Title", condition)?);
            }
            Property::Year => {
                filters.push(build_number_filter("ReleaseYear", condition)?);
            }
        }
    }
    Ok((joins, filters))
}
