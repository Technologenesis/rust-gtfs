use csv;
use std::io;
use std::iter;
use std::collections;
use std::fmt;
use std::str::FromStr;

use hex_color;

// Routes is a collection of routes, indexed by route_id.
pub struct Routes {
    pub routes: std::collections::HashMap<String, Route>
}

impl<'a> iter::IntoIterator for &'a Routes {
    type Item = &'a Route;
    type IntoIter = std::collections::hash_map::Values<'a, String, Route>;

    fn into_iter(self) -> Self::IntoIter {
        self.routes.values()
    }
}

impl iter::IntoIterator for Routes {
    type Item = Route;
    type IntoIter = std::collections::hash_map::IntoValues<String, Route>;

    fn into_iter(self) -> Self::IntoIter {
        self.routes.into_values()
    }
}

// RoutesCsvLoadError is an error that occurs when loading routes from a CSV file.
pub enum RoutesCsvLoadError {
    NoHeader,
    RouteLoadError(RouteLoadError),
    CSVReadError(csv::Error)
}

impl fmt::Display for RoutesCsvLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeader => write!(f, "No header found"),
            Self::RouteLoadError(e) => write!(f, "Error loading route: {}", e),
            Self::CSVReadError(e) => write!(f, "Error reading CSV: {}", e)
        }
    }
}

// Routes implements TryFrom<csv::Reader<R>> by attempting to consume and read from a csv::Reader<R>.
impl<R: io::Read> TryFrom<csv::Reader<R>> for Routes {
    // The error type for this function is RoutesCsvLoadError.
    type Error = RoutesCsvLoadError;

    // try_from consumes the csv::Reader<R> and returns a Result holding a Routes object, or a RoutesCsvLoadError.
    fn try_from(mut r: csv::Reader<R>) -> Result<Self, Self::Error> {
        // try to get the headers; if there are no headers, return a RoutesCsvLoadError::NoHeader.
        r.headers().cloned().map_err(|_| RoutesCsvLoadError::NoHeader).and_then(
            // if there are headers, try to create a Routes object from the remaining records.
            |header|
            Ok(Routes {
                // to create the actual collection of routes, we need to iterate over the records
                routes: r.into_records()
                    // and fold them into an overarching result containing the collection.
                    .fold(
                        Ok(collections::HashMap::new()),
                        // at each stage of the fold,
                        |routes_result, record_result|
                        // proceed only if the running result is Ok.
                        routes_result
                        .and_then(|mut routes|
                            // if there was an error reading this record, return that error.
                            record_result.map_err(RoutesCsvLoadError::CSVReadError)
                                .and_then(
                                    // otherwise, try to create a Route object from the record.
                                    |record|
                                    Route::try_from(
                                        // Zip the header and record together,
                                        iter::zip(
                                            header.iter().map(|s| s.to_string()),
                                            record.iter().map(|s| s.to_string())
                                        )
                                        // and collect the results into a HashMap.
                                        .collect::<collections::HashMap<String, String>>()
                                    )
                                    // if there is an error creating the Route object from the HashMap, return that error.
                                    .map_err(|err| RoutesCsvLoadError::RouteLoadError(err))
                            )
                            .map(
                                |route| {
                                    // insert the Route object into the HashMap.
                                    routes.insert(route.route_id.clone(), route);
                                    // return the updated HashMap.
                                    routes
                                }
                            )
                        )
                    // extract the HashMap from the Result, or return the error.
                    )?
            })
        )
    }
}

#[derive(Debug)]
pub struct Route {
    pub route_id: String,
    pub agency_id: Option<String>,
    name: RouteName,
    pub route_desc: Option<String>,
    pub route_type: RouteType,
    pub route_url: Option<String>,
    pub route_color: Option<hex_color::HexColor>,
    pub route_text_color: Option<hex_color::HexColor>,
    pub route_sort_order: Option<usize>,
    pub continuous_pickup: Option<RouteContinuityPolicy>,
    pub continuous_drop_off: Option<RouteContinuityPolicy>,
    pub network_id: Option<String>,
}

