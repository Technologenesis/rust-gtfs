use crate::commands::gtfs::GtfsNode;
use crate::commands::CommandInterpreter;
use crate::commands::gtfs::GTFSCommandInterpreterError;
use crate::gtfs::GtfsSchedule;
use crate::gtfs::routes::Routes;
use crate::gtfs::trips::Trips;
use crate::gtfs::stops::Stops;
use crate::gtfs::stop_times::StopTimes;
use colored::Colorize;
use std::collections::HashMap;

pub struct RoutesCommandInterpreter<'a>(pub &'a GtfsNode);

#[derive(Debug)]
pub enum RoutesCommandError {
    InvalidCommand(String),
    ErrorGettingRoute(String),
    ErrorExecutingCommandForRoute(String, Box<GTFSCommandInterpreterError>),
    NoSuchRoute(String),
}

impl std::fmt::Display for RoutesCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutesCommandError::InvalidCommand(command) => write!(f, "Invalid command: {}", command),
            RoutesCommandError::ErrorGettingRoute(route_id) => write!(f, "Error getting route: {}", route_id),
            RoutesCommandError::ErrorExecutingCommandForRoute(route_id, cause) => write!(f, "Error executing command for route {}: {}", route_id, **cause),
            RoutesCommandError::NoSuchRoute(route_id) => write!(f, "No such route: {}", route_id),
        }
    }
}

impl std::error::Error for RoutesCommandError {}

impl<'a> CommandInterpreter for RoutesCommandInterpreter<'a> {
    type CommandResult = ();
    type CommandError = RoutesCommandError;

    fn interpret(&self, command: &str) -> Result<Self::CommandResult, Self::CommandError> {
        let (first, rest) = command.find(".").and_then(|i| command.split_at_checked(i)).unwrap_or((command, ""));
        match first {
            "list" => Ok(self.list()),
            "info" => Ok(self.info()),
            _ => match self.0.gtfs.routes.routes.get(first) {
                None => Err(RoutesCommandError::InvalidCommand(command.to_string())),
                Some(route) => self.route(route.route_id.as_str())
                    .map_err(|e| RoutesCommandError::ErrorGettingRoute(e.to_string()))?
                    .interpret(rest.chars().skip(1).collect::<String>().as_str())
                    .map_err(|e| RoutesCommandError::ErrorExecutingCommandForRoute(route.route_id.clone(), Box::new(e)))
            },
        }
    }
}


impl RoutesCommandInterpreter<'_> {
    fn list(&self) {
        for (_, route) in &self.0.gtfs.routes.routes {
            println!("{}: {}", route.route_id, match (route.route_long_name(), route.route_short_name()) {
                (Some(long_name), Some(short_name)) => format!("{} ({})", long_name, short_name),
                _ => route.name()
            });
        }
    }

    fn info(&self) {
        println!("{}: {}", "Routes".truecolor(128, 128, 128).bold(), self.0.gtfs.routes.routes.len());
    }

    fn route(&self, route_id: &str) -> Result<GtfsNode, RoutesCommandError> {
        let raw_route = self.0.gtfs.routes.routes.get(route_id)
            .ok_or(RoutesCommandError::NoSuchRoute(route_id.to_string()))?;

        let routes = Routes{
            routes: HashMap::from([(route_id.to_string(), raw_route.clone())])
        };
        
        let trips = (&self.0.gtfs.trips).into_iter()
            .filter(
                |trip|
                trip.route_id == route_id
            )
            .map(
                |trip|
                (trip.trip_id.clone(), trip.clone())
            )
            .collect::<HashMap<_, _>>();

        let stop_times_by_stop = self.0.gtfs.stop_times.iter()
            .filter_map(
                |stop_time|
                stop_time.stop_id.as_ref()
                    .and_then(
                        |stop_id|
                        trips.get(&stop_time.trip_id)
                        .map(|_| (stop_id, stop_time))
                    )
            )
            .fold(
                HashMap::new(),
                |mut acc, (stop_id, stop_time)| {
                    acc.entry(stop_id).or_insert(Vec::new()).push(stop_time);
                    acc
                }
            );
        
        let stops = (&self.0.gtfs.stops).into_iter()
            .filter_map(
                |stop| stop_times_by_stop.get(&stop.stop_id).map(|_| (stop.stop_id.clone(), stop.clone()))
            )
            .collect::<HashMap<_, _>>();
        
        let stop_times = stop_times_by_stop.into_iter()
            .map(|(stop_id, stop_times)| (stop_id.clone(), stop_times.into_iter().cloned().collect::<Vec<_>>()))
            .collect::<HashMap<_, _>>();
        
        Ok(GtfsNode{
            gtfs: GtfsSchedule{
                routes,
                trips: Trips{
                    trips
                },
                stops: Stops{
                    stops
                },
                stop_times: StopTimes{
                    stop_times
                }
            },
            parent: Some(Box::new(self.0.clone())),
            node_id: route_id.to_string(),
            node_name: Some(raw_route.name())
        })
    }
}