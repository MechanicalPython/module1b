use std::collections::HashMap;
use serde::{Deserialize, Deserializer};

/// Structs to define the JSON coming from the NASA NEO API.
/// There are two types of API calls/returns.
/// 1. NEO Lookup. This provides details of a single NEO from a NEO id number.
/// 2. NEO feed. This provides a list of NEOs within that date range.
///
/// NEO details structure:
/// ```
///  "links": { "self": "http://api.nasa.gov/neo/rest/v1/neo/3542519?api_key=DEMO_KEY"},
///  "id": "3542519",
///  "neo_reference_id": "3542519",
///  "name": "(2010 PK9)",
///  "designation": "2010 PK9",
///  "nasa_jpl_url": "https://ssd.jpl.nasa.gov/tools/sbdb_lookup.html#/?sstr=3542519",
///  "absolute_magnitude_h": 21.81,
///  "estimated_diameter": {
///    "kilometers": {"estimated_diameter_min": 0.2170475943, "estimated_diameter_max": 0.4853331752},
///    "meters": {"estimated_diameter_min": 217.0475943071,"estimated_diameter_max": 485.3331752235},
///    "miles": {"estimated_diameter_min": 0.1348670807, "estimated_diameter_max": 0.3015719604},
///    "feet": {"estimated_diameter_min": 712.0984293066, "estimated_diameter_max": 1592.3004946003}},
///   "is_potentially_hazardous_asteroid": true,
/// close_approach_data is a long list of all the times that a NEO will have a close approach to another planet in the past and future.
///   "close_approach_data": [
///      {"close_approach_date": "1900-06-01",
///       "close_approach_date_full": "1900-Jun-01 16:40",
///      "epoch_date_close_approach": -2195882400000,
///      "relative_velocity": {
///        "kilometers_per_second": "30.9354328365",
///        "kilometers_per_hour": "111367.5582113129",
///        "miles_per_hour": "69199.4697119127"
///      },
///      "miss_distance": {
///        "astronomical": "0.0445495565",
///        "lunar": "17.3297774785",
///        "kilometers": "6664518.761844655",
///        "miles": "4141139.931400039"
///      },
///      "orbiting_body": "Merc"
///     },
///     Next close apprach data in list.
///   ],
/// "orbital_data": {
///    "orbit_id": "30",
///    "orbit_determination_date": "2023-08-23 05:49:41",
///    "first_observation_date": "2010-07-18",
///    "last_observation_date": "2023-08-22",
///    "data_arc_in_days": 4783,
///    "observations_used": 123,
///    "orbit_uncertainty": "0",
///    "minimum_orbit_intersection": ".0161596",
///    "jupiter_tisserand_invariant": "8.150",
///    "epoch_osculation": "2460600.5",
///    "eccentricity": ".675827388781843",
///    "semi_major_axis": ".6820681358625633",
///    "inclination": "12.58812105676965",
///    "ascending_node_longitude": "306.5145012039707",
///    "orbital_period": "205.7501064196426",
///    "perihelion_distance": ".2211078086312678",
///    "perihelion_argument": "195.638736952186",
///    "aphelion_distance": "1.143028463093859",
///    "perihelion_time": "2460683.644436243885",
///    "mean_anomaly": "214.5225683298073",
///    "mean_motion": "1.749695328301573",
///    "equinox": "J2000",
///    "orbit_class": {
///      "orbit_class_type": "ATE",
///      "orbit_class_description": "Near-Earth asteroid orbits similar to that of 2062 Aten",
///      "orbit_class_range": "a (semi-major axis) < 1.0 AU; q (perihelion) > 0.983 AU"
///     }
///   },
///   "is_sentry_object": false
/// ```
///
/// NEO feed structure:
///
/// ```"links": {
///     "next": link to next date range,
///     "previous": link to previous date range,
///     "self": link for this date range
///   },
///   "element_count": int,
///   "near_earth_objects": {
/// Keys here are dynamically generated for each day in the date range.
///    "2015-09-08":
///     [
///      { "links": { "self": "http://api.nasa.gov/neo/rest/v1/neo/2465633?api_key=DEMO_KEY"},
///        "id": "2465633",
///        "neo_reference_id": "2465633",
///        "name": "465633 (2009 JR5)",
///        "nasa_jpl_url": "https://ssd.jpl.nasa.gov/tools/sbdb_lookup.html#/?sstr=2465633",
///        "absolute_magnitude_h": 20.44,
///        "estimated_diameter": {
///          "kilometers": {"estimated_diameter_min": 0.2170475943, "estimated_diameter_max": 0.4853331752},
///          "meters": {"estimated_diameter_min": 217.0475943071,"estimated_diameter_max": 485.3331752235},
///          "miles": {"estimated_diameter_min": 0.1348670807, "estimated_diameter_max": 0.3015719604},
///          "feet": {"estimated_diameter_min": 712.0984293066, "estimated_diameter_max": 1592.3004946003}},
///        "is_potentially_hazardous_asteroid": true,
///        "close_approach_data": [
///          {"close_approach_date": "2015-09-08",
///           "close_approach_date_full": "2015-Sep-08 20:28",
///           "epoch_date_close_approach": 1441744080000,
///           "relative_velocity": {"kilometers_per_second": "18.1279360862", "kilometers_per_hour": "65260.5699103704", "miles_per_hour": "40550.3802312521"},
///           "miss_distance": {"astronomical": "0.3027469457", "lunar": "117.7685618773", "kilometers": "45290298.225725659", "miles": "28142086.3515817342"},
///           "orbiting_body": "Earth"}
///           ],
///         "is_sentry_object": false
///       },
///      {Next NEO details}...
///     ],
///     day2: [list of day 2 NEOs]
///```
///