impl Route {
    pub fn route_long_name(&self) -> Option<&str> {
        self.name.long()
    }

    pub fn route_short_name(&self) -> Option<&str> {
        self.name.short()
    }

    pub fn long_or_short_name(&self) -> &str {
        match &self.name {
            RouteName::Long(name) => &name,
            RouteName::Short(name) => &name,
            RouteName::LongAndShort(long, _) => &long
        }
    }

    pub fn short_or_long_name(&self) -> &str {
        match &self.name {
            RouteName::Long(name) => &name,
            RouteName::Short(name) => &name,
            RouteName::LongAndShort(_, short) => &short
        }
    }

    pub fn name(&self) -> &str {
        self.long_or_short_name()
    }
}

pub enum RouteLoadError {
    RouteIdRequired,
    RouteNameError(String),
    ParseRouteColorError(hex_color::ParseHexColorError),
    ParseRouteTextColorError(hex_color::ParseHexColorError),
    ParseRouteSortOrderError(std::num::ParseIntError),
    RouteTypeError(String),
    InvalidContinuousPickup(RouteContinuityPolicyLoadError),
    InvalidContinuousDropOff(RouteContinuityPolicyLoadError),
}

impl fmt::Display for RouteLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RouteIdRequired => write!(f, "route_id is required"),
            Self::RouteNameError(e) => write!(f, "Error parsing route name: {}", e),
            Self::ParseRouteColorError(e) => write!(f, "Error parsing route color: {}", e),
            Self::ParseRouteTextColorError(e) => write!(f, "Error parsing route text color: {}", e),
            Self::ParseRouteSortOrderError(e) => write!(f, "Error parsing route sort order: {}", e),
            Self::RouteTypeError(e) => write!(f, "Error parsing route type: {}", e),
            Self::InvalidContinuousPickup(e) => write!(f, "Error parsing continuous pickup: {}", e),
            Self::InvalidContinuousDropOff(e) => write!(f, "Error parsing continuous drop off: {}", e),
        }
    }
}

// Route implements TryFrom<collections::HashMap<String, String>> by interpreting the keys as field names, and
// the values as string-encoded values for those fields.
impl TryFrom<collections::HashMap<String, String>> for Route {
    type Error = RouteLoadError;

    fn try_from(fields: collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Route {
            route_id: fields.get("route_id")
                .filter(|s| !s.is_empty())
                .ok_or(RouteLoadError::RouteIdRequired)?
                .clone(),
            agency_id: fields.get("agency_id").filter(|s| !s.is_empty()).cloned(),
            name: RouteName::try_from(&fields).map_err(|e| RouteLoadError::RouteNameError(e))?,
            route_desc: fields.get("route_desc").filter(|s| !s.is_empty()).cloned(),
            route_type: RouteType::try_from(&fields).map_err(|e| RouteLoadError::RouteTypeError(e))?,
            route_url: fields.get("route_url").filter(|s| !s.is_empty()).cloned(),
            route_color: match
                    fields.get("route_color")
                        .filter(|s| !s.is_empty())
                        .map(|s| hex_color::HexColor::from_str(&(String::from("#") + s)))
                {
                    Some(res) => res
                        .map(|color| Some(color))
                        .map_err(|e| RouteLoadError::ParseRouteColorError(e))?,
                    None => Ok(None)?
                },
            route_text_color: match
                fields.get("route_text_color")
                    .filter(|s| !s.is_empty())
                    .map(|s| hex_color::HexColor::from_str(&(String::from("#") + s)))
                {
                    Some(res) => res
                        .map(|color| Some(color))
                        .map_err(|e| RouteLoadError::ParseRouteTextColorError(e))?,
                    None => Ok(None)?
                },
            route_sort_order: match
                fields.get("route_sort_order")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<usize>())
                {
                    Some(res) => res
                        .map(|order| Some(order))
                        .map_err(|e| RouteLoadError::ParseRouteSortOrderError(e)),
                    None => Ok(None)
                }?,
            continuous_pickup: match fields.get("continuous_pickup")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<RouteContinuityPolicy>()) {
                    Some(res) => res
                        .map(|policy| Some(policy))
                        .map_err(|e| RouteLoadError::InvalidContinuousPickup(e)),
                    None => Ok(None)
                }?,
            continuous_drop_off: match fields.get("continuous_drop_off")
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<RouteContinuityPolicy>()) {
                    Some(res) => res
                        .map(|policy| Some(policy))
                        .map_err(|e| RouteLoadError::InvalidContinuousDropOff(e)),
                    None => Ok(None)
                }?,
            network_id: fields.get("network_id").filter(|s| !s.is_empty()).cloned(),
        })
    }
}

