fn parse_optional_field<T: FromStr>(field: Option<&str>) -> Result<Option<T>, E>
where
    E: From<T::Err>,
{
    field.map(|s| s.parse::<T>()).transpose()
}