#[derive(Debug, serde::Deserialize)]
pub struct NeoFeed {
    pub links: Links,
    pub element_count: i64,
    pub near_earth_objects: NearEarthObjects,
}

/// For some bizzar reason, when start and end date are the same, it's prev, when they are different,
/// it's previous.
#[derive(Debug, serde::Deserialize)]
pub struct Links {
    pub next: String,
    pub prev: String,
    #[serde(rename = "self")]
    pub field_self: String,
}

/// Keys for NearEarthObjects are the dates in the requested range.
#[derive(Debug, serde::Deserialize)]
pub struct NearEarthObjects {
    #[serde(flatten)]
    pub days: HashMap<String, Vec<BasicNeoInfo>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BasicNeoInfo {
    pub links: NeoLinks,
    pub id: String,
    pub neo_reference_id: String,
    pub name: String,
    pub nasa_jpl_url: String,
    pub absolute_magnitude_h: f64,
    pub estimated_diameter: EstimatedDiameter,
    pub is_potentially_hazardous_asteroid: bool,
    pub close_approach_data: Vec<CloseApproachData>,
    pub is_sentry_object: bool,
}


#[derive(Debug, serde::Deserialize)]
pub struct NeoLinks {
    #[serde(rename = "self")]
    pub field_self: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EstimatedDiameter {
    pub kilometers: DiameterMinMax,
    pub meters: DiameterMinMax,
    pub miles: DiameterMinMax,
    pub feet: DiameterMinMax,
}

#[derive(Debug, serde::Deserialize)]
pub struct DiameterMinMax {
    pub estimated_diameter_min: f64,
    pub estimated_diameter_max: f64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CloseApproachData {
    pub close_approach_date: String,
    pub close_approach_date_full: String,
    pub epoch_date_close_approach: i64,
    pub relative_velocity: RelativeVelocity,
    pub miss_distance: MissDistance,
    pub orbiting_body: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct RelativeVelocity {
    #[serde(deserialize_with = "string_to_f64")]
    pub kilometers_per_second: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub kilometers_per_hour: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub miles_per_hour: f64,
}



#[derive(Debug, serde::Deserialize)]
pub struct MissDistance {
    #[serde(deserialize_with = "string_to_f64")]
    pub astronomical: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub lunar: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub kilometers: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub miles: f64,
}


#[derive(Deserialize, Debug)]
pub struct NeoLookup {
    pub links: NeoLinks,
    pub id: String,
    pub neo_reference_id: String,
    pub name: String,
    pub designation: String,
    pub nasa_jpl_url: String,
    pub absolute_magnitude_h: f64,
    pub estimated_diameter: EstimatedDiameter,
    pub is_potentially_hazardous_asteroid: bool,
    pub close_approach_data: Vec<CloseApproachData>,
    pub orbital_data: OrbitalData,
    pub is_sentry_object: bool,
}

#[derive(Deserialize, Debug)]
pub struct OrbitClass {
    pub orbit_class_type: String,
    pub orbit_class_description: String,
    pub orbit_class_range: String,
}

#[derive(Deserialize, Debug)]
pub struct OrbitalData {
    pub orbit_id: String,
    pub orbit_determination_date: String,
    pub first_observation_date: String,
    pub last_observation_date: String,
    pub data_arc_in_days: i64,
    pub observations_used: i64,
    pub orbit_uncertainty: String,
    pub minimum_orbit_intersection: String,
    pub jupiter_tisserand_invariant: String,
    pub epoch_osculation: String,
    pub eccentricity: String,
    pub semi_major_axis: String,
    pub inclination: String,
    pub ascending_node_longitude: String,
    pub orbital_period: String,
    pub perihelion_distance: String,
    pub perihelion_argument: String,
    pub aphelion_distance: String,
    pub perihelion_time: String,
    pub mean_anomaly: String,
    pub mean_motion: String,
    pub equinox: String,
    pub orbit_class: OrbitClass,
}


fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}