// RouteName is a type that represents the name of a route.
// It represents the requirement that a route must have at
// least one of a short name or a long name.
#[derive(Debug)]
pub enum RouteName {
    Short(String),
    Long(String),
    LongAndShort(String, String),
}

impl RouteName {
    pub fn long(&self) -> Option<&str> {
        match self {
            RouteName::Long(name) => Some(name),
            RouteName::LongAndShort(name, _) => Some(name),
            RouteName::Short(_) => None
        }
    }

    pub fn short(&self) -> Option<&str> {
        match self {
            RouteName::Short(name) => Some(name),
            RouteName::LongAndShort(_, name) => Some(name),
            RouteName::Long(_) => None
        }
    }
}

impl TryFrom<&collections::HashMap<String, String>> for RouteName {
    type Error = String;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        let short_name = fields.get("route_short_name").filter(|s| !s.is_empty()).cloned();
        let long_name = fields.get("route_long_name").filter(|s| !s.is_empty()).cloned();

        match (short_name, long_name) {
            (Some(short), Some(long)) => Ok(RouteName::LongAndShort(long, short)),
            (Some(short), None) => Ok(RouteName::Short(short)),
            (None, Some(long)) => Ok(RouteName::Long(long)),
            (None, None) => Err(String::from("route_short_name or route_long_name is required")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RouteContinuityPolicy {
    Continuous,
    NotContinuous,
    Prearrange,
    CoordinateWithDriver,
}

pub struct RouteContinuityPolicyLoadError (String);

impl fmt::Display for RouteContinuityPolicyLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid route_continuity_policy '{}'", self.0)
    }
}

impl FromStr for RouteContinuityPolicy {
    type Err = RouteContinuityPolicyLoadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(RouteContinuityPolicy::Continuous),
            "1" => Ok(RouteContinuityPolicy::NotContinuous),
            "2" => Ok(RouteContinuityPolicy::Prearrange),
            "3" => Ok(RouteContinuityPolicy::CoordinateWithDriver),
            _ => Err(RouteContinuityPolicyLoadError(s.to_string())),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RouteType {
    TramStreetcarLightRail,
    SubwayMetro,
    Rail,
    Bus,
    Ferry,
    CableTram,
    AerialLift,
    Funicular,
    Trolleybus,
    Monorail,
}

impl TryFrom<&collections::HashMap<String, String>> for RouteType {
    type Error = String;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        let zero_str = String::from("0");

        let route_type = fields.get("route_type").unwrap_or(&zero_str);

        match route_type.parse::<u8>()
            .map_err(
                |err|
                format!("invalid route_type '{}': {}", route_type, err)
            )?
        {
            0 => Ok(RouteType::TramStreetcarLightRail),
            1 => Ok(RouteType::SubwayMetro),
            2 => Ok(RouteType::Rail),
            3 => Ok(RouteType::Bus),
            4 => Ok(RouteType::Ferry),
            5 => Ok(RouteType::CableTram),
            6 => Ok(RouteType::AerialLift),
            7 => Ok(RouteType::Funicular),
            8 => Ok(RouteType::Trolleybus),
            9 => Ok(RouteType::Monorail),
            _ => Err(format!("invalid route_type '{}'", route_type))
        }.map_err(|err| format!("failed to load route type '{}': {}", route_type, err))
    }
}
