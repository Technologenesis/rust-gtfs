pub mod gtfs;
mod stops;
mod routes;
mod trips;
pub trait CommandInterpreter {
    type CommandResult;
    type CommandError: std::error::Error;

    fn interpret(&self, command: &str) -> Result<Self::CommandResult, Self::CommandError>;
}