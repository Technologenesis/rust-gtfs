use chrono_tz::Tz;
use csv;
use std::io;
use std::iter;
use std::collections;
use std::fmt;
use std::str::FromStr;

// Stops is a collection of stops, indexed by stop_id.
pub struct Stops {
    pub stops: std::collections::HashMap<String, Stop>
}

impl<'a> iter::IntoIterator for &'a Stops {
    type Item = &'a Stop;
    type IntoIter = std::collections::hash_map::Values<'a, String, Stop>;

    fn into_iter(self) -> Self::IntoIter {
        self.stops.values()
    }
}

impl iter::IntoIterator for Stops {
    type Item = Stop;
    type IntoIter = std::collections::hash_map::IntoValues<String, Stop>;

    fn into_iter(self) -> Self::IntoIter {
        self.stops.into_values()
    }
}

// StopsCsvLoadError is an error that occurs when loading stops from a CSV file.
pub enum StopsCsvLoadError {
    NoHeader,
    StopLoadError(String),
    CSVReadError(csv::Error)
}

impl fmt::Display for StopsCsvLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeader => write!(f, "No header found"),
            Self::StopLoadError(e) => write!(f, "Error loading stop: {}", e),
            Self::CSVReadError(e) => write!(f, "Error reading CSV: {}", e)
        }
    }
}

// Stops implements TryFrom<csv::Reader<R>> by attempting to consume and read from a csv::Reader<R>.
impl<R: io::Read> TryFrom<csv::Reader<R>> for Stops {
    // The error type for this function is StopsCsvLoadError.
    type Error = StopsCsvLoadError;

