use crate::gtfs::GtfsSchedule;
use crate::commands::CommandInterpreter;

pub struct TripsCommandInterpreter<'a>(pub &'a GtfsSchedule);

#[derive(Debug)]
pub enum TripsCommandError {}

impl std::fmt::Display for TripsCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl std::error::Error for TripsCommandError {}

impl<'a> CommandInterpreter for TripsCommandInterpreter<'a> {
    type CommandResult = ();
    type CommandError = TripsCommandError;

    fn interpret(&self, command: &str) -> Result<Self::CommandResult, Self::CommandError> {
        let (first, rest) = command.find(".").and_then(|i| command.split_at_checked(i)).unwrap_or((command, ""));
        Ok(())
    }
}
