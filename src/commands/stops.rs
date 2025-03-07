use crate::gtfs::stop_times::StopTimes;
use crate::{commands::gtfs::GtfsNode, gtfs::GtfsSchedule};
use crate::commands::CommandInterpreter;
use colored::Colorize;
use crate::commands::gtfs::GTFSCommandInterpreterError;
use std::collections::{self, HashMap, HashSet};
use crate::gtfs::stops::{Stops, Stop};
use crate::gtfs::routes::Routes;
use crate::gtfs::trips::Trips;
pub struct StopsCommandInterpreter<'a>(pub &'a GtfsSchedule);

#[derive(Debug)]
pub enum StopsCommandError {
    InvalidCommand(String),
    ErrorGettingStop(String),
    ErrorExecutingCommandForStop(String, Box<GTFSCommandInterpreterError>),
}

impl std::fmt::Display for StopsCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopsCommandError::InvalidCommand(command) => write!(f, "Invalid command: {}", command),
            StopsCommandError::ErrorGettingStop(stop_id) => write!(f, "Error getting stop: {}", stop_id),
            StopsCommandError::ErrorExecutingCommandForStop(stop_id, cause) => write!(f, "Error executing command for stop {}: {}", stop_id, **cause),
        }
    }
}

impl std::error::Error for StopsCommandError {}

impl<'a> CommandInterpreter for StopsCommandInterpreter<'a> {
    type CommandResult = ();
    type CommandError = StopsCommandError;

    fn interpret(&self, command: &str) -> Result<Self::CommandResult, Self::CommandError> {
        let (first, rest) = command.find(".").and_then(|i| command.split_at_checked(i)).unwrap_or((command, ""));
        match first {
            "list" => Ok(self.list()),
            "info" => Ok(self.info()),
            _ => match self.0.stops.stops.get(first) {
                None => Err(StopsCommandError::InvalidCommand(command.to_string())),
                Some(stop) => self.stop(stop.stop_id.as_str())
                    .map_err(|e| StopsCommandError::ErrorGettingStop(e.to_string()))?
                    .interpret(rest.chars().skip(1).collect::<String>().as_str())
                    .map_err(|e| StopsCommandError::ErrorExecutingCommandForStop(stop.stop_id.clone(), Box::new(e)))
            },
        }
    }
}

#[derive(Debug)]
pub enum StopCommandError {
    NoSuchStop(String),
    ErrorGettingDescendants(String, Box<StopCommandError>),
}

impl std::fmt::Display for StopCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopCommandError::NoSuchStop(stop_id) => write!(f, "No such stop: {}", stop_id),
            StopCommandError::ErrorGettingDescendants(stop_id, cause) => write!(f, "Error getting descendants for stop {}: {}", stop_id, **cause),
        }
    }
}

impl std::error::Error for StopCommandError {}

impl StopsCommandInterpreter<'_> {
    fn list(&self) {
        for (_, stop) in &self.0.stops.stops {
            match stop.get_stop_name() {
                Some(name) => println!("{}: {}", stop.stop_id, name),
                None => println!("{}: {}", stop.stop_id, "Unnamed Location"),
            }
        }
    }

    fn info(&self) {
        println!("{}: {}", "Stops".truecolor(128, 128, 128).bold(), self.0.stops.stops.len());
    }

    fn stop(&self, stop_id: &str) -> Result<GtfsNode, StopCommandError> {
        let raw_stop = self.0.stops.stops.get(stop_id)
            .ok_or(StopCommandError::NoSuchStop(stop_id.to_string()))?;
        
        let stops = self.clone_descendants(stop_id)?;

        let stop_times = self.0.stop_times.iter()
            .filter_map(
                |stop_time|
                stop_time.stop_id.as_ref().and_then(
                    |stop_id|
                    stops.stops.get(stop_id.as_str())
                    .map(|_| (stop_time.trip_id.clone(), stop_time))
                )
            )
            .fold(
                HashMap::new(),
                |mut acc, (trip_id, stop_time)| {
                    acc.entry(trip_id).or_insert(Vec::new()).push(stop_time.clone());
                    acc
                }
            );

        let trips_by_route = (&self.0.trips).into_iter()
            .filter_map(
                |trip|
                stop_times.get(&trip.trip_id).map(|_| (trip.route_id.clone(), trip.clone()))
            )
            .fold(
                HashMap::new(),
                |mut acc, (route_id, trip)| {
                    acc.entry(route_id.clone()).or_insert(Vec::new()).push(trip.clone());
                    acc
                }
            );

        let routes = (&self.0.routes).into_iter()
            .filter_map(
                |route|
                trips_by_route.get(&route.route_id).map(|_| (route.route_id.clone(), route.clone()))
            )
            .collect::<HashMap<_, _>>();

        let trips = trips_by_route.into_iter()
            .map(|(_, trips)| trips.into_iter())
            .flatten()
            .map(|trip| (trip.trip_id.clone(), trip.clone()))
            .collect::<HashMap<_, _>>();
        

        Ok(GtfsNode{
            gtfs: GtfsSchedule{
                stops,
                routes: Routes{
                    routes
                },
                trips: Trips{
                    trips
                },
                stop_times: StopTimes{
                    stop_times
                }
            },
            node_id: stop_id.to_string(),
            node_name: raw_stop.get_stop_name().map(|s| s.to_string()),
            parent: None,
            
        })
    }

    fn clone_descendants(&self, stop_id: &str) -> Result<Stops, StopCommandError> {
        let stops_and_children = self.0.stops.stops.iter().fold(
            HashMap::new(),
            |mut acc, (stop_id, stop)| {
                acc.entry(stop_id.as_str()).or_insert((None, Vec::new())).0 = Some(stop);
                stop.parent_station().map(
                    |parent_id| {
                        acc.entry(parent_id).or_insert((None, Vec::new())).1.push(stop_id.clone());
                    }
                );
                acc
            }
        );
        
        let root = stops_and_children.get(stop_id)
            .and_then(|(maybe_parent, _)| maybe_parent.clone())
            .ok_or(StopCommandError::NoSuchStop(stop_id.to_string()))?;

        let mut descendants = collections::HashMap::new();
        put_descendants(&mut descendants, &root, &stops_and_children)
            .map_err(|e| StopCommandError::ErrorGettingDescendants(stop_id.to_string(), Box::new(e)))?;

        Ok(Stops {
            stops: descendants,
        })
    }
}

fn put_descendants(descendants: &mut HashMap<String, Stop>, stop: &Stop, stops_and_children: &HashMap<&str, (Option<&Stop>, Vec<String>)>) ->Result<(), StopCommandError>  {
    descendants.insert(stop.stop_id.clone(), stop.clone());
    if let Some(children) = stops_and_children.get(stop.stop_id.as_str()).map(|(_, children)| children) {
        for child_id in children {
            let child = stops_and_children.get(child_id.as_str())
                .and_then(|(maybe_parent, _)| maybe_parent.clone())
                .ok_or(StopCommandError::NoSuchStop(child_id.to_string()))?;
            put_descendants(descendants, child, stops_and_children)
                .map_err(|e| StopCommandError::ErrorGettingDescendants(child_id.clone(), Box::new(e)))?;
        }
    }
    Ok(())
}