    // try_from consumes the csv::Reader<R> and returns a Result holding a Stops object, or a StopsCsvLoadError.
    fn try_from(mut r: csv::Reader<R>) -> Result<Self, Self::Error> {
        // try to get the headers; if there are no headers, return a StopsCsvLoadError::NoHeader.
        r.headers().cloned().map_err(|_| StopsCsvLoadError::NoHeader).and_then(
            // if there are headers, try to create a Stops object from the remaining records.
            |header|
            Ok(Stops {
                // to create the actual collection of stops, we need to iterate over the records
                stops: r.into_records()
                    // and fold them into an overarching result containing the collection.
                    .fold(
                        Ok(collections::HashMap::new()),
                        // at each stage of the fold,
                        |stops_result, record_result|
                        // proceed only if the running result is Ok.
                        stops_result
                        .and_then(|mut stops|
                            // if there was an error reading this record, return that error.
                            record_result.map_err(StopsCsvLoadError::CSVReadError)
                                .and_then(
                                    // otherwise, try to create a Stop object from the record.
                                    |record|
                                    Stop::try_from(
                                        // Zip the header and record together,
                                        iter::zip(
                                            header.iter().map(|s| s.to_string()),
                                            record.iter().map(|s| s.to_string())
                                        )
                                        // and collect the results into a HashMap.
                                        .collect::<collections::HashMap<String, String>>()
                                    )
                                    // if there is an error creating the Stop object from the HashMap, return that error.
                                    .map_err(|err| StopsCsvLoadError::StopLoadError(err))
                            )
                            .map(
                                |stop| {
                                    // insert the Stop object into the HashMap.
                                    stops.insert(stop.stop_id.clone(), stop);
                                    // return the updated HashMap.
                                    stops
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
pub struct Stop {
    pub stop_id: String,
    pub stop_code: Option<String>,
    pub tts_stop_name: Option<String>,
    pub stop_desc: Option<String>,
    pub zone_id: Option<String>,
    pub stop_url: Option<String>,
    pub stop_timezone: Option<Tz>,
    pub wheelchair_boarding: Option<bool>,
    pub level_id: Option<String>,
    pub platform_code: Option<String>,
    // encodes location type and type-specific fields
    pub location_type_details: LocationTypeDetails
}

impl Stop {
    // convenience functions to access type-specific fields in a unified way
    pub fn get_stop_name(&self) -> Option<&str> {
        match &self.location_type_details {
            LocationTypeDetails::Stop(stop_details) => Some(&stop_details.stop_name),
            LocationTypeDetails::Station(station_details) => Some(&station_details.stop_name),
            LocationTypeDetails::EntranceExit(entrance_exit_details) => Some(&entrance_exit_details.stop_name),
            LocationTypeDetails::GenericNode(generic_node_details) => generic_node_details.stop_name.as_deref(),
            LocationTypeDetails::BoardingArea(boarding_area_details) => boarding_area_details.stop_name.as_deref()
        }
    }

    pub fn mut_stop_name(&mut self) -> Option<&mut String> {
        match &mut self.location_type_details {
            LocationTypeDetails::Stop(stop_details) => Some(&mut stop_details.stop_name),
            LocationTypeDetails::Station(station_details) => Some(&mut station_details.stop_name),
            LocationTypeDetails::EntranceExit(entrance_exit_details) => Some(&mut entrance_exit_details.stop_name),
            LocationTypeDetails::GenericNode(generic_node_details) => generic_node_details.stop_name.as_mut(),
            LocationTypeDetails::BoardingArea(boarding_area_details) => boarding_area_details.stop_name.as_mut()
        }
    }

    // take_stop_name gives ownership of the stop name, if it exists.
    // it also returns ownership of the given Stop with the stop name removed,
    // if its location_type permits an empty stop name.
    pub fn take_stop_name(mut self) -> (Option<String>, Option<Stop>) {
        match &mut (self.location_type_details) {
            LocationTypeDetails::Stop(stop_details) => (Some(std::mem::take(&mut stop_details.stop_name)), None),
            LocationTypeDetails::Station(station_details) => (Some(std::mem::take(&mut station_details.stop_name)), None),
            LocationTypeDetails::EntranceExit(entrance_exit_details) => (Some(std::mem::take(&mut entrance_exit_details.stop_name)), None),
            LocationTypeDetails::GenericNode(generic_node_details) => (std::mem::take(&mut generic_node_details.stop_name), Some(self)),
            LocationTypeDetails::BoardingArea(boarding_area_details) => (std::mem::take(&mut boarding_area_details.stop_name), Some(self))
        }
    }

    pub fn stop_lat(&self) -> Option<f64> {
        match &self.location_type_details {
            LocationTypeDetails::Stop(stop_details) => Some(stop_details.stop_lat),
            LocationTypeDetails::Station(station_details) => Some(station_details.stop_lat),
            LocationTypeDetails::EntranceExit(entrance_exit_details) => Some(entrance_exit_details.stop_lat),
            LocationTypeDetails::GenericNode(generic_node_details) => generic_node_details.stop_lat,
            LocationTypeDetails::BoardingArea(boarding_area_details) => boarding_area_details.stop_lat
        }
    }

    pub fn stop_lon(&self) -> Option<f64> {
        match &self.location_type_details {
            LocationTypeDetails::Stop(stop_details) => Some(stop_details.stop_lon),
            LocationTypeDetails::Station(station_details) => Some(station_details.stop_lon),
            LocationTypeDetails::EntranceExit(entrance_exit_details) => Some(entrance_exit_details.stop_lon),
            LocationTypeDetails::GenericNode(generic_node_details) => generic_node_details.stop_lon,
            LocationTypeDetails::BoardingArea(boarding_area_details) => boarding_area_details.stop_lon
        }
    }

    pub fn parent_station(&self) -> Option<&str> {
        match &self.location_type_details {
            LocationTypeDetails::Stop(stop_details) => stop_details.parent_station.as_deref(),
            LocationTypeDetails::Station(_) => None,
            LocationTypeDetails::EntranceExit(entrance_exit_details) => Some(&entrance_exit_details.parent_station),
            LocationTypeDetails::GenericNode(generic_node_details) => Some(&generic_node_details.parent_station),
            LocationTypeDetails::BoardingArea(boarding_area_details) => Some(&boarding_area_details.parent_station)
        }
    }
}

// Stop implements TryFrom<collections::HashMap<String, String>> by interpreting the keys as field names, and
// the values as string-encoded values for those fields.
impl TryFrom<collections::HashMap<String, String>> for Stop {
    type Error = String;

    fn try_from(fields: collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Stop {
            stop_id: fields.get("stop_id")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_id is required"))?
                .clone(),
            location_type_details: LocationTypeDetails::try_from(&fields)?,
            stop_code: fields.get("stop_code").filter(|s| !s.is_empty()).cloned(),
            tts_stop_name: fields.get("tts_stop_name").filter(|s| !s.is_empty()).cloned(),
            stop_desc: fields.get("stop_desc").filter(|s| !s.is_empty()).cloned(),
            zone_id: fields.get("zone_id").filter(|s| !s.is_empty()).cloned(),
            stop_url: fields.get("stop_url").filter(|s| !s.is_empty()).cloned(),
            stop_timezone: match fields.get("stop_timezone")
                    .filter(|s| !s.is_empty())
                    .map(
                        |stop_timezone_string|
                        Tz::from_str(stop_timezone_string)
                    )
                {
                    Some(res) => res
                        .map(|tz| Some(tz))
                        .map_err(|e| format!("Invalid timezone: {}", e)),
                    None => Ok(None)
                }?,
            wheelchair_boarding: Ok(fields.get("wheelchair_boarding").filter(|s| !s.is_empty()))
                    .and_then(|wheelchair_boarding| match wheelchair_boarding.map(|s| s.as_str()) {
                        None => Ok(None),
                        Some("0") => Ok(None),
                        Some("1") => Ok(Some(true)),
                        Some("2") => Ok(Some(false)),
                        Some(s) => Err(format!("Invalid wheelchair_boarding: {}", s))
                    })?,
            level_id: fields.get("level_id").filter(|s| !s.is_empty()).cloned(),
            platform_code: fields.get("platform_code").filter(|s| !s.is_empty()).cloned()
        })
    }
}

#[derive(Debug)]
pub enum LocationTypeDetails {
    Stop(StopDetails),
    Station(StationDetails),
    EntranceExit(EntranceExitDetails),
    GenericNode(GenericNodeDetails),
    BoardingArea(BoardingAreaDetails),
}

impl TryFrom<&collections::HashMap<String, String>> for LocationTypeDetails {
    type Error = String;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        let zero_str = String::from("0");

        let location_type = fields.get("location_type").unwrap_or(&zero_str);

        match location_type.parse::<u8>()
            .map_err(
                |err|
                format!("invalid location_type '{}': {}", location_type, err)
            )?
        {
            0 => StopDetails::try_from(fields).map(LocationTypeDetails::Stop),
            1 => StationDetails::try_from(fields).map(LocationTypeDetails::Station),
            2 => EntranceExitDetails::try_from(fields).map(LocationTypeDetails::EntranceExit),
            3 => GenericNodeDetails::try_from(fields).map(LocationTypeDetails::GenericNode),
            4 => BoardingAreaDetails::try_from(fields).map(LocationTypeDetails::BoardingArea),
            _ => Err(format!("invalid location_type '{}'", location_type))
        }.map_err(|err| format!("failed to load location as type '{}': {}", location_type, err))
    }
}

#[derive(Debug)]
pub struct StopDetails {
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
    pub parent_station: Option<String>,
}

impl TryFrom<&collections::HashMap<String, String>> for StopDetails {
    type Error = String;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(StopDetails {
            stop_name: fields.get("stop_name")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_name is required"))?
                .clone(),
            stop_lat: fields.get("stop_lat")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_lat is required"))?
                .parse::<f64>()
                .map_err(|err| format!("invalid stop_lat '{}': {}", fields.get("stop_lat").unwrap(), err))?,
            stop_lon: fields.get("stop_lon")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_lon is required"))?
                .parse::<f64>()
                .map_err(|err| format!("invalid stop_lon '{}': {}", fields.get("stop_lon").unwrap(), err))?,
            parent_station: fields.get("parent_station").filter(|s| !s.is_empty()).cloned(),
        })
    }
}

#[derive(Debug)]
pub struct StationDetails {
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
}

impl TryFrom<&collections::HashMap<String, String>> for StationDetails {
    type Error = String;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(StationDetails {
            stop_name: fields.get("stop_name")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_name is required"))?
                .clone(),
            stop_lat: fields.get("stop_lat")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_lat is required"))?
                .parse::<f64>()
                .map_err(|err| format!("invalid stop_lat '{}': {}", fields.get("stop_lat").unwrap(), err))?,
            stop_lon: fields.get("stop_lon")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_lon is required"))?
                .parse::<f64>()
                .map_err(|err| format!("invalid stop_lon '{}': {}", fields.get("stop_lon").unwrap(), err))?,
        })
    }
}

#[derive(Debug)]
pub struct EntranceExitDetails {
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
    pub parent_station: String,
}

impl TryFrom<&collections::HashMap<String, String>> for EntranceExitDetails {
    type Error = String;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(EntranceExitDetails {
            stop_name: fields.get("stop_name")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_name is required"))?
                .clone(),
            stop_lat: fields.get("stop_lat")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_lat is required"))?
                .parse::<f64>()
                .map_err(|err| format!("invalid stop_lat '{}': {}", fields.get("stop_lat").unwrap(), err))?,
            stop_lon: fields.get("stop_lon")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("stop_lon is required"))?
                .parse::<f64>()
                .map_err(|err| format!("invalid stop_lon '{}': {}", fields.get("stop_lon").unwrap(), err))?,
            parent_station: fields.get("parent_station")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("parent_station is required"))?
                .clone(),
        })
    }
}

#[derive(Debug)]
pub struct GenericNodeDetails {
    pub     stop_name: Option<String>,
    pub stop_lat: Option<f64>,
    pub stop_lon: Option<f64>,
    pub parent_station: String,
}

impl TryFrom<&collections::HashMap<String, String>> for GenericNodeDetails {
    type Error = String;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(GenericNodeDetails {
            stop_name: fields.get("stop_name").filter(|s| !s.is_empty()).cloned(),
            stop_lat: match fields.get("stop_lat")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<f64>()
                    .map_err(
                        |err|
                        format!("invalid stop_lat '{}': {}", s, err)
                    ))
                {
                    Some(res) => res.map(|lat| Some(lat)),
                    None => Ok(None)
                }?,
            stop_lon: match fields.get("stop_lon")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<f64>()
                    .map_err(
                        |err|
                        format!("invalid stop_lon '{}': {}", s, err)
                    ))
                {
                    Some(res) => res.map(|lon| Some(lon)),
                    None => Ok(None)
                }?,
            parent_station: fields.get("parent_station")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("parent_station is required"))?
                .clone(),
        })
    }
}

#[derive(Debug)]
pub struct BoardingAreaDetails {
    pub stop_name: Option<String>,
    pub stop_lat: Option<f64>,
    pub stop_lon: Option<f64>,
    pub parent_station: String,
}

impl TryFrom<&collections::HashMap<String, String>> for BoardingAreaDetails {
    type Error = String;

    fn try_from(fields: &collections::HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(BoardingAreaDetails {
            stop_name: fields.get("stop_name").filter(|s| !s.is_empty()).cloned(),
            stop_lat: match fields.get("stop_lat")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<f64>()
                    .map_err(
                        |err|
                        format!("invalid stop_lat '{}': {}", s, err)
                    ))
                {
                    Some(res) => res.map(|lat| Some(lat)),
                    None => Ok(None)
                }?,
            stop_lon: match fields.get("stop_lon")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<f64>()
                    .map_err(
                        |err|
                        format!("invalid stop_lon '{}': {}", s, err)
                    ))
                {
                    Some(res) => res.map(|lon| Some(lon)),
                    None => Ok(None)
                }?,
            parent_station: fields.get("parent_station")
                .filter(|s| !s.is_empty())
                .ok_or(String::from("parent_station is required"))?
                .clone(),
        })
    }
}