use std::fmt;

/// Type information for Flux types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo {
    Int,
    String,
    Table {
        element: Box<TypeInfo>,
    },
    Named {
        name: String,
    },
    Function {
        params: Vec<TypeInfo>,
        ret: Box<TypeInfo>,
    },
    // Temporal types
    /// Calendar date only (YYYY-MM-DD)
    Date,
    /// Time of day only (HH:mm:ss)
    Time,
    /// Date + time + timezone (for all user-facing/local time; always explicit)
    DateTime,
    /// Absolute UTC time, for events/logs/causality
    Timestamp,
    /// Unified duration supporting years, months, days, hours, minutes, seconds, nanos
    Duration,
    Unknown,
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeInfo::Int => write!(f, "int"),
            TypeInfo::String => write!(f, "string"),
            TypeInfo::Table { element } => write!(f, "Table<{}>", element),
            TypeInfo::Named { name } => write!(f, "{}", name),
            TypeInfo::Function { params, ret } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            TypeInfo::Date => write!(f, "Date"),
            TypeInfo::Time => write!(f, "Time"),
            TypeInfo::DateTime => write!(f, "DateTime"),
            TypeInfo::Timestamp => write!(f, "Timestamp"),
            TypeInfo::Duration => write!(f, "Duration"),
            TypeInfo::Unknown => write!(f, "?"),
        }
    }
}

/// Type checker for Flux
pub struct TypeChecker {
    // Placeholder for type checking logic
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn infer_type(&self, _expr: &flux_syntax::Expr) -> TypeInfo {
        // Placeholder - would implement full type inference
        TypeInfo::Unknown
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_types_display() {
        assert_eq!(TypeInfo::Date.to_string(), "Date");
        assert_eq!(TypeInfo::Time.to_string(), "Time");
        assert_eq!(TypeInfo::DateTime.to_string(), "DateTime");
        assert_eq!(TypeInfo::Timestamp.to_string(), "Timestamp");
        assert_eq!(TypeInfo::Duration.to_string(), "Duration");
    }

    #[test]
    fn test_temporal_types_equality() {
        assert_eq!(TypeInfo::Date, TypeInfo::Date);
        assert_eq!(TypeInfo::Time, TypeInfo::Time);
        assert_eq!(TypeInfo::DateTime, TypeInfo::DateTime);
        assert_eq!(TypeInfo::Timestamp, TypeInfo::Timestamp);
        assert_eq!(TypeInfo::Duration, TypeInfo::Duration);

        // Different temporal types should not be equal
        assert_ne!(TypeInfo::Date, TypeInfo::Time);
        assert_ne!(TypeInfo::DateTime, TypeInfo::Timestamp);
        assert_ne!(TypeInfo::Duration, TypeInfo::Date);
    }

    #[test]
    fn test_temporal_types_clone() {
        let date = TypeInfo::Date;
        let cloned = date.clone();
        assert_eq!(date, cloned);

        let duration = TypeInfo::Duration;
        let cloned_duration = duration.clone();
        assert_eq!(duration, cloned_duration);
    }

    #[test]
    fn test_temporal_types_debug() {
        // Ensure Debug trait works for all temporal types
        assert!(format!("{:?}", TypeInfo::Date).contains("Date"));
        assert!(format!("{:?}", TypeInfo::Time).contains("Time"));
        assert!(format!("{:?}", TypeInfo::DateTime).contains("DateTime"));
        assert!(format!("{:?}", TypeInfo::Timestamp).contains("Timestamp"));
        assert!(format!("{:?}", TypeInfo::Duration).contains("Duration"));
    }

    #[test]
    fn test_all_type_info_display() {
        // Test all TypeInfo variants for completeness
        assert_eq!(TypeInfo::Int.to_string(), "int");
        assert_eq!(TypeInfo::String.to_string(), "string");
        assert_eq!(
            TypeInfo::Table {
                element: Box::new(TypeInfo::Int)
            }
            .to_string(),
            "Table<int>"
        );
        assert_eq!(
            TypeInfo::Named {
                name: "MyType".to_string()
            }
            .to_string(),
            "MyType"
        );
        assert_eq!(TypeInfo::Unknown.to_string(), "?");
    }

    #[test]
    fn test_function_type_with_temporal_params() {
        let func_type = TypeInfo::Function {
            params: vec![TypeInfo::DateTime, TypeInfo::Duration],
            ret: Box::new(TypeInfo::Timestamp),
        };
        assert_eq!(func_type.to_string(), "(DateTime, Duration) -> Timestamp");
    }

    #[test]
    fn test_table_with_temporal_element() {
        let table_date = TypeInfo::Table {
            element: Box::new(TypeInfo::Date),
        };
        assert_eq!(table_date.to_string(), "Table<Date>");

        let table_timestamp = TypeInfo::Table {
            element: Box::new(TypeInfo::Timestamp),
        };
        assert_eq!(table_timestamp.to_string(), "Table<Timestamp>");
    }
}
