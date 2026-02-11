# Language Design Decisions

## Temporal Types
### Overview:
Implement a comprehensive set of temporal types to represent time-based data with precision and clarity. The type system distinguishes between different time concepts to avoid common bugs and ambiguities in time handling.

### Temporal Type System:
- **`Date`** — Calendar date only (YYYY-MM-DD). Use for birth dates, anniversaries, or any date without time-of-day concerns.
- **`Time`** — Time of day only (HH:mm:ss). Use for schedules, recurring events, or time-of-day without date context.
- **`DateTime`** — Date + time + timezone (for all user-facing/local time; always explicit). Use for displaying times to users in their local timezone or scheduling events at specific local times.
- **`Timestamp`** — Absolute UTC time, for events/logs/causality. Use for recording when events occurred, ordering events, or any absolute point in time that needs to be comparable across timezones.
- **`Duration`** — Unified duration supporting years, months, days, hours, minutes, seconds, and nanoseconds. Replaces the concept of separate "Period" types. Use for time intervals, elapsed time, or time arithmetic.

### Principles:
- **All date+time must be with timezone (`DateTime`)** — Never use naive date-time without timezone information, as it leads to ambiguity and bugs.
- **`Period` is merged into `Duration`** — A single unified type supports both calendar-based (years, months, days) and precise (hours, minutes, seconds, nanos) durations.
- **Clear separation of concerns** — `DateTime` for zoned/local time (user-facing), `Timestamp` for absolute points/ordering (system-facing).
- **No collections/arrays/primitives in boilerplate** — Temporal types are first-class types, not composite structures.

### Status:
- Accepted

### Consequences:
- **Robust time handling** — Prevents common bugs around timezone conversions, naive date-time usage, and duration calculations.
- **Clear use cases** — Each type has a well-defined purpose, making code intent explicit.
- **Minimal yet evolution-friendly** — The design is simple enough to understand but comprehensive enough to handle real-world time scenarios.
- **Future-proof** — The type system can evolve to support additional time-related operations without breaking existing code.
- Facilitates the creation of time-sensitive applications and improves usability in domains requiring complex date and time handling.

## Structural/Nominal Typing
### Decision:
- Adopt a hybrid typing system that allows both structural and nominal typing. This enables developers to choose between explicit interface definitions and flexible type implementations, enhancing code clarity and reusability.

### Status:
- Accepted

### Consequences:
- Improves type safety without sacrificing convenience; allows developers to enforce type constraints while retaining flexibility.

## Explicit Returns
### Decision:
- Require explicit returns for functions to enhance code readability and predictability in function behavior. This means every function signature must clearly define its return type, ensuring developers understand what to expect from the function output.

### Status:
- Accepted

### Consequences:
- Promotes clearer function definitions, which can aid in debugging and reduces the chance of unintended behavior due to implicit return types.

--- 

## Future Additions
- Here, various design decisions will be documented as they are made. 
- [ ] New decision 1
- [ ] New decision 2
- [ ] New decision 3
- [ ] New decision 4

*Keep this section updated with new design decisions as they arise.